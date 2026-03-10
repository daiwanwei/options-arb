use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use common::types::{DeribitTicker, Greeks, Ticker};

pub const DERIBIT_MAINNET_WS: &str = "wss://www.deribit.com/ws/api/v2";
pub const DERIBIT_TESTNET_WS: &str = "wss://test.deribit.com/ws/api/v2";

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
