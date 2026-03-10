use connector_deribit::{
    backoff_delay_ms, build_set_heartbeat_request, build_subscribe_request, channel_names,
    extract_order_id, heartbeat_reply_if_requested, parse_auth_response,
    parse_orderbook_notification, parse_ticker_notification, reset_local_orderbook_on_reconnect,
    DeribitWsClient, LocalOrderBook,
};

#[test]
fn builds_expected_subscription_channels() {
    let channels = channel_names("BTC-27DEC24-50000-C");
    assert!(channels.contains(&"book.BTC-27DEC24-50000-C.100ms".to_string()));
    assert!(channels.contains(&"ticker.BTC-27DEC24-50000-C.100ms".to_string()));
    assert!(channels.contains(&"trades.BTC-27DEC24-50000-C.raw".to_string()));
}

#[test]
fn rejects_out_of_order_book_sequence() {
    let mut book = LocalOrderBook::new();

    assert!(book
        .apply_snapshot(10, vec![(100.0, 1.0)], vec![(101.0, 1.0)])
        .is_ok());
    assert!(book
        .apply_delta(9, vec![(100.0, 2.0)], vec![(101.0, 2.0)])
        .is_err());
}

#[test]
fn backoff_is_capped() {
    assert_eq!(backoff_delay_ms(0), 500);
    assert_eq!(backoff_delay_ms(1), 1000);
    assert_eq!(backoff_delay_ms(10), 30_000);
}

#[test]
fn builds_public_subscribe_rpc_request() {
    let channels = vec!["ticker.BTC-28MAR26-50000-C.100ms".to_string()];
    let payload = build_subscribe_request(7, &channels);
    assert_eq!(payload["id"].as_u64(), Some(7));
    assert_eq!(payload["method"].as_str(), Some("public/subscribe"));
}

#[test]
fn builds_set_heartbeat_request() {
    let payload = build_set_heartbeat_request(9, 60);
    assert_eq!(payload["method"].as_str(), Some("public/set_heartbeat"));
    assert_eq!(payload["params"]["interval"].as_u64(), Some(60));
}

#[test]
fn parses_ticker_subscription_payload() {
    let text = r#"{
      "jsonrpc":"2.0",
      "method":"subscription",
      "params":{
        "channel":"ticker.BTC-28MAR26-50000-C.100ms",
        "data":{
          "instrument_name":"BTC-28MAR26-50000-C",
          "best_bid_price":123.4,
          "best_ask_price":124.5,
          "mark_price":124.0,
          "index_price":50000.0,
          "mark_iv":0.55,
          "bid_iv":0.54,
          "ask_iv":0.56,
          "timestamp":1700000000000
        }
      }
    }"#;

    let ticker = parse_ticker_notification(text)
        .expect("parse ok")
        .expect("ticker present");
    assert_eq!(ticker.instrument.venue_symbol, "BTC-28MAR26-50000-C");
    assert_eq!(ticker.bid_iv, Some(0.54));
    assert_eq!(ticker.ask_iv, Some(0.56));
}

#[test]
fn parses_orderbook_snapshot_then_delta() {
    let mut local = LocalOrderBook::new();
    let snapshot = r#"{
      "jsonrpc":"2.0",
      "method":"subscription",
      "params":{
        "channel":"book.BTC-28MAR26-50000-C.100ms",
        "data":{
          "type":"snapshot",
          "change_id":1,
          "timestamp":1700000000000,
          "bids":[["new",123.0,2.0]],
          "asks":[["new",124.0,1.5]]
        }
      }
    }"#;

    let delta = r#"{
      "jsonrpc":"2.0",
      "method":"subscription",
      "params":{
        "channel":"book.BTC-28MAR26-50000-C.100ms",
        "data":{
          "type":"change",
          "change_id":2,
          "timestamp":1700000000100,
          "bids":[["change",123.0,3.0]],
          "asks":[["change",124.0,1.0]]
        }
      }
    }"#;

    let book1 = parse_orderbook_notification(snapshot, "BTC-28MAR26-50000-C", &mut local)
        .expect("snapshot parse ok")
        .expect("book exists");
    let book2 = parse_orderbook_notification(delta, "BTC-28MAR26-50000-C", &mut local)
        .expect("delta parse ok")
        .expect("book exists");

    assert_eq!(book1.bids[0].price, 123.0);
    assert_eq!(book2.bids[0].size, 3.0);
}

#[test]
fn responds_to_deribit_test_request_heartbeat() {
    let incoming = r#"{
      "jsonrpc": "2.0",
      "id": 13,
      "method": "heartbeat",
      "params": {"type": "test_request"}
    }"#;

    let reply = heartbeat_reply_if_requested(incoming)
        .expect("parsing should succeed")
        .expect("heartbeat should need reply");
    assert_eq!(reply["method"].as_str(), Some("public/test"));
}

#[test]
fn resets_local_book_on_reconnect() {
    let mut book = LocalOrderBook::new();
    book.apply_snapshot(3, vec![(100.0, 1.0)], vec![(101.0, 2.0)])
        .expect("snapshot should apply");

    reset_local_orderbook_on_reconnect(&mut book);
    assert!(book.last_sequence.is_none());
    assert!(book.bids.is_empty());
    assert!(book.asks.is_empty());
}

#[test]
fn client_uses_expected_default_url() {
    let client = DeribitWsClient::new(connector_deribit::DERIBIT_TESTNET_WS);
    assert_eq!(client.url(), connector_deribit::DERIBIT_TESTNET_WS);
}

#[test]
fn parses_deribit_auth_response() {
    let value = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {
            "access_token": "token-abc",
            "expires_in": 900
        }
    });

    let auth = parse_auth_response(&value).expect("auth parse should succeed");
    assert_eq!(auth.access_token, "token-abc");
    assert_eq!(auth.expires_in, 900);
}

#[test]
fn extracts_order_id_from_private_response() {
    let value = serde_json::json!({
        "result": {
            "order": {
                "order_id": "ETH-123"
            }
        }
    });

    let order_id = extract_order_id(&value).expect("order id should exist");
    assert_eq!(order_id, "ETH-123");
}
