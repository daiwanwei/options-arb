use common::types::VenueId;
use connector_derive::{
    build_json_rpc_request, reconnect_delay_ms, to_unified_ticker, RawDeriveTicker,
};

#[test]
fn builds_json_rpc_payload() {
    let payload = build_json_rpc_request(
        1,
        "public/get_ticker",
        serde_json::json!({"instrument_name": "ETH-28MAR26-3000-C"}),
    );
    assert_eq!(payload["jsonrpc"], "2.0");
    assert_eq!(payload["id"], 1);
    assert_eq!(payload["method"], "public/get_ticker");
}

#[test]
fn converts_derive_raw_ticker() {
    let raw = RawDeriveTicker {
        instrument_name: "ETH-28MAR26-3000-C".to_string(),
        best_bid_price: Some(220.0),
        best_ask_price: Some(230.0),
        mark_price: Some(225.0),
        index_price: Some(2950.0),
        option_iv: Some(0.60),
        bid_iv: Some(0.59),
        ask_iv: Some(0.61),
        timestamp_ms: 123,
    };

    let ticker = to_unified_ticker(raw).expect("conversion works");
    assert_eq!(ticker.venue, VenueId::Derive);
    assert_eq!(ticker.iv, Some(0.60));
}

#[test]
fn reconnect_backoff_caps() {
    assert_eq!(reconnect_delay_ms(0), 500);
    assert_eq!(reconnect_delay_ms(1), 1000);
    assert_eq!(reconnect_delay_ms(10), 30_000);
}
