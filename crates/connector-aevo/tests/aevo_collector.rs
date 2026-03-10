use connector_aevo::{
    build_markets_url, build_subscribe_message, compute_orderbook_checksum, extract_aevo_order_id,
    orderbook_channel, parse_orderbook_message, sign_aevo_request, trades_channel,
    AevoLocalOrderBook, AevoWsClient,
};

#[test]
fn builds_markets_endpoint() {
    let url = build_markets_url("ETH", "OPTION");
    assert_eq!(
        url,
        "https://api.aevo.xyz/markets?asset=ETH&instrument_type=OPTION"
    );
}

#[test]
fn builds_ws_channels() {
    assert_eq!(
        orderbook_channel("ETH-28MAR25-2000-C"),
        "orderbook-100ms:ETH-28MAR25-2000-C"
    );
    assert_eq!(trades_channel("ETH"), "trades:ETH");
}

#[test]
fn validates_checksum_after_delta() {
    let mut book = AevoLocalOrderBook::new();
    book.apply_snapshot(vec![(100.0, 1.0)], vec![(101.0, 2.0)]);

    let mut expected_book = book.clone();
    expected_book
        .apply_delta(vec![(100.0, 2.0)], vec![], u64::MAX)
        .ok();
    let expected_checksum = compute_orderbook_checksum(&expected_book);

    assert!(book
        .apply_delta(vec![(100.0, 2.0)], vec![], expected_checksum)
        .is_ok());
}

#[test]
fn builds_ws_subscribe_message() {
    let channels = vec![orderbook_channel("ETH-28MAR25-2000-C")];
    let payload = build_subscribe_message(&channels);
    assert_eq!(payload["op"].as_str(), Some("subscribe"));
}

#[test]
fn signs_aevo_request_with_hmac() {
    let signature = sign_aevo_request(
        "secret",
        1710000000000,
        "POST",
        "/orders",
        "{\"instrument\":\"ETH-28MAR25-2000-C\"}",
    )
    .expect("signature should be generated");
    assert_eq!(signature.len(), 64);
}

#[test]
fn parses_orderbook_snapshot_payload() {
    let mut book = AevoLocalOrderBook::new();
    let text = r#"{
      "type": "orderbook",
      "data": {
        "snapshot": true,
        "bids": [[100.0, 2.0]],
        "asks": [[101.0, 1.5]],
        "checksum": 202
      }
    }"#;
    let updated = parse_orderbook_message(text, &mut book)
        .expect("parse should succeed")
        .expect("book should be emitted");
    assert_eq!(updated.bids.len(), 1);
    assert_eq!(updated.asks.len(), 1);
}

#[test]
fn extracts_order_id_from_aevo_response() {
    let value = serde_json::json!({ "order_id": "aevo-123" });
    let order_id = extract_aevo_order_id(&value).expect("order id should exist");
    assert_eq!(order_id, "aevo-123");
}

#[test]
fn can_build_ws_client() {
    let _client = AevoWsClient::new(connector_aevo::AEVO_WS_BASE);
}
