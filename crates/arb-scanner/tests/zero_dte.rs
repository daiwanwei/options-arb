use arb_scanner::scan_0dte_opportunities;
use common::types::{Greeks, Instrument, OptionStyle, OptionType, Ticker, VenueId};

fn mk(venue: VenueId, option_type: OptionType, ask: f64, iv: f64) -> Ticker {
    Ticker {
        instrument: Instrument {
            underlying: "ETH".into(),
            strike: 3000.0,
            expiry: "10MAR26".into(),
            option_type,
            style: OptionStyle::European,
            venue,
            venue_symbol: "ETH-10MAR26-3000-C".into(),
        },
        venue,
        bid: Some(ask - 1.0),
        ask: Some(ask),
        mid: Some(ask - 0.5),
        mark_price: Some(ask),
        index_price: Some(3200.0),
        iv: Some(iv),
        bid_iv: Some(iv - 0.01),
        ask_iv: Some(iv + 0.01),
        greeks: Greeks {
            delta: Some(0.5),
            gamma: Some(0.0),
            theta: Some(0.0),
            vega: Some(20.0),
            rho: Some(0.0),
        },
        timestamp_ms: 1,
    }
}

#[test]
fn detects_0dte_opportunity_when_stryke_is_cheap() {
    let deribit = mk(VenueId::Deribit, OptionType::Call, 220.0, 0.7);
    let stryke = mk(VenueId::Stryke, OptionType::Call, 120.0, 0.2);

    let signals = scan_0dte_opportunities(&[deribit], &[stryke], 6, 0.01, 10.0);
    assert_eq!(signals.len(), 1);
    assert!(signals[0].expected_edge > 0.0);
}
