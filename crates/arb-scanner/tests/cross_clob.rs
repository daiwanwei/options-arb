use arb_scanner::{scan_cross_clob_opportunities, FeeModel, ScannerConfig};
use common::types::{Greeks, Instrument, OptionStyle, OptionType, Ticker, VenueId};

fn sample_ticker(venue: VenueId, bid_iv: f64, ask_iv: f64, vega: f64) -> Ticker {
    Ticker {
        instrument: Instrument {
            underlying: "BTC".to_string(),
            strike: 50_000.0,
            expiry: "27DEC24".to_string(),
            option_type: OptionType::Call,
            style: OptionStyle::European,
            venue,
            venue_symbol: "BTC-27DEC24-50000-C".to_string(),
        },
        venue,
        bid: Some(1000.0),
        ask: Some(1010.0),
        mid: Some(1005.0),
        mark_price: Some(1005.0),
        index_price: Some(50_000.0),
        iv: Some((bid_iv + ask_iv) / 2.0),
        bid_iv: Some(bid_iv),
        ask_iv: Some(ask_iv),
        greeks: Greeks {
            delta: Some(0.5),
            gamma: Some(0.0),
            theta: Some(0.0),
            vega: Some(vega),
            rho: Some(0.0),
        },
        timestamp_ms: 1,
    }
}

#[test]
fn finds_cross_clob_signal_when_spread_beats_fees() {
    let deribit = sample_ticker(VenueId::Deribit, 0.62, 0.63, 120.0);
    let derive = sample_ticker(VenueId::Derive, 0.70, 0.71, 120.0);

    let config = ScannerConfig {
        min_expected_pnl: 1.0,
        fee_model: FeeModel::default(),
        slippage_bps: 1.0,
    };

    let signals = scan_cross_clob_opportunities(&[deribit, derive], &config);
    assert_eq!(signals.len(), 1);
    assert!(signals[0].estimated_pnl > 0.0);
}

#[test]
fn ignores_pairs_when_spread_too_small() {
    let deribit = sample_ticker(VenueId::Deribit, 0.60, 0.61, 100.0);
    let derive = sample_ticker(VenueId::Derive, 0.61, 0.62, 100.0);

    let config = ScannerConfig {
        min_expected_pnl: 10.0,
        fee_model: FeeModel::default(),
        slippage_bps: 5.0,
    };

    let signals = scan_cross_clob_opportunities(&[deribit, derive], &config);
    assert!(signals.is_empty());
}
