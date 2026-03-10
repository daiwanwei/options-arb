use storage::{migrations_sql, retention_policy_sql, StorageConfig};

#[test]
fn migration_contains_required_tables() {
    let sql = migrations_sql();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS tickers"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS orderbook_snapshots"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS trades"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS arb_signals"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS positions"));
}

#[test]
fn migration_includes_hypertable_setup() {
    let sql = migrations_sql();
    assert!(sql.contains("create_hypertable('tickers'"));
    assert!(sql.contains("create_hypertable('trades'"));
}

#[test]
fn retention_policy_is_configurable() {
    let config = StorageConfig {
        raw_ticker_retention_days: 30,
    };
    let sql = retention_policy_sql(&config);
    assert!(sql.contains("INTERVAL '30 days'"));
}
