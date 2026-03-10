# Rust Options Math Crates

## Recommended Stack

```toml
[dependencies]
optionstratlib = "0.15"    # Core: BS, Greeks, American options, 25+ strategies
implied-vol = "2.0"        # Production IV solver (Jackel's "Let's Be Rational")
volsurf = "1.0"            # Vol surface calibration (SVI, SABR, SSVI, eSSVI)
```

## Crate Comparison

| Crate | Install | Downloads | Last Updated | Maintained? | Best For |
|-------|---------|-----------|-------------|-------------|----------|
| **OptionStratLib** | `cargo add optionstratlib` | 31K | Feb 2026 | Active | Full-featured options toolkit |
| **implied-vol** | `cargo add implied-vol` | 92K | Aug 2025 | Yes | Production IV solver (Jackel) |
| **volsurf** | `cargo add volsurf` | New | Feb 2026 | Active | Vol surface (SVI, SABR, SSVI) |
| **black_scholes** | `cargo add black_scholes` | 120K | Jan 2026 | Yes | Lightweight BS + Greeks |
| **RustQuant** | `cargo add RustQuant` | 101K (1.7k stars) | Nov 2024 | Slowing | QuantLib-lite, research |
| **quantrs** | `cargo add quantrs` | 6K | Feb 2026 | Active | Lightweight, 29x faster than QuantLib-py |
| **blackscholes** | `cargo add blackscholes` | 36K | Sep 2023 | Stale | Fast (~35ns/price), WASM bindings |

## Feature Matrix

| Feature | OptionStratLib | implied-vol | volsurf | black_scholes | RustQuant |
|---------|---------------|-------------|---------|---------------|-----------|
| BS Pricing | Yes | No | Yes | Yes | Yes |
| Greeks (all orders) | Yes (Vanna, Vomma, Charm, Color) | No | No | 1st order | Via AAD |
| IV Solver | Newton-Raphson | Jackel (gold standard) | Jackel rational | Yes | NR |
| American Options | Yes (BAW + Binomial) | No | No | No | No |
| Vol Surface | Yes | No | SVI/SABR/SSVI/eSSVI, arb detection | No | No |
| Exotic Options | 14 types | No | No | No | No |
| Monte Carlo | Yes | No | No | No | Yes |
| Strategy Builder | 25+ templates | No | No | No | No |

## Notes

- OptionStratLib uses `rust_decimal` for precision (important for arb)
- OptionStratLib is by joaquinbejar (same author as deribit-websocket Rust crate)
- volsurf has sub-20ns vol queries, zero-alloc design, butterfly + calendar arb detection
- implied-vol includes C++ code (Jackel's reference impl) — check license for commercial use
- No maintained QuantLib Rust bindings exist
