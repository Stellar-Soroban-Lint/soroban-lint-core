# Changelog

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

All notable changes to `soroban-lint` are recorded here. Versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-10-09

First release whose `soroban-lint-wasm` package contains the real linter.

### Fixed

- The `v0.1.0` tag predated the WebAssembly implementation, so its
  `soroban-lint-wasm-v0.1.0.tar.gz` asset was a stub exporting only `version()`.
  `v0.1.1` is cut from the commit that exposes `lintSource` and `rulesJson`.

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
