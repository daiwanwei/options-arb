# Required Checks for `main`

To protect `main`, configure branch protection with these required status checks:

- `checks / Cargo fmt`
- `checks / Cargo clippy`
- `checks / Cargo test`

## GitHub settings

1. Go to repository **Settings** → **Branches**.
2. Add (or edit) a branch protection rule for `main`.
3. Enable:
   - Require a pull request before merging
   - Require status checks to pass before merging
4. Select required checks from the `Rust CI` workflow:
   - Cargo fmt
   - Cargo clippy
   - Cargo test

## Notes

- CI runs on `push` and `pull_request` targeting `main`.
- Failing checks should block merges to keep `main` healthy.
