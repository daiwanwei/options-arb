use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VenueId {
    Deribit,
    Derive,
    Aevo,
    Premia,
    Stryke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionStyle {
    European,
    American,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instrument {
    pub underlying: String,
    pub strike: f64,
    pub expiry: String,
    pub option_type: OptionType,
    pub style: OptionStyle,
    pub venue: VenueId,
    pub venue_symbol: String,
}

impl Instrument {
    pub fn from_clob_symbol(venue: VenueId, symbol: &str) -> Option<Self> {
        let mut parts = symbol.split('-');
        let underlying = parts.next()?.trim().to_uppercase();
        let expiry = parts.next()?.trim().to_uppercase();
        let strike = parts.next()?.trim().parse::<f64>().ok()?;
        let option_type = match parts.next()?.trim().to_uppercase().as_str() {
            "C" => OptionType::Call,
            "P" => OptionType::Put,
            _ => return None,
        };

        Some(Self {
            underlying,
            strike,
            expiry,
            option_type,
            style: OptionStyle::European,
            venue,
            venue_symbol: symbol.to_string(),
        })
    }
}

pub fn match_instrument(a: &Instrument, b: &Instrument) -> bool {
    a.underlying == b.underlying
        && a.expiry == b.expiry
        && (a.strike - b.strike).abs() < f64::EPSILON
        && a.option_type == b.option_type
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Greeks {
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub theta: Option<f64>,
    pub vega: Option<f64>,
    pub rho: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticker {
    pub instrument: Instrument,
    pub venue: VenueId,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub mid: Option<f64>,
    pub mark_price: Option<f64>,
    pub index_price: Option<f64>,
    pub iv: Option<f64>,
    pub bid_iv: Option<f64>,
    pub ask_iv: Option<f64>,
    pub greeks: Greeks,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: f64,
    pub size: f64,
    pub iv: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBook {
    pub instrument: Instrument,
    pub venue: VenueId,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    pub instrument: Instrument,
    pub venue: VenueId,
    pub price: f64,
    pub size: f64,
    pub side: TradeSide,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone)]
pub struct DeribitTicker {
    pub instrument_name: String,
    pub best_bid_price: Option<f64>,
    pub best_ask_price: Option<f64>,
    pub mark_price: Option<f64>,
    pub index_price: Option<f64>,
    pub iv: Option<f64>,
    pub bid_iv: Option<f64>,
    pub ask_iv: Option<f64>,
    pub greeks: Greeks,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone)]
pub struct DeriveTicker {
    pub instrument_name: String,
    pub best_bid_price: Option<f64>,
    pub best_ask_price: Option<f64>,
    pub mark_price: Option<f64>,
    pub index_price: Option<f64>,
    pub option_iv: Option<f64>,
    pub bid_iv: Option<f64>,
    pub ask_iv: Option<f64>,
    pub greeks: Greeks,
    pub timestamp_ms: i64,
}

impl TryFrom<DeribitTicker> for Ticker {
    type Error = &'static str;

    fn try_from(value: DeribitTicker) -> Result<Self, Self::Error> {
        let instrument = Instrument::from_clob_symbol(VenueId::Deribit, &value.instrument_name)
            .ok_or("invalid deribit instrument")?;
        let mid = match (value.best_bid_price, value.best_ask_price) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        };

        Ok(Self {
            instrument,
            venue: VenueId::Deribit,
            bid: value.best_bid_price,
            ask: value.best_ask_price,
            mid,
            mark_price: value.mark_price,
            index_price: value.index_price,
            iv: value.iv,
            bid_iv: value.bid_iv,
            ask_iv: value.ask_iv,
            greeks: value.greeks,
            timestamp_ms: value.timestamp_ms,
        })
    }
}

impl TryFrom<DeriveTicker> for Ticker {
    type Error = &'static str;

    fn try_from(value: DeriveTicker) -> Result<Self, Self::Error> {
        let instrument = Instrument::from_clob_symbol(VenueId::Derive, &value.instrument_name)
            .ok_or("invalid derive instrument")?;
        let mid = match (value.best_bid_price, value.best_ask_price) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        };

        Ok(Self {
            instrument,
            venue: VenueId::Derive,
            bid: value.best_bid_price,
            ask: value.best_ask_price,
            mid,
            mark_price: value.mark_price,
            index_price: value.index_price,
            iv: value.option_iv,
            bid_iv: value.bid_iv,
            ask_iv: value.ask_iv,
            greeks: value.greeks,
            timestamp_ms: value.timestamp_ms,
        })
    }
}
