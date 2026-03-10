use std::collections::HashMap;

use common::types::{match_instrument, OptionType, Ticker, VenueId};

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
    pub aevo_taker_rate: f64,
    pub premia_taker_rate: f64,
    pub stryke_protocol_rate: f64,
    pub estimated_gas_cost: f64,
}

impl Default for FeeModel {
    fn default() -> Self {
        Self {
            deribit_taker_rate: 0.0003,
            derive_taker_rate: 0.0005,
            aevo_taker_rate: 0.0005,
            premia_taker_rate: 0.001,
            stryke_protocol_rate: 0.15,
            estimated_gas_cost: 0.05,
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
    pub signal_type: &'static str,
}

#[derive(Debug, Clone)]
pub struct ParitySignal {
    pub venue: VenueId,
    pub instrument_group: String,
    pub parity_gap: f64,
    pub timestamp_ms: i64,
}

pub fn scan_cross_clob_opportunities(tickers: &[Ticker], config: &ScannerConfig) -> Vec<ArbSignal> {
    scan_cross_venue_opportunities(tickers, config)
        .into_iter()
        .filter(|signal| {
            matches!(
                (signal.buy_venue, signal.sell_venue),
                (VenueId::Deribit, VenueId::Derive)
                    | (VenueId::Derive, VenueId::Deribit)
                    | (VenueId::Deribit, VenueId::Aevo)
                    | (VenueId::Aevo, VenueId::Deribit)
                    | (VenueId::Derive, VenueId::Aevo)
                    | (VenueId::Aevo, VenueId::Derive)
            )
        })
        .collect()
}

pub fn scan_cross_venue_opportunities(
    tickers: &[Ticker],
    config: &ScannerConfig,
) -> Vec<ArbSignal> {
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

pub fn scan_put_call_parity(tickers: &[Ticker], risk_free_rate: f64) -> Vec<ParitySignal> {
    let mut grouped: HashMap<(VenueId, String), (Option<Ticker>, Option<Ticker>)> = HashMap::new();

    for ticker in tickers {
        let key = (
            ticker.venue,
            format!(
                "{}:{}:{}",
                ticker.instrument.underlying, ticker.instrument.expiry, ticker.instrument.strike
            ),
        );
        let entry = grouped.entry(key).or_insert((None, None));
        match ticker.instrument.option_type {
            OptionType::Call => entry.0 = Some(ticker.clone()),
            OptionType::Put => entry.1 = Some(ticker.clone()),
        }
    }

    let mut signals = Vec::new();
    for ((venue, group), (call, put)) in grouped {
        let (Some(call), Some(put)) = (call, put) else {
            continue;
        };
        let call_price = call.mid.or(call.mark_price).unwrap_or(0.0);
        let put_price = put.mid.or(put.mark_price).unwrap_or(0.0);
        let spot = call.index_price.or(put.index_price).unwrap_or(0.0);
        let strike = call.instrument.strike;
        let t = year_fraction_from_expiry_code(&call.instrument.expiry);

        let parity_gap =
            (call_price - put_price) - (spot - strike * (-risk_free_rate * t.max(1e-6)).exp());

        if parity_gap.abs() > 1.0 {
            signals.push(ParitySignal {
                venue,
                instrument_group: group,
                parity_gap,
                timestamp_ms: call.timestamp_ms.min(put.timestamp_ms),
            });
        }
    }

    signals
}

pub fn build_alerts(signals: &[ArbSignal]) -> Vec<String> {
    signals
        .iter()
        .map(|signal| {
            format!(
                "[{}] {} buy {:?} sell {:?} spread {:.4} pnl {:.4}",
                signal.signal_type,
                signal.instrument_symbol,
                signal.buy_venue,
                signal.sell_venue,
                signal.iv_spread,
                signal.estimated_pnl
            )
        })
        .collect()
}

pub fn replay_backtest(frames: Vec<Vec<Ticker>>, config: &ScannerConfig) -> Vec<ArbSignal> {
    let mut all = Vec::new();
    for frame in frames {
        all.extend(scan_cross_venue_opportunities(&frame, config));
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
        signal_type: "cross_venue_iv",
    })
}

fn estimated_fees(buy: &Ticker, sell: &Ticker, fee_model: &FeeModel) -> f64 {
    let buy_notional = buy.ask.unwrap_or(0.0);
    let sell_notional = sell.bid.unwrap_or(0.0);

    venue_fee(buy.venue, buy_notional, fee_model) + venue_fee(sell.venue, sell_notional, fee_model)
}

fn venue_fee(venue: VenueId, notional: f64, fee_model: &FeeModel) -> f64 {
    match venue {
        VenueId::Deribit => notional * fee_model.deribit_taker_rate,
        VenueId::Derive => notional * fee_model.derive_taker_rate,
        VenueId::Aevo => notional * fee_model.aevo_taker_rate,
        VenueId::Premia => notional * fee_model.premia_taker_rate + fee_model.estimated_gas_cost,
        VenueId::Stryke => notional * fee_model.stryke_protocol_rate + fee_model.estimated_gas_cost,
    }
}

fn estimated_slippage(buy: &Ticker, sell: &Ticker, bps: f64) -> f64 {
    let notional = buy.ask.unwrap_or(0.0) + sell.bid.unwrap_or(0.0);
    notional * bps / 10_000.0
}

fn year_fraction_from_expiry_code(_expiry: &str) -> f64 {
    30.0 / 365.0
}

#[derive(Debug, Clone)]
pub struct VolLagSignal {
    pub instrument_symbol: String,
    pub deribit_iv: f64,
    pub premia_iv: f64,
    pub oracle_iv: Option<f64>,
    pub iv_gap: f64,
    pub timestamp_ms: i64,
}

pub fn scan_cefi_amm_vol_lag(
    deribit_tickers: &[Ticker],
    premia_tickers: &[Ticker],
    oracle_ivs: &HashMap<String, f64>,
    min_iv_gap: f64,
) -> Vec<VolLagSignal> {
    let mut out = Vec::new();

    for deribit in deribit_tickers {
        for premia in premia_tickers {
            if !match_instrument(&deribit.instrument, &premia.instrument) {
                continue;
            }

            let deribit_iv = deribit.iv.or(deribit.ask_iv).unwrap_or(0.0);
            let premia_iv = premia.iv.or(premia.ask_iv).unwrap_or(0.0);
            let iv_gap = deribit_iv - premia_iv;
            if iv_gap < min_iv_gap {
                continue;
            }

            let oracle_iv = oracle_ivs.get(&deribit.instrument.venue_symbol).copied();
            out.push(VolLagSignal {
                instrument_symbol: deribit.instrument.venue_symbol.clone(),
                deribit_iv,
                premia_iv,
                oracle_iv,
                iv_gap,
                timestamp_ms: deribit.timestamp_ms.min(premia.timestamp_ms),
            });
        }
    }

    out
}

#[derive(Debug, Clone)]
pub struct CrossVenueParitySignal {
    pub instrument_group: String,
    pub venue_a: VenueId,
    pub venue_b: VenueId,
    pub forward_a: f64,
    pub forward_b: f64,
    pub forward_gap: f64,
}

pub fn scan_cross_venue_parity_dislocations(
    tickers: &[Ticker],
    risk_free_rate: f64,
    min_forward_gap: f64,
) -> Vec<CrossVenueParitySignal> {
    let mut venue_forwards: HashMap<(VenueId, String), f64> = HashMap::new();

    for parity in scan_put_call_parity(tickers, risk_free_rate) {
        let (underlying, expiry, strike) = parse_group_key(&parity.instrument_group);
        let t = year_fraction_from_expiry_code(&expiry);
        let forward = strike * (-risk_free_rate * t.max(1e-6)).exp() + parity.parity_gap;
        let key = (parity.venue, format!("{underlying}:{expiry}:{strike}"));
        venue_forwards.insert(key, forward);
    }

    let mut grouped: HashMap<String, Vec<(VenueId, f64)>> = HashMap::new();
    for ((venue, group), forward) in venue_forwards {
        grouped.entry(group).or_default().push((venue, forward));
    }

    let mut signals = Vec::new();
    for (group, values) in grouped {
        for (i, (venue_a, forward_a)) in values.iter().enumerate() {
            for (venue_b, forward_b) in values.iter().skip(i + 1) {
                let gap = forward_a - forward_b;
                if gap.abs() < min_forward_gap {
                    continue;
                }
                signals.push(CrossVenueParitySignal {
                    instrument_group: group.clone(),
                    venue_a: *venue_a,
                    venue_b: *venue_b,
                    forward_a: *forward_a,
                    forward_b: *forward_b,
                    forward_gap: gap,
                });
            }
        }
    }

    signals
}

fn parse_group_key(value: &str) -> (String, String, f64) {
    let mut parts = value.split(':');
    let underlying = parts.next().unwrap_or_default().to_string();
    let expiry = parts.next().unwrap_or_default().to_string();
    let strike = parts
        .next()
        .and_then(|item| item.parse::<f64>().ok())
        .unwrap_or_default();
    (underlying, expiry, strike)
}

#[derive(Debug, Clone)]
pub struct ZeroDteSignal {
    pub instrument_symbol: String,
    pub fair_value: f64,
    pub stryke_price: f64,
    pub expected_edge: f64,
    pub timestamp_ms: i64,
}

pub fn scan_0dte_opportunities(
    deribit_tickers: &[Ticker],
    stryke_tickers: &[Ticker],
    hours_to_expiry: i64,
    risk_free_rate: f64,
    min_edge: f64,
) -> Vec<ZeroDteSignal> {
    let mut out = Vec::new();
    let maturity_years = (hours_to_expiry as f64 / 24.0 / 365.0).max(1e-6);

    for deribit in deribit_tickers {
        for stryke in stryke_tickers {
            if !match_instrument(&deribit.instrument, &stryke.instrument) {
                continue;
            }

            let spot = deribit
                .index_price
                .or(deribit.mark_price)
                .or(stryke.index_price)
                .unwrap_or(0.0);
            let strike = deribit.instrument.strike;
            let iv = deribit.iv.or(deribit.ask_iv).unwrap_or(0.0);
            let option_kind = match deribit.instrument.option_type {
                OptionType::Call => pricing::OptionKind::Call,
                OptionType::Put => pricing::OptionKind::Put,
            };

            let fair_value = pricing::black_scholes_price(
                spot,
                strike,
                maturity_years,
                risk_free_rate,
                iv,
                option_kind,
            );

            let stryke_price = stryke.ask.unwrap_or(stryke.mark_price.unwrap_or(0.0));
            let protocol_fee = stryke_price * 0.15;
            let expected_edge = fair_value - stryke_price - protocol_fee;

            if expected_edge > min_edge {
                out.push(ZeroDteSignal {
                    instrument_symbol: stryke.instrument.venue_symbol.clone(),
                    fair_value,
                    stryke_price,
                    expected_edge,
                    timestamp_ms: deribit.timestamp_ms.min(stryke.timestamp_ms),
                });
            }
        }
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceSignalType {
    Calendar,
    Butterfly,
    CrossVenueSkew,
}

#[derive(Debug, Clone)]
pub struct SurfacePointInput {
    pub venue: String,
    pub strike: f64,
    pub maturity_years: f64,
    pub iv: f64,
}

impl SurfacePointInput {
    pub fn new(venue: &str, strike: f64, maturity_years: f64, iv: f64) -> Self {
        Self {
            venue: venue.to_string(),
            strike,
            maturity_years,
            iv,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceArbSignal {
    pub signal_type: SurfaceSignalType,
    pub description: String,
    pub buy_venue: Option<String>,
    pub sell_venue: Option<String>,
    pub strike: f64,
    pub maturity_years: f64,
}

#[derive(Debug, Clone)]
pub struct SurfaceTradeLeg {
    pub venue: String,
    pub side: &'static str,
    pub strike: f64,
    pub maturity_years: f64,
}

pub fn scan_vol_surface_arbitrage(
    points: &[SurfacePointInput],
    min_calendar_gap: f64,
    min_skew_gap: f64,
) -> Vec<SurfaceArbSignal> {
    let mut signals = Vec::new();

    for (index, left) in points.iter().enumerate() {
        for right in points.iter().skip(index + 1) {
            if (left.strike - right.strike).abs() < 1e-9
                && right.maturity_years > left.maturity_years
            {
                let diff = left.iv - right.iv;
                if diff > min_calendar_gap {
                    signals.push(SurfaceArbSignal {
                        signal_type: SurfaceSignalType::Calendar,
                        description: format!(
                            "calendar inversion strike={} short_iv={} long_iv={}",
                            left.strike, left.iv, right.iv
                        ),
                        buy_venue: Some(right.venue.clone()),
                        sell_venue: Some(left.venue.clone()),
                        strike: left.strike,
                        maturity_years: right.maturity_years,
                    });
                }
            }

            if (left.maturity_years - right.maturity_years).abs() < 1e-6
                && (left.strike - right.strike).abs() > 1.0
                && left.venue != right.venue
            {
                let skew_gap = (left.iv - right.iv).abs();
                if skew_gap > min_skew_gap {
                    let (buy, sell) = if left.iv < right.iv {
                        (left.venue.clone(), right.venue.clone())
                    } else {
                        (right.venue.clone(), left.venue.clone())
                    };
                    signals.push(SurfaceArbSignal {
                        signal_type: SurfaceSignalType::CrossVenueSkew,
                        description: format!("cross venue skew gap={skew_gap}"),
                        buy_venue: Some(buy),
                        sell_venue: Some(sell),
                        strike: left.strike,
                        maturity_years: left.maturity_years,
                    });
                }
            }
        }
    }

    // Simple butterfly convexity check at same maturity on sorted strikes
    let mut by_maturity: HashMap<i64, Vec<&SurfacePointInput>> = HashMap::new();
    for point in points {
        by_maturity
            .entry((point.maturity_years * 10000.0) as i64)
            .or_default()
            .push(point);
    }
    for values in by_maturity.values_mut() {
        values.sort_by(|a, b| {
            a.strike
                .partial_cmp(&b.strike)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for win in values.windows(3) {
            let left = win[0];
            let mid = win[1];
            let right = win[2];
            let wing_avg = (left.iv + right.iv) / 2.0;
            if mid.iv + min_skew_gap < wing_avg {
                signals.push(SurfaceArbSignal {
                    signal_type: SurfaceSignalType::Butterfly,
                    description: format!(
                        "butterfly convexity violation at strike {} maturity {}",
                        mid.strike, mid.maturity_years
                    ),
                    buy_venue: Some(mid.venue.clone()),
                    sell_venue: Some(left.venue.clone()),
                    strike: mid.strike,
                    maturity_years: mid.maturity_years,
                });
            }
        }
    }

    signals
}

pub fn generate_surface_trade_legs(signal: &SurfaceArbSignal) -> Vec<SurfaceTradeLeg> {
    match signal.signal_type {
        SurfaceSignalType::Calendar => vec![
            SurfaceTradeLeg {
                venue: signal
                    .buy_venue
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                side: "BUY",
                strike: signal.strike,
                maturity_years: signal.maturity_years,
            },
            SurfaceTradeLeg {
                venue: signal
                    .sell_venue
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                side: "SELL",
                strike: signal.strike,
                maturity_years: signal.maturity_years / 2.0,
            },
        ],
        SurfaceSignalType::Butterfly => vec![
            SurfaceTradeLeg {
                venue: signal
                    .buy_venue
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                side: "BUY",
                strike: signal.strike,
                maturity_years: signal.maturity_years,
            },
            SurfaceTradeLeg {
                venue: signal
                    .sell_venue
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                side: "SELL",
                strike: signal.strike * 0.95,
                maturity_years: signal.maturity_years,
            },
            SurfaceTradeLeg {
                venue: signal
                    .sell_venue
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                side: "SELL",
                strike: signal.strike * 1.05,
                maturity_years: signal.maturity_years,
            },
        ],
        SurfaceSignalType::CrossVenueSkew => vec![
            SurfaceTradeLeg {
                venue: signal
                    .buy_venue
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                side: "BUY",
                strike: signal.strike,
                maturity_years: signal.maturity_years,
            },
            SurfaceTradeLeg {
                venue: signal
                    .sell_venue
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                side: "SELL",
                strike: signal.strike,
                maturity_years: signal.maturity_years,
            },
        ],
    }
}
