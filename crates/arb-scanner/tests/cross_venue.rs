use arb_scanner::{
    build_alerts, scan_cross_venue_opportunities, scan_put_call_parity, FeeModel, ScannerConfig,
};
use common::types::{Greeks, Instrument, OptionStyle, OptionType, Ticker, VenueId};

fn ticker(venue: VenueId, option_type: OptionType, bid_iv: f64, ask_iv: f64) -> Ticker {
    Ticker {
        instrument: Instrument {
            underlying: "ETH".to_string(),
            strike: 3000.0,
            expiry: "28MAR26".to_string(),
            option_type,
            style: OptionStyle::European,
            venue,
            venue_symbol: format!(
                "ETH-28MAR26-3000-{}",
                if matches!(option_type, OptionType::Call) {
                    "C"
                } else {
                    "P"
                }
            ),
        },
        venue,
        bid: Some(200.0),
        ask: Some(205.0),
        mid: Some(202.5),
        mark_price: Some(202.5),
        index_price: Some(2950.0),
        iv: Some((bid_iv + ask_iv) / 2.0),
        bid_iv: Some(bid_iv),
        ask_iv: Some(ask_iv),
        greeks: Greeks {
            delta: Some(0.5),
            gamma: Some(0.0),
            theta: Some(0.0),
            vega: Some(150.0),
            rho: Some(0.0),
        },
        timestamp_ms: 1,
    }
}

#[test]
fn finds_cross_venue_signals_including_defi() {
    let t1 = ticker(VenueId::Premia, OptionType::Call, 0.60, 0.61);
    let t2 = ticker(VenueId::Deribit, OptionType::Call, 0.70, 0.71);

    let config = ScannerConfig {
        min_expected_pnl: 1.0,
        fee_model: FeeModel::default(),
        slippage_bps: 1.0,
    };

    let signals = scan_cross_venue_opportunities(&[t1, t2], &config);
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].signal_type, "cross_venue_iv");
}

#[test]
fn builds_alert_messages() {
    let t1 = ticker(VenueId::Deribit, OptionType::Call, 0.60, 0.61);
    let t2 = ticker(VenueId::Aevo, OptionType::Call, 0.72, 0.73);
    let config = ScannerConfig {
        min_expected_pnl: 1.0,
        fee_model: FeeModel::default(),
        slippage_bps: 1.0,
    };
    let signals = scan_cross_venue_opportunities(&[t1, t2], &config);
    let alerts = build_alerts(&signals);
    assert_eq!(alerts.len(), 1);
    assert!(alerts[0].contains("buy"));
}

#[test]
fn detects_put_call_parity_gap() {
    let mut call = ticker(VenueId::Deribit, OptionType::Call, 0.60, 0.61);
    call.mid = Some(300.0);
    let mut put = ticker(VenueId::Deribit, OptionType::Put, 0.60, 0.61);
    put.mid = Some(100.0);

    let parity = scan_put_call_parity(&[call, put], 0.01);
    assert!(!parity.is_empty());
}
