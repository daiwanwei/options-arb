use common::types::{match_instrument, OptionType, VenueId};

#[test]
fn parses_clob_symbol_into_instrument() {
    let instrument =
        common::types::Instrument::from_clob_symbol(VenueId::Deribit, "BTC-27DEC24-50000-C")
            .expect("instrument should parse");

    assert_eq!(instrument.underlying, "BTC");
    assert_eq!(instrument.strike, 50_000.0);
    assert_eq!(instrument.option_type, OptionType::Call);
}

#[test]
fn matches_instruments_across_venues() {
    let a = common::types::Instrument::from_clob_symbol(VenueId::Deribit, "ETH-28MAR26-3000-C")
        .unwrap();
    let b =
        common::types::Instrument::from_clob_symbol(VenueId::Derive, "ETH-28MAR26-3000-C").unwrap();

    assert!(match_instrument(&a, &b));
}

#[test]
fn converts_deribit_ticker_to_unified_ticker() {
    let raw = common::types::DeribitTicker {
        instrument_name: "BTC-27DEC24-50000-C".to_string(),
        best_bid_price: Some(1000.0),
        best_ask_price: Some(1010.0),
        mark_price: Some(1005.0),
        index_price: Some(50_000.0),
        iv: Some(0.55),
        bid_iv: Some(0.54),
        ask_iv: Some(0.56),
        greeks: common::types::Greeks::default(),
        timestamp_ms: 1,
    };

    let ticker = common::types::Ticker::try_from(raw).expect("conversion should work");
    assert_eq!(ticker.venue, VenueId::Deribit);
    assert_eq!(ticker.mid, Some(1005.0));
}

#[test]
fn converts_derive_ticker_to_unified_ticker() {
    let raw = common::types::DeriveTicker {
        instrument_name: "ETH-28MAR26-3000-C".to_string(),
        best_bid_price: Some(220.0),
        best_ask_price: Some(230.0),
        mark_price: Some(225.0),
        index_price: Some(2_950.0),
        option_iv: Some(0.60),
        bid_iv: Some(0.59),
        ask_iv: Some(0.61),
        greeks: common::types::Greeks::default(),
        timestamp_ms: 1,
    };

    let ticker = common::types::Ticker::try_from(raw).expect("conversion should work");
    assert_eq!(ticker.venue, VenueId::Derive);
    assert_eq!(ticker.iv, Some(0.60));
}
