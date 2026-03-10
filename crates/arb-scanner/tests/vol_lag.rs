use std::collections::HashMap;

use arb_scanner::scan_cefi_amm_vol_lag;
use common::types::{Greeks, Instrument, OptionStyle, OptionType, Ticker, VenueId};

fn mk(venue: VenueId, bid_iv: f64, ask_iv: f64) -> Ticker {
    Ticker {
        instrument: Instrument {
            underlying: "ETH".into(),
            strike: 3000.0,
            expiry: "28MAR26".into(),
            option_type: OptionType::Call,
            style: OptionStyle::European,
            venue,
            venue_symbol: "ETH-28MAR26-3000-C".into(),
        },
        venue,
        bid: Some(220.0),
        ask: Some(225.0),
        mid: Some(222.5),
        mark_price: Some(222.5),
        index_price: Some(2950.0),
        iv: Some((bid_iv + ask_iv) / 2.0),
        bid_iv: Some(bid_iv),
        ask_iv: Some(ask_iv),
        greeks: Greeks {
            delta: Some(0.5),
            gamma: Some(0.0),
            theta: Some(0.0),
            vega: Some(120.0),
            rho: Some(0.0),
        },
        timestamp_ms: 1,
    }
}

#[test]
fn detects_vol_lag_when_deribit_spikes_above_premia() {
    let deribit = mk(VenueId::Deribit, 0.90, 0.92);
    let premia = mk(VenueId::Premia, 0.62, 0.64);
    let mut oracle = HashMap::new();
    oracle.insert("ETH-28MAR26-3000-C".to_string(), 0.63);

    let signals = scan_cefi_amm_vol_lag(&[deribit], &[premia], &oracle, 0.1);
    assert_eq!(signals.len(), 1);
    assert!(signals[0].iv_gap > 0.2);
}
