# Deployment Runbook: Paper Trading → Guarded Live

## 1) Environment matrix

| Environment | Purpose | Data Sources | Order Routing | Risk Mode |
|---|---|---|---|---|
| `dev` | Local development + debugging | Public endpoints / mocks | Disabled | Loose limits |
| `stage` | Integration and paper validation | Live market data | Paper only | Tight limits |
| `prod` | Guarded live trading | Live market data | Limited live routing | Strict limits + kill switch |

## 2) Required secrets and key rotation

### Required secrets

- `DERIBIT_CLIENT_ID`
- `DERIBIT_CLIENT_SECRET`
- `DERIVE_SESSION_KEY`
- `AEVO_API_KEY` (if private endpoints needed)
- `DATABASE_URL`
- `WEBHOOK_ALERT_URL`

### Rotation policy

- Rotate all exchange/API credentials at least every 90 days.
- Perform staged rotation:
  1. Add new secret in CI/CD and runtime manager.
  2. Deploy to `stage` and run smoke checks.
  3. Promote to `prod`.
  4. Revoke old secret.

## 3) Preflight checks before deployment

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test -q`
4. Live smoke checks: `./scripts/run_live_smoke.sh`
5. Storage integration workflow green
6. Risk limits config reviewed and signed off

## 4) Deployment steps

### Paper mode deployment

1. Deploy to `stage` with paper mode enabled.
2. Verify `/metrics` and dashboards are populated.
3. Run for at least one trading session.
4. Review PnL drift, fill rate, and risk alerts.

### Guarded live cutover

1. Enable live routing for one venue pair only.
2. Set conservative sizing and low max positions.
3. Confirm kill switch operation in rehearsal.
4. Enable alerting channels and on-call handoff.
5. Start canary window and observe continuously.

## 5) Rollback procedure

1. Trigger kill switch (flatten all positions).
2. Disable live routing flags.
3. Revert deployment to previous known-good release.
4. Verify no active live orders remain.
5. Announce rollback completion in incident channel.

## 6) Guarded-live checklist

- [ ] Risk limits loaded and validated
- [ ] Kill switch dry run executed successfully
- [ ] Alerting receivers acknowledged and online
- [ ] Metrics dashboard healthy
- [ ] DB retention and migration status checked
- [ ] On-call owner assigned for next 24h

## 7) Incident response quick guide

### Severity 1 (capital at risk / uncontrolled exposure)

1. Trigger kill switch immediately.
2. Pause all strategy scanners and executors.
3. Confirm position flattening and order cancellation.
4. Escalate to trading + engineering incident bridge.

### Severity 2 (degraded data / partial venue outage)

1. Disable affected venue connector.
2. Keep unaffected venue pairs in paper mode only.
3. Monitor recovery and resume gradually.

### Post-incident

- Collect timeline and impacted components.
- Open remediation issues.
- Update this runbook with lessons learned.
