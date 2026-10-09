# Changelog

All notable changes to `soroban-lint` are recorded here. Versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-09

First release. Published to crates.io as `soroban-lint-core` and `soroban-lint-cli`,
with prebuilt binaries and a `wasm-pack` package on the
[releases page](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/tag/v0.1.0).

### Added

- `soroban-lint` CLI: `check` and `rules` subcommands, `text`, `json`, and SARIF 2.1.0 output.
- `soroban-lint-core`: a `syn`-based, per-file analysis engine with a versioned rule catalog.
- `soroban-lint-wasm`: WebAssembly bindings (`wasm-pack --target web`) for the browser playground.
- Suppression directives: `// soroban-lint-ignore: <ID>` and `// soroban-lint-ignore-next-line: <ID>`.
- Configuration via `soroban-lint.toml` (`include`, `exclude`, `experimental`, per-rule severity).

### Rules

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

SL003–SL007 are experimental and off unless `--experimental` or `experimental = true`.

### Known limitations

- No macro expansion, no type resolution, and no cross-file analysis.
- SL001 follows a same-file helper call one level deep; auth delegated further is missed.
- Parse failures are reported as `SL000`, never as a panic.
- A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.
