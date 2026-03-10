# Prometheus Metrics

## Endpoint

- Path: `/metrics`
- Bind address: set `METRICS_BIND`, for example:

```bash
METRICS_BIND=0.0.0.0:9000 cargo run -p options-arb
```

## Metric naming conventions

- Prefix: `options_arb_`
- Gauges:
  - `options_arb_signals_per_min`
  - `options_arb_fill_rate`
  - `options_arb_pnl`
  - `options_arb_latency_ms`
  - `options_arb_risk_utilization`

## Prometheus scrape config example

```yaml
scrape_configs:
  - job_name: options-arb
    scrape_interval: 15s
    static_configs:
      - targets: ["localhost:9000"]
```

## Notes

- Keep labels low-cardinality for long-term storage efficiency.
- Pair metrics with alerting on risk utilization and PnL drawdown.
