use common::live_smoke::{run_live_smoke, SmokeResult};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn skips_when_flag_not_enabled() {
    let _guard = ENV_LOCK.lock().expect("lock env");
    std::env::remove_var("RUN_LIVE_SMOKE");
    std::env::remove_var("LIVE_SMOKE_DRY_RUN");
    std::env::remove_var("ASSERT_LIVE_SMOKE");
    let results = run_live_smoke().expect("runner should return");
    assert_eq!(results.len(), 0);
}

#[test]
fn returns_structured_results_with_flag_enabled() {
    let _guard = ENV_LOCK.lock().expect("lock env");
    std::env::set_var("RUN_LIVE_SMOKE", "1");
    std::env::set_var("LIVE_SMOKE_DRY_RUN", "1");
    std::env::remove_var("ASSERT_LIVE_SMOKE");
    let results = run_live_smoke().expect("runner should return");
    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|item: &SmokeResult| !item.venue.is_empty()));
}

#[test]
fn fails_with_venue_and_phase_context_when_strict_enabled() {
    let _guard = ENV_LOCK.lock().expect("lock env");
    if std::env::var("RUN_LIVE_SMOKE").unwrap_or_default() != "1" {
        return;
    }
    if std::env::var("ASSERT_LIVE_SMOKE").unwrap_or_default() != "1" {
        return;
    }
    if std::env::var("LIVE_SMOKE_DRY_RUN").unwrap_or_default() == "1" {
        return;
    }

    let results = run_live_smoke().expect("runner should return results");
    let failures: Vec<&SmokeResult> = results.iter().filter(|item| !item.ok).collect();
    if failures.is_empty() {
        return;
    }

    let message = failures
        .iter()
        .map(|item| format!("{}:{}:{}", item.venue, item.phase, item.details))
        .collect::<Vec<String>>()
        .join(" | ");
    panic!("live smoke failures => {message}");
}
