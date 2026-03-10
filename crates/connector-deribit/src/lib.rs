use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use common::types::{DeribitTicker, Greeks, OrderBook, OrderBookLevel, Ticker, VenueId};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub const DERIBIT_MAINNET_WS: &str = "wss://www.deribit.com/ws/api/v2";
pub const DERIBIT_TESTNET_WS: &str = "wss://test.deribit.com/ws/api/v2";

pub struct DeribitWsClient {
    url: &'static str,
}

impl DeribitWsClient {
    pub async fn connect() -> Result<Self> {
        let client = Self {
            url: DERIBIT_MAINNET_WS,
        };
        client.connect_once().await?;
        Ok(client)
    }

    pub fn new(url: &'static str) -> Self {
        Self { url }
    }

    pub fn url(&self) -> &'static str {
        self.url
    }

    pub async fn subscribe_ticker(&self, instrument: &str) -> Result<impl tokio_stream::Stream<Item = Ticker>> {
        let channel = format!("ticker.{instrument}.100ms");
        let ws = establish_subscription(self.url, &[channel]).await?;
        let (tx, rx) = mpsc::channel(1024);

        tokio::spawn(async move {
            let (_write, mut read) = ws.split();
            while let Some(incoming) = read.next().await {
                let Ok(message) = incoming else {
                    break;
                };
                let Ok(text) = message.into_text() else {
                    continue;
                };
                if let Ok(Some(ticker)) = parse_ticker_notification(&text) {
                    if tx.send(ticker).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }

    pub async fn subscribe_orderbook(
        &self,
        instrument: &str,
    ) -> Result<impl tokio_stream::Stream<Item = OrderBook>> {
        let channel = format!("book.{instrument}.100ms");
        let ws = establish_subscription(self.url, &[channel]).await?;
        let (tx, rx) = mpsc::channel(1024);
        let instrument_name = instrument.to_string();

        tokio::spawn(async move {
            let (_write, mut read) = ws.split();
            let mut local_book = LocalOrderBook::new();
            while let Some(incoming) = read.next().await {
                let Ok(message) = incoming else {
                    break;
                };
                let Ok(text) = message.into_text() else {
                    continue;
                };
                if let Ok(Some(orderbook)) =
                    parse_orderbook_notification(&text, &instrument_name, &mut local_book)
                {
                    if tx.send(orderbook).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }

    pub async fn reconnect_with_backoff(&self) -> Result<()> {
        for attempt in 0..=5 {
            if self.connect_once().await.is_ok() {
                return Ok(());
            }
            sleep(Duration::from_millis(backoff_delay_ms(attempt))).await;
        }
        Err(anyhow!("failed to reconnect after retries"))
    }

    async fn connect_once(&self) -> Result<()> {
        let (mut ws, _) = connect_async(self.url).await?;
        ws.close(None).await?;
        Ok(())
    }
}

async fn establish_subscription(
    url: &str,
    channels: &[String],
) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>
{
    let (mut ws, _) = connect_async(url).await?;
    let request = build_subscribe_request(1, channels);
    ws.send(Message::Text(request.to_string().into())).await?;
    Ok(ws)
}

pub fn build_subscribe_request(id: u64, channels: &[String]) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "public/subscribe",
        "params": { "channels": channels },
    })
}

pub fn parse_ticker_notification(text: &str) -> Result<Option<Ticker>> {
    let value: Value = serde_json::from_str(text)?;
    if value["method"].as_str() != Some("subscription") {
        return Ok(None);
    }

    let channel = value["params"]["channel"].as_str().unwrap_or_default();
    if !channel.starts_with("ticker.") {
        return Ok(None);
    }

    let data = value["params"]["data"].clone();
    let raw: RawDeribitTicker = serde_json::from_value(data)?;
    Ok(Some(to_unified_ticker(raw)?))
}

pub fn parse_orderbook_notification(
    text: &str,
    instrument_name: &str,
    local_book: &mut LocalOrderBook,
) -> Result<Option<OrderBook>> {
    let value: Value = serde_json::from_str(text)?;
    if value["method"].as_str() != Some("subscription") {
        return Ok(None);
    }
    let channel = value["params"]["channel"].as_str().unwrap_or_default();
    if !channel.starts_with("book.") {
        return Ok(None);
    }

    let data = &value["params"]["data"];
    let sequence = data["change_id"].as_u64().unwrap_or(0);
    let bids = parse_orderbook_levels(&data["bids"]);
    let asks = parse_orderbook_levels(&data["asks"]);
    let is_snapshot = data["type"].as_str() == Some("snapshot") || local_book.last_sequence.is_none();
    if is_snapshot {
        local_book.apply_snapshot(sequence, bids, asks)?;
    } else {
        local_book.apply_delta(sequence, bids, asks)?;
    }

    let instrument = common::types::Instrument::from_clob_symbol(VenueId::Deribit, instrument_name)
        .ok_or_else(|| anyhow!("invalid deribit instrument: {instrument_name}"))?;

    let orderbook = OrderBook {
        instrument,
        venue: VenueId::Deribit,
        bids: local_book
            .bids
            .iter()
            .rev()
            .take(25)
            .map(|(price, size)| OrderBookLevel {
                price: *price as f64 / 10_000.0,
                size: *size,
                iv: None,
            })
            .collect(),
        asks: local_book
            .asks
            .iter()
            .take(25)
            .map(|(price, size)| OrderBookLevel {
                price: *price as f64 / 10_000.0,
                size: *size,
                iv: None,
            })
            .collect(),
        timestamp_ms: data["timestamp"].as_i64().unwrap_or_default(),
    };

    Ok(Some(orderbook))
}

fn parse_orderbook_levels(value: &Value) -> Vec<(f64, f64)> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            if let Some(items) = entry.as_array() {
                if items.len() >= 3 {
                    let price = items[1].as_f64()?;
                    let size = items[2].as_f64()?;
                    return Some((price, size));
                }
                if items.len() == 2 {
                    let price = items[0].as_f64()?;
                    let size = items[1].as_f64()?;
                    return Some((price, size));
                }
            }
            None
        })
        .collect()
}

pub fn channel_names(instrument: &str) -> Vec<String> {
    vec![
        format!("book.{instrument}.100ms"),
        format!("ticker.{instrument}.100ms"),
        format!("trades.{instrument}.raw"),
    ]
}

pub fn backoff_delay_ms(attempt: u32) -> u64 {
    let value = 500_u64.saturating_mul(2_u64.saturating_pow(attempt));
    value.min(30_000)
}

#[derive(Debug, Clone)]
pub struct LocalOrderBook {
    pub last_sequence: Option<u64>,
    pub bids: BTreeMap<i64, f64>,
    pub asks: BTreeMap<i64, f64>,
}

impl LocalOrderBook {
    pub fn new() -> Self {
        Self {
            last_sequence: None,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn apply_snapshot(
        &mut self,
        sequence: u64,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    ) -> Result<()> {
        self.last_sequence = Some(sequence);
        self.bids.clear();
        self.asks.clear();
        update_levels(&mut self.bids, bids);
        update_levels(&mut self.asks, asks);
        Ok(())
    }

    pub fn apply_delta(
        &mut self,
        sequence: u64,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    ) -> Result<()> {
        let last = self
            .last_sequence
            .ok_or_else(|| anyhow!("snapshot required before delta"))?;

        if sequence <= last {
            return Err(anyhow!(
                "stale sequence received: got {sequence}, expected > {last}"
            ));
        }

        self.last_sequence = Some(sequence);
        update_levels(&mut self.bids, bids);
        update_levels(&mut self.asks, asks);
        Ok(())
    }
}

impl Default for LocalOrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawDeribitTicker {
    pub instrument_name: String,
    pub best_bid_price: Option<f64>,
    pub best_ask_price: Option<f64>,
    pub mark_price: Option<f64>,
    pub index_price: Option<f64>,
    pub mark_iv: Option<f64>,
    pub bid_iv: Option<f64>,
    pub ask_iv: Option<f64>,
    pub timestamp: i64,
}

pub fn to_unified_ticker(raw: RawDeribitTicker) -> Result<Ticker> {
    let ticker = DeribitTicker {
        instrument_name: raw.instrument_name,
        best_bid_price: raw.best_bid_price,
        best_ask_price: raw.best_ask_price,
        mark_price: raw.mark_price,
        index_price: raw.index_price,
        iv: raw.mark_iv,
        bid_iv: raw.bid_iv,
        ask_iv: raw.ask_iv,
        greeks: Greeks::default(),
        timestamp_ms: raw.timestamp,
    };

    Ticker::try_from(ticker).map_err(|err| anyhow!(err))
}

fn update_levels(side: &mut BTreeMap<i64, f64>, updates: Vec<(f64, f64)>) {
    for (price, size) in updates {
        let key = (price * 10000.0).round() as i64;
        if size <= 0.0 {
            side.remove(&key);
        } else {
            side.insert(key, size);
        }
    }
}
