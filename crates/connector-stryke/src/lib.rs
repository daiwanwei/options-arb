use anyhow::{anyhow, Result};
use common::types::{Greeks, Ticker, VenueId};

pub const WETH_USDC_MARKET: &str = "0x25360000000000000000000000000000000002bb";
pub const WBTC_USDC_MARKET: &str = "0xaBa500000000000000000000000000000000002e";

pub fn market_address(symbol: &str) -> Option<&'static str> {
    match symbol {
        "WETH_USDC" => Some(WETH_USDC_MARKET),
        "WBTC_USDC" => Some(WBTC_USDC_MARKET),
        _ => None,
    }
}

pub fn protocol_fee_multiplier() -> f64 {
    0.15
}

pub fn short_expiry_filter_hours(hours_to_expiry: i64) -> bool {
    (0..=24).contains(&hours_to_expiry)
}

pub fn normalize_premium_to_ticker(
    symbol: &str,
    premium: f64,
    hours_to_expiry: i64,
    timestamp_ms: i64,
) -> Result<Ticker> {
    if !short_expiry_filter_hours(hours_to_expiry) {
        return Err(anyhow!("expiry outside 0DTE/short window"));
    }

    let instrument = common::types::Instrument::from_clob_symbol(VenueId::Stryke, symbol)
        .ok_or_else(|| anyhow!("invalid stryke symbol"))?;

    let net = premium * (1.0 - protocol_fee_multiplier());
    Ok(Ticker {
        instrument,
        venue: VenueId::Stryke,
        bid: Some(net),
        ask: Some(premium),
        mid: Some((premium + net) / 2.0),
        mark_price: Some(premium),
        index_price: None,
        iv: None,
        bid_iv: None,
        ask_iv: None,
        greeks: Greeks::default(),
        timestamp_ms,
    })
}
