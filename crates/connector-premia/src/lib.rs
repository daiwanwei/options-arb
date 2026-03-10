use anyhow::{anyhow, Result};
use common::types::{Greeks, Instrument, OptionStyle, OptionType, Ticker, VenueId};

pub const PREMIA_QUOTES_WS: &str = "wss://quotes.premia.finance";
const PREMIA_SUBGRAPH: &str = "https://api.thegraph.com/subgraphs/name/premian-labs/premia-blue";
const PREMIA_ORACLE: Option<&str> = None;

#[derive(Debug, Clone)]
pub struct PremiaQuoteRequest {
    pub chain: String,
    pub pool: String,
    pub size: f64,
    pub is_buy: bool,
}

pub fn premia_subgraph_url() -> &'static str {
    PREMIA_SUBGRAPH
}

pub fn premia_oracle_address() -> Option<&'static str> {
    PREMIA_ORACLE.filter(|value| is_valid_evm_address(value))
}

pub fn is_valid_evm_address(value: &str) -> bool {
    if value.len() != 42 || !value.starts_with("0x") {
        return false;
    }
    value[2..].chars().all(|item| item.is_ascii_hexdigit())
}

pub fn build_quote_request(chain: &str, pool: &str, size: f64, is_buy: bool) -> PremiaQuoteRequest {
    PremiaQuoteRequest {
        chain: chain.to_string(),
        pool: pool.to_string(),
        size,
        is_buy,
    }
}

pub fn normalize_quote_to_ticker(
    symbol: &str,
    bid: f64,
    ask: f64,
    iv: f64,
    timestamp_ms: i64,
) -> Result<Ticker> {
    let instrument = parse_instrument(symbol)?;
    let mid = (bid + ask) / 2.0;

    Ok(Ticker {
        instrument,
        venue: VenueId::Premia,
        bid: Some(bid),
        ask: Some(ask),
        mid: Some(mid),
        mark_price: Some(mid),
        index_price: None,
        iv: Some(iv),
        bid_iv: Some(iv),
        ask_iv: Some(iv),
        greeks: Greeks::default(),
        timestamp_ms,
    })
}

fn parse_instrument(symbol: &str) -> Result<Instrument> {
    let base = Instrument::from_clob_symbol(VenueId::Premia, symbol)
        .ok_or_else(|| anyhow!("invalid premia symbol: {symbol}"))?;

    Ok(Instrument {
        underlying: base.underlying,
        strike: base.strike,
        expiry: base.expiry,
        option_type: match base.option_type {
            OptionType::Call => OptionType::Call,
            OptionType::Put => OptionType::Put,
        },
        style: OptionStyle::European,
        venue: VenueId::Premia,
        venue_symbol: symbol.to_string(),
    })
}
