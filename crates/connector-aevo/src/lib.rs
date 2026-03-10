use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub const AEVO_REST_BASE: &str = "https://api.aevo.xyz";
pub const AEVO_WS_BASE: &str = "wss://ws.aevo.xyz";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct AevoAuthConfig {
    pub api_key: String,
    pub api_secret: String,
}

pub struct AevoWsClient {
    url: &'static str,
}

impl AevoWsClient {
    pub fn new(url: &'static str) -> Self {
        Self { url }
    }

    pub async fn subscribe_orderbook(
        &self,
        instrument: &str,
    ) -> Result<impl tokio_stream::Stream<Item = AevoLocalOrderBook>> {
        let (mut ws, _) = connect_async(self.url).await?;
        let subscribe = build_subscribe_message(&[orderbook_channel(instrument)]);
        ws.send(Message::Text(subscribe.to_string().into())).await?;

        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(async move {
            let (_write, mut read) = ws.split();
            let mut local_book = AevoLocalOrderBook::new();
            while let Some(incoming) = read.next().await {
                let Ok(message) = incoming else {
                    break;
                };
                let Ok(text) = message.into_text() else {
                    continue;
                };

                if let Ok(Some(updated)) = parse_orderbook_message(&text, &mut local_book) {
                    if tx.send(updated).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }
}

pub struct AevoRestClient {
    http: reqwest::Client,
    base_url: String,
    auth: AevoAuthConfig,
}

impl AevoRestClient {
    pub fn new(base_url: &str, api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            auth: AevoAuthConfig {
                api_key: api_key.into(),
                api_secret: api_secret.into(),
            },
        }
    }

    pub async fn place_order(
        &self,
        instrument: &str,
        is_buy: bool,
        limit_price: &str,
        quantity: &str,
        timestamp_ms: i64,
    ) -> Result<String> {
        let path = "/orders";
        let url = format!("{}{}", self.base_url, path);
        let body = serde_json::json!({
            "instrument": instrument,
            "is_buy": is_buy,
            "limit_price": limit_price,
            "quantity": quantity,
        });
        let body_text = body.to_string();

        let signature = sign_aevo_request(
            &self.auth.api_secret,
            timestamp_ms,
            "POST",
            path,
            &body_text,
        )?;

        let response = self
            .http
            .post(url)
            .header("AEVO-KEY", &self.auth.api_key)
            .header("AEVO-TIMESTAMP", timestamp_ms)
            .header("AEVO-SIGNATURE", signature)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let value = response.json::<Value>().await?;
        extract_aevo_order_id(&value)
    }
}

pub fn build_subscribe_message(channels: &[String]) -> Value {
    serde_json::json!({
        "op": "subscribe",
        "data": channels,
    })
}

pub fn sign_aevo_request(
    api_secret: &str,
    timestamp_ms: i64,
    method: &str,
    path: &str,
    body: &str,
) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes())
        .map_err(|_| anyhow!("invalid hmac key"))?;
    let payload = format!("{timestamp_ms}{}{}{}", method.to_uppercase(), path, body);
    mac.update(payload.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn extract_aevo_order_id(value: &Value) -> Result<String> {
    value["order_id"]
        .as_str()
        .or_else(|| value["result"]["order_id"].as_str())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("missing order_id in aevo response"))
}

pub fn parse_orderbook_message(
    text: &str,
    local_book: &mut AevoLocalOrderBook,
) -> Result<Option<AevoLocalOrderBook>> {
    let value: Value = serde_json::from_str(text)?;
    if value["type"].as_str() != Some("orderbook") {
        return Ok(None);
    }

    let bids = parse_side(&value["data"]["bids"]);
    let asks = parse_side(&value["data"]["asks"]);
    let checksum = value["data"]["checksum"]
        .as_u64()
        .unwrap_or_else(|| {
            let mut preview = local_book.clone();
            preview.apply_snapshot(bids.clone(), asks.clone());
            compute_orderbook_checksum(&preview)
        });

    if value["data"]["snapshot"].as_bool().unwrap_or(false) || local_book.bids.is_empty() {
        local_book.apply_snapshot(bids, asks);
        return Ok(Some(local_book.clone()));
    }

    local_book.apply_delta(bids, asks, checksum)?;
    Ok(Some(local_book.clone()))
}

fn parse_side(value: &Value) -> Vec<(f64, f64)> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let items = entry.as_array()?;
            if items.len() < 2 {
                return None;
            }
            Some((items[0].as_f64()?, items[1].as_f64()?))
        })
        .collect()
}

pub fn build_markets_url(asset: &str, instrument_type: &str) -> String {
    format!("{AEVO_REST_BASE}/markets?asset={asset}&instrument_type={instrument_type}")
}

pub fn orderbook_channel(instrument: &str) -> String {
    format!("orderbook-100ms:{instrument}")
}

pub fn trades_channel(asset: &str) -> String {
    format!("trades:{asset}")
}

#[derive(Debug, Clone)]
pub struct AevoLocalOrderBook {
    pub bids: BTreeMap<i64, f64>,
    pub asks: BTreeMap<i64, f64>,
}

impl AevoLocalOrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn apply_snapshot(&mut self, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) {
        self.bids.clear();
        self.asks.clear();
        update_side(&mut self.bids, bids);
        update_side(&mut self.asks, asks);
    }

    pub fn apply_delta(
        &mut self,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        expected_checksum: u64,
    ) -> Result<()> {
        update_side(&mut self.bids, bids);
        update_side(&mut self.asks, asks);

        let current = compute_orderbook_checksum(self);
        if current != expected_checksum {
            return Err(anyhow!(
                "checksum mismatch: expected {expected_checksum}, got {current}"
            ));
        }

        Ok(())
    }
}

impl Default for AevoLocalOrderBook {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compute_orderbook_checksum(book: &AevoLocalOrderBook) -> u64 {
    let mut value = 0_u64;

    for (price, size) in book.bids.iter().take(5) {
        value = value.wrapping_add(*price as u64).wrapping_add(*size as u64);
    }
    for (price, size) in book.asks.iter().take(5) {
        value = value.wrapping_add(*price as u64).wrapping_add(*size as u64);
    }

    value
}

fn update_side(side: &mut BTreeMap<i64, f64>, levels: Vec<(f64, f64)>) {
    for (price, size) in levels {
        let key = (price * 10000.0).round() as i64;
        if size <= 0.0 {
            side.remove(&key);
        } else {
            side.insert(key, size);
        }
    }
}
