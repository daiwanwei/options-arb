use common::types::{
    Greeks, Instrument, OptionStyle, OptionType, Ticker, Trade, TradeSide, VenueId,
};
use storage::{SqlStorage, StorageConfig};

#[tokio::test]
async fn migrates_and_inserts_records_when_db_available() {
    let db_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return,
    };

    let storage = SqlStorage::connect(&db_url).await.expect("db connect");
    storage
        .migrate(&StorageConfig::default())
        .await
        .expect("migrate");

    let instrument = Instrument {
        underlying: "ETH".to_string(),
        strike: 3000.0,
        expiry: "28MAR26".to_string(),
        option_type: OptionType::Call,
        style: OptionStyle::European,
        venue: VenueId::Deribit,
        venue_symbol: "ETH-28MAR26-3000-C".to_string(),
    };

    let ticker = Ticker {
        instrument: instrument.clone(),
        venue: VenueId::Deribit,
        bid: Some(220.0),
        ask: Some(230.0),
        mid: Some(225.0),
        mark_price: Some(225.0),
        index_price: Some(2950.0),
        iv: Some(0.6),
        bid_iv: Some(0.59),
        ask_iv: Some(0.61),
        greeks: Greeks::default(),
        timestamp_ms: 1,
    };

    let trade = Trade {
        instrument,
        venue: VenueId::Deribit,
        price: 225.0,
        size: 1.0,
        side: TradeSide::Buy,
        timestamp_ms: 1,
    };

    storage.insert_ticker(&ticker).await.expect("insert ticker");
    storage.insert_trade(&trade).await.expect("insert trade");
}
