use arb_scanner::scan_cross_venue_parity_dislocations;
use common::types::{Greeks, Instrument, OptionStyle, OptionType, Ticker, VenueId};

fn option_ticker(venue: VenueId, kind: OptionType, mid: f64) -> Ticker {
    Ticker {
        instrument: Instrument {
            underlying: "BTC".into(),
            strike: 50000.0,
            expiry: "27DEC24".into(),
            option_type: kind,
            style: OptionStyle::European,
            venue,
            venue_symbol: format!(
                "BTC-27DEC24-50000-{}",
                if matches!(kind, OptionType::Call) {
                    "C"
                } else {
                    "P"
                }
            ),
        },
        venue,
        bid: Some(mid - 1.0),
        ask: Some(mid + 1.0),
        mid: Some(mid),
        mark_price: Some(mid),
        index_price: Some(51000.0),
        iv: Some(0.6),
        bid_iv: Some(0.59),
        ask_iv: Some(0.61),
        greeks: Greeks::default(),
        timestamp_ms: 1,
    }
}

#[test]
fn detects_cross_venue_synthetic_forward_gap() {
    let deribit_call = option_ticker(VenueId::Deribit, OptionType::Call, 2000.0);
    let deribit_put = option_ticker(VenueId::Deribit, OptionType::Put, 1200.0);

    let aevo_call = option_ticker(VenueId::Aevo, OptionType::Call, 2400.0);
    let aevo_put = option_ticker(VenueId::Aevo, OptionType::Put, 900.0);

    let signals = scan_cross_venue_parity_dislocations(
        &[deribit_call, deribit_put, aevo_call, aevo_put],
        0.01,
        100.0,
    );

    assert_eq!(signals.len(), 1);
    assert!(signals[0].forward_gap.abs() > 100.0);
}
