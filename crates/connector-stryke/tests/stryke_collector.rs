use common::types::VenueId;
use connector_stryke::{
    market_address, normalize_premium_to_ticker, protocol_fee_multiplier, short_expiry_filter_hours,
};

#[test]
fn exposes_known_market_addresses() {
    assert!(market_address("WETH_USDC").is_some());
    assert!(market_address("WBTC_USDC").is_some());
}

#[test]
fn applies_protocol_fee_multiplier() {
    let gross: f64 = 100.0;
    let net = gross * (1.0 - protocol_fee_multiplier());
    assert!((net - 85.0).abs() < 1e-9);
}

#[test]
fn normalizes_short_expiry_premium() {
    let ticker = normalize_premium_to_ticker("ETH-10MAR26-3000-C", 120.0, 4, 1).expect("normalize");
    assert_eq!(ticker.venue, VenueId::Stryke);
    assert!(short_expiry_filter_hours(4));
}
