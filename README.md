# soroban-lint-core

Syntactic, per-file static analysis for Soroban smart contracts, with a CLI and a
WASM build. The engine behind [`soroban-lint-action`](https://github.com/Stellar-Soroban-Lint/soroban-lint-action)
and the [playground](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal).

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

## Status

Pre-release (`0.1.0`). The engine, CLI, and rule set are implemented and tested.
Not yet published to crates.io. See `docs/BENCHMARKS.md` for real-corpus results.

## Rules

| ID | Rule | Default severity | Stability |
|---|---|---|---|
| SL001 | missing `require_auth` | error | stable |
| SL002 | panic hazards | warning | stable |
| SL003 | unchecked arithmetic | warning | experimental |
| SL004 | unbounded storage growth | warning | experimental |
| SL005 | missing TTL extension | info | experimental |
| SL006 | questionable storage type | warning | experimental |
| SL007 | unprotected initializer | warning | experimental |
| SL008 | `unsafe` / missing `#![no_std]` | warning | stable |

`experimental` rules are off unless `--experimental` or `experimental = true` in config.

## Install / build

```bash
cargo build --release -p soroban-lint-cli      # binary: target/release/soroban-lint
cargo build --target wasm32-unknown-unknown -p soroban-lint-wasm --release
```

## Usage

```bash
soroban-lint check <path> [--format text|json|sarif] [--config <file>]
                          [--fail-on error|warning|info|never] [--experimental]
soroban-lint rules [--format json|text]
```

Exit codes: `0` clean, `1` findings at or above `--fail-on`, `2` usage/internal error.
Parse failures are reported as `SL000` diagnostics, never as panics.

## Configuration

```toml
experimental = false
include = ["src/**/*.rs"]
exclude = ["target/**"]

[rules]
SL006 = "off"
[rules.SL001]
severity = "warning"
```

## Suppression

```rust
env.storage().persistent().set(&k, &v); // soroban-lint-ignore: SL005
// soroban-lint-ignore-next-line: SL001, SL002
let v = env.storage().instance().get(&k).unwrap();
```

A directive applies only to its exact target line(s). Unknown ids produce an `SL000` warning.

## Verification

```bash
cargo test --workspace                                # 36 tests
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo build --target wasm32-unknown-unknown -p soroban-lint-wasm --release
```

## Docs

- `SPEC.md` — interfaces, CLI contract, per-rule algorithms.
- `docs/ARCHITECTURE.md` — data flow and supported SDK range.
- `docs/WRITING_RULES.md` — how to add a rule.
- `docs/BENCHMARKS.md` — real-corpus results and triage.

## Related repositories

- [`soroban-lint-action`](https://github.com/Stellar-Soroban-Lint/soroban-lint-action) — GitHub Action.
- [`soroban-lint-portal`](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal) — browser playground running this crate as WebAssembly.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`SECURITY.md`](SECURITY.md).
