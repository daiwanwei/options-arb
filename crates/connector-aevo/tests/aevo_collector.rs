use connector_aevo::{
    build_markets_url, compute_orderbook_checksum, orderbook_channel, trades_channel,
    AevoLocalOrderBook,
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
