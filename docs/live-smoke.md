# Live Smoke Tests

## Single command

```bash
./scripts/run_live_smoke.sh
```

This command runs live endpoint smoke checks for:

- Deribit
- Derive
- Aevo
- Premia
- Stryke

## Runtime policy

- Timeout: `8s` per attempt
- Retry: `3` attempts per venue
- Strict mode: enabled by `ASSERT_LIVE_SMOKE=1`

When strict mode is enabled, failures include explicit `venue:phase:details` context.

## CI behavior

- Manual dispatch (`workflow_dispatch`): blocking
- Nightly schedule: non-blocking (`continue-on-error`)
- Artifacts: `artifacts/live-smoke-summary.txt` and test output logs
