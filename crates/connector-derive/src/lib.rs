use anyhow::{anyhow, Result};
use common::types::{DeriveTicker, Greeks, Ticker};
use serde_json::Value;

pub const DERIVE_PROD_WS: &str = "wss://api.lyra.finance/ws";
pub const DERIVE_TESTNET_WS: &str = "wss://api-demo.lyra.finance/ws";
pub const DERIVE_PROD_REST: &str = "https://api.lyra.finance";
pub const DERIVE_TESTNET_REST: &str = "https://api-demo.lyra.finance";

pub fn build_json_rpc_request(id: u64, method: &str, params: Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    })
}

pub fn build_get_all_instruments_request(id: u64, base_currency: &str) -> Value {
    build_json_rpc_request(
        id,
        "public/get_all_instruments",
        serde_json::json!({ "base_currency": base_currency }),
    )
}

pub fn build_get_ticker_request(id: u64, instrument_name: &str) -> Value {
    build_json_rpc_request(
        id,
        "public/get_ticker",
        serde_json::json!({ "instrument_name": instrument_name }),
    )
}

pub fn build_session_key_auth_request(id: u64, session_key: &str) -> Value {
    build_json_rpc_request(
        id,
        "private/login",
        serde_json::json!({
            "grant_type": "session_key",
            "session_key": session_key,
        }),
    )
}

pub fn parse_instrument_symbols(response: &Value) -> Vec<String> {
    response["result"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["instrument_name"].as_str().map(ToString::to_string))
        .collect()
}

pub fn reconnect_delay_ms(attempt: u32) -> u64 {
    let value = 500_u64.saturating_mul(2_u64.saturating_pow(attempt));
    value.min(30_000)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawDeriveTicker {
    pub instrument_name: String,
    pub best_bid_price: Option<f64>,
    pub best_ask_price: Option<f64>,
    pub mark_price: Option<f64>,
    pub index_price: Option<f64>,
    pub option_iv: Option<f64>,
    pub bid_iv: Option<f64>,
    pub ask_iv: Option<f64>,
    pub timestamp_ms: i64,
}

pub fn to_unified_ticker(raw: RawDeriveTicker) -> Result<Ticker> {
    let ticker = DeriveTicker {
        instrument_name: raw.instrument_name,
        best_bid_price: raw.best_bid_price,
        best_ask_price: raw.best_ask_price,
        mark_price: raw.mark_price,
        index_price: raw.index_price,
        option_iv: raw.option_iv,
        bid_iv: raw.bid_iv,
        ask_iv: raw.ask_iv,
        greeks: Greeks::default(),
        timestamp_ms: raw.timestamp_ms,
    };

    Ticker::try_from(ticker).map_err(|err| anyhow!(err))
}
