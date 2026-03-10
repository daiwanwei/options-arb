use connector_deribit::{backoff_delay_ms, channel_names, LocalOrderBook};

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
