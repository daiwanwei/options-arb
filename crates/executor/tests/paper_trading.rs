use executor::{
    build_grafana_dashboard_template, check_risk_alert, PaperFill, PaperTrader, PrometheusSnapshot,
};

#[test]
fn tracks_pnl_and_hit_rate() {
    let mut trader = PaperTrader::default();
    trader.record_fill(PaperFill::new("s1", 100.0, 110.0, 1.0));
    trader.record_fill(PaperFill::new("s2", 100.0, 95.0, 1.0));

    assert!((trader.total_pnl() - 5.0).abs() < 1e-9);
    assert!((trader.hit_rate() - 0.5).abs() < 1e-9);
}

#[test]
fn emits_prometheus_snapshot() {
    let mut trader = PaperTrader::default();
    trader.record_signal_latency_ms(42.0);
    trader.record_fill(PaperFill::new("s1", 100.0, 101.0, 1.0));

    let snap: PrometheusSnapshot = trader.metrics_snapshot(60.0);
    assert!(snap.signals_per_min >= 0.0);
    assert!(snap.fill_rate >= 0.0);
}

#[test]
fn risk_alerts_when_utilization_high() {
    assert!(check_risk_alert(0.91, 0.90).is_some());
    assert!(check_risk_alert(0.50, 0.90).is_none());
}

#[test]
fn provides_grafana_template_json() {
    let json = build_grafana_dashboard_template();
    assert!(json.contains("signals_per_min"));
    assert!(json.contains("fill_rate"));
    assert!(json.contains("pnl"));
}

#[test]
fn renders_prometheus_metrics_text() {
    let snapshot = PrometheusSnapshot {
        signals_per_min: 1.2,
        fill_rate: 0.5,
        pnl: 42.0,
        avg_latency_ms: 10.0,
    };

    let text = executor::render_prometheus(&snapshot, 0.75);
    assert!(text.contains("options_arb_signals_per_min"));
    assert!(text.contains("options_arb_fill_rate"));
    assert!(text.contains("options_arb_pnl"));
    assert!(text.contains("options_arb_risk_utilization"));
}
