use common::types::{match_instrument, Ticker, VenueId};

#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub min_expected_pnl: f64,
    pub fee_model: FeeModel,
    pub slippage_bps: f64,
}

#[derive(Debug, Clone)]
pub struct FeeModel {
    pub deribit_taker_rate: f64,
    pub derive_taker_rate: f64,
}

impl Default for FeeModel {
    fn default() -> Self {
        Self {
            deribit_taker_rate: 0.0003,
            derive_taker_rate: 0.0005,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArbSignal {
    pub instrument_symbol: String,
    pub buy_venue: VenueId,
    pub sell_venue: VenueId,
    pub iv_spread: f64,
    pub estimated_pnl: f64,
    pub timestamp_ms: i64,
}

pub fn scan_cross_clob_opportunities(tickers: &[Ticker], config: &ScannerConfig) -> Vec<ArbSignal> {
    let mut signals = Vec::new();

    for (index, buy) in tickers.iter().enumerate() {
        for sell in tickers.iter().skip(index + 1) {
            if buy.venue == sell.venue || !match_instrument(&buy.instrument, &sell.instrument) {
                continue;
            }

            if let Some(signal) = build_signal(buy, sell, config) {
                signals.push(signal);
            }
            if let Some(signal) = build_signal(sell, buy, config) {
                signals.push(signal);
            }
        }
    }

    signals
}

pub fn replay_backtest(frames: Vec<Vec<Ticker>>, config: &ScannerConfig) -> Vec<ArbSignal> {
    let mut all = Vec::new();
    for frame in frames {
        all.extend(scan_cross_clob_opportunities(&frame, config));
    }
    all
}

fn build_signal(buy: &Ticker, sell: &Ticker, config: &ScannerConfig) -> Option<ArbSignal> {
    let buy_ask_iv = buy.ask_iv?;
    let sell_bid_iv = sell.bid_iv?;
    if buy_ask_iv >= sell_bid_iv {
        return None;
    }

    let iv_spread = sell_bid_iv - buy_ask_iv;
    let vega = buy.greeks.vega.or(sell.greeks.vega).unwrap_or(0.0);
    let gross = iv_spread * vega;
    let fees = estimated_fees(buy, sell, &config.fee_model);
    let slippage = estimated_slippage(buy, sell, config.slippage_bps);
    let estimated_pnl = gross - fees - slippage;

    if estimated_pnl <= config.min_expected_pnl {
        return None;
    }

    Some(ArbSignal {
        instrument_symbol: buy.instrument.venue_symbol.clone(),
        buy_venue: buy.venue,
        sell_venue: sell.venue,
        iv_spread,
        estimated_pnl,
        timestamp_ms: buy.timestamp_ms.min(sell.timestamp_ms),
    })
}

fn estimated_fees(buy: &Ticker, sell: &Ticker, fee_model: &FeeModel) -> f64 {
    let buy_notional = buy.ask.unwrap_or(0.0);
    let sell_notional = sell.bid.unwrap_or(0.0);

    let buy_fee = match buy.venue {
        VenueId::Deribit => buy_notional * fee_model.deribit_taker_rate,
        VenueId::Derive => buy_notional * fee_model.derive_taker_rate,
        _ => buy_notional * fee_model.derive_taker_rate,
    };
    let sell_fee = match sell.venue {
        VenueId::Deribit => sell_notional * fee_model.deribit_taker_rate,
        VenueId::Derive => sell_notional * fee_model.derive_taker_rate,
        _ => sell_notional * fee_model.derive_taker_rate,
    };

    buy_fee + sell_fee
}

fn estimated_slippage(buy: &Ticker, sell: &Ticker, bps: f64) -> f64 {
    let notional = buy.ask.unwrap_or(0.0) + sell.bid.unwrap_or(0.0);
    notional * bps / 10_000.0
}
