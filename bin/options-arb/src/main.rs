use anyhow::Result;
use arb_scanner::{scan_cross_venue_opportunities, FeeModel, ScannerConfig};
use common::AppConfig;
use common::types::{Greeks, Instrument, OptionStyle, OptionType, Ticker, VenueId};
use executor::{render_prometheus, PaperFill, PaperTrader, PrometheusSnapshot};
use futures_util::StreamExt;
use risk_manager::{evaluate_pre_trade, MarginState, Position, RiskConfig, RiskLimits, TradeIntent};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{Duration, Instant};
use tokio_stream as stream;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = AppConfig::load()?;
    tracing::info!(environment = %cfg.environment, "options-arb boot");

    if let Ok(bind) = std::env::var("METRICS_BIND") {
        run_metrics_server(&bind).await?;
    }

    if std::env::var("PAPER_TRADING_LOOP").ok().as_deref() == Some("1") {
        let duration_secs = std::env::var("PAPER_LOOP_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(60);
        run_paper_trading_loop(Duration::from_secs(duration_secs)).await?;
    }

    Ok(())
}

async fn run_paper_trading_loop(run_for: Duration) -> Result<()> {
    let mut paper_trader = PaperTrader::default();
    let scanner_config = ScannerConfig {
        min_expected_pnl: -1.0,
        fee_model: FeeModel {
            deribit_taker_rate: 0.0,
            derive_taker_rate: 0.0,
            aevo_taker_rate: 0.0,
            premia_taker_rate: 0.0,
            stryke_protocol_rate: 0.0,
            estimated_gas_cost: 0.0,
        },
        slippage_bps: 0.0,
    };
    let risk_config = RiskConfig {
        limits: RiskLimits {
            max_position_per_instrument: 100.0,
            max_position_per_underlying: 500.0,
            max_abs_delta: 1_000.0,
            max_abs_gamma: 1_000.0,
            max_abs_vega: 1_000.0,
            max_margin_utilization: 0.95,
        },
    };

    let start = Instant::now();
    let mut frame_count = 0_u64;
    let mut tickers = stream::iter((0_u64..).map(simulated_deribit_ticker));

    while start.elapsed() < run_for {
        if let Some(deribit) = tickers.next().await {
            frame_count += 1;
            process_ticker_frame(&mut paper_trader, &scanner_config, &risk_config, deribit)?;

            if frame_count % 60 == 0 {
                let minutes = (start.elapsed().as_secs_f64() / 60.0).max(1.0 / 60.0);
                let snapshot = paper_trader.metrics_snapshot(minutes);
                tracing::info!(
                    signals_per_min = snapshot.signals_per_min,
                    fill_rate = snapshot.fill_rate,
                    pnl = snapshot.pnl,
                    avg_latency_ms = snapshot.avg_latency_ms,
                    "paper trading snapshot"
                );
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

fn process_ticker_frame(
    paper_trader: &mut PaperTrader,
    scanner_config: &ScannerConfig,
    risk_config: &RiskConfig,
    deribit_ticker: Ticker,
) -> Result<()> {
    let aevo_ticker = synthesize_cross_venue_ticker(&deribit_ticker, VenueId::Aevo);
    let frame = vec![deribit_ticker.clone(), aevo_ticker];
    let signals = scan_cross_venue_opportunities(&frame, scanner_config);

    for signal in signals {
        let intent = TradeIntent::new(
            &signal.instrument_symbol,
            &deribit_ticker.instrument.underlying,
            1.0,
            0.0,
            0.0,
            0.0,
        );
        let positions: Vec<Position> = Vec::new();
        let margin = MarginState { utilization: 0.1 };
        if evaluate_pre_trade(risk_config, &positions, &margin, &intent).is_err() {
            continue;
        }

        let entry = frame
            .iter()
            .find(|item| item.venue == signal.buy_venue)
            .and_then(|item| item.ask.or(item.mark_price))
            .unwrap_or(0.0);
        let exit = frame
            .iter()
            .find(|item| item.venue == signal.sell_venue)
            .and_then(|item| item.bid.or(item.mark_price))
            .unwrap_or(entry);

        paper_trader.record_fill(PaperFill::new(
            &signal.instrument_symbol,
            entry,
            exit,
            1.0,
        ));
    }

    Ok(())
}

fn simulated_deribit_ticker(step: u64) -> Ticker {
    let bid_iv = 0.50 + ((step % 10) as f64) * 0.001;
    let ask_iv = bid_iv + 0.01;
    let bid = 100.0 + ((step % 5) as f64);
    let ask = bid + 1.0;
    Ticker {
        instrument: Instrument {
            underlying: "ETH".to_string(),
            strike: 3000.0,
            expiry: "28MAR26".to_string(),
            option_type: OptionType::Call,
            style: OptionStyle::European,
            venue: VenueId::Deribit,
            venue_symbol: "ETH-28MAR26-3000-C".to_string(),
        },
        venue: VenueId::Deribit,
        bid: Some(bid),
        ask: Some(ask),
        mid: Some((bid + ask) / 2.0),
        mark_price: Some((bid + ask) / 2.0),
        index_price: Some(3000.0),
        iv: Some((bid_iv + ask_iv) / 2.0),
        bid_iv: Some(bid_iv),
        ask_iv: Some(ask_iv),
        greeks: Greeks::default(),
        timestamp_ms: (step as i64) * 1000,
    }
}

fn synthesize_cross_venue_ticker(base: &Ticker, venue: VenueId) -> Ticker {
    let mut cloned = base.clone();
    cloned.venue = venue;
    cloned.instrument.venue = venue;
    cloned.bid = base.bid.map(|value| value + 0.4);
    cloned.ask = base.ask.map(|value| value + 1.4);
    cloned.bid_iv = base.bid_iv.map(|value| value + 0.2);
    cloned.ask_iv = base.ask_iv.map(|value| value + 0.2);
    cloned.iv = cloned
        .bid_iv
        .zip(cloned.ask_iv)
        .map(|(bid_iv, ask_iv)| (bid_iv + ask_iv) / 2.0);
    cloned
}

async fn run_metrics_server(bind: &str) -> Result<()> {
    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "metrics endpoint started");

    loop {
        let (mut stream, _addr) = match listener.accept().await {
            Ok(value) => value,
            Err(err) => {
                tracing::warn!(error = %err, "incoming metrics stream error");
                continue;
            }
        };

        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await;
        let request_text = String::from_utf8_lossy(&request);

        let response = build_http_response(&request_text);
        stream.write_all(response.as_bytes()).await?;
    }
}

fn build_http_response(request_text: &str) -> String {
    if request_text.starts_with("GET /metrics") {
        let snapshot = PrometheusSnapshot {
            signals_per_min: 0.0,
            fill_rate: 0.0,
            pnl: PaperTrader::default().total_pnl(),
            avg_latency_ms: 0.0,
        };
        let body = render_prometheus(&snapshot, 0.0);
        return format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
    }

    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::{build_http_response, process_ticker_frame, simulated_deribit_ticker};
    use arb_scanner::{FeeModel, ScannerConfig};
    use executor::PaperTrader;
    use risk_manager::{RiskConfig, RiskLimits};

    #[test]
    fn metrics_path_returns_ok() {
        let response = build_http_response("GET /metrics HTTP/1.1\r\n\r\n");
        assert!(response.starts_with("HTTP/1.1 200 OK"));
    }

    #[test]
    fn unknown_path_returns_404() {
        let response = build_http_response("GET /unknown HTTP/1.1\r\n\r\n");
        assert!(response.starts_with("HTTP/1.1 404 Not Found"));
    }

    #[test]
    fn paper_loop_generates_fills_from_signals() {
        let mut trader = PaperTrader::default();
        let scanner_config = ScannerConfig {
            min_expected_pnl: -1.0,
            fee_model: FeeModel {
                deribit_taker_rate: 0.0,
                derive_taker_rate: 0.0,
                aevo_taker_rate: 0.0,
                premia_taker_rate: 0.0,
                stryke_protocol_rate: 0.0,
                estimated_gas_cost: 0.0,
            },
            slippage_bps: 0.0,
        };
        let risk_config = RiskConfig {
            limits: RiskLimits {
                max_position_per_instrument: 100.0,
                max_position_per_underlying: 500.0,
                max_abs_delta: 1_000.0,
                max_abs_gamma: 1_000.0,
                max_abs_vega: 1_000.0,
                max_margin_utilization: 0.95,
            },
        };

        for step in 0..10 {
            let ticker = simulated_deribit_ticker(step);
            process_ticker_frame(&mut trader, &scanner_config, &risk_config, ticker)
                .expect("paper frame processing should succeed");
        }

        let snapshot = trader.metrics_snapshot(1.0);
        assert!(snapshot.fill_rate > 0.0);
    }
}
