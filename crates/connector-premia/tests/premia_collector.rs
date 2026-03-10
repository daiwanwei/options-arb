use common::types::VenueId;
use connector_premia::{
    build_quote_request, is_valid_evm_address, normalize_quote_to_ticker, premia_oracle_address,
    premia_subgraph_url, PREMIA_QUOTES_WS,
};

#[test]
fn exposes_required_endpoints() {
    assert_eq!(PREMIA_QUOTES_WS, "wss://quotes.premia.finance");
    assert!(premia_subgraph_url().contains("premia-blue"));
    assert!(premia_oracle_address().is_none());
    assert!(is_valid_evm_address(
        "0x1111111111111111111111111111111111111111"
    ));
}

#[test]
fn builds_pool_quote_request() {
    let req = build_quote_request("arb-one", "0xpool", 1.5, true);
    assert_eq!(req.chain, "arb-one");
    assert_eq!(req.pool, "0xpool");
}

#[test]
fn normalizes_quote_to_common_ticker() {
    let ticker = normalize_quote_to_ticker("ETH-28MAR26-3000-C", 225.0, 226.0, 0.62, 2)
        .expect("normalization should work");

    assert_eq!(ticker.venue, VenueId::Premia);
    assert_eq!(ticker.bid, Some(225.0));
}
