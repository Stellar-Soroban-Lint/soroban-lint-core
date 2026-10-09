<p align="center">
  <img src="https://raw.githubusercontent.com/Stellar-Soroban-Lint/soroban-lint-core/main/docs/assets/banner.svg" alt="soroban-lint-core — syntactic static analysis for Soroban contracts" width="100%">
</p>

<p align="center">
  <a href="https://github.com/Stellar-Soroban-Lint/soroban-lint-core/actions/workflows/ci.yml"><img src="https://github.com/Stellar-Soroban-Lint/soroban-lint-core/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="MIT OR Apache-2.0">
  <a href="https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/latest"><img src="https://img.shields.io/github/v/release/Stellar-Soroban-Lint/soroban-lint-core" alt="Latest release"></a>
  <a href="https://stellar-soroban-lint.github.io/soroban-lint-core/"><img src="https://img.shields.io/badge/docs-online-7C3AED" alt="Documentation"></a>
  <a href="https://discord.gg/xZRZT6TpB"><img src="https://img.shields.io/badge/Discord-join-5865F2?logo=discord&logoColor=white" alt="Discord"></a>
  <a href="https://t.me/+MrTh9uraIS5jMjhk"><img src="https://img.shields.io/badge/Telegram-join-26A5E4?logo=telegram&logoColor=white" alt="Telegram"></a>
</p>

`soroban-lint-core` is a Rust library and CLI that reviews Soroban contract source for risky code patterns. soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

[Docs](https://stellar-soroban-lint.github.io/soroban-lint-core/) · [Playground](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal) · [GitHub Action](https://github.com/Stellar-Soroban-Lint/soroban-lint-action) · [Demo PR #1 (closed): annotations](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/pull/1) · [Demo PR #2 (merged): passing run](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/pull/2) · [Issues](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/issues)

## What it does

The CLI parses Rust source and reports rule findings as text, JSON, or SARIF. The rule list below is generated from `soroban-lint rules --format json`.

| ID | Name | Default severity | Stability |
|---|---|---|---|
| SL001 | missing-require-auth | error | stable |
| SL002 | panic-hazards | warning | stable |
| SL003 | unchecked-arithmetic | warning | experimental |
| SL004 | unbounded-storage-growth | warning | experimental |
| SL005 | missing-ttl-extension | info | experimental |
| SL006 | questionable-storage-type | warning | experimental |
| SL007 | unprotected-initializer | warning | experimental |
| SL008 | unsafe-and-no-std | warning | stable |

SL003–SL007 are experimental and off by default. Enable them with `--experimental` or `experimental = true` in the config.

The pinned `stellar/soroban-examples` corpus produced 103 findings across 129 Rust files at commit [`03d42aa6b973dcf3a453a99d0c6a6e8d25a196e2`](https://github.com/stellar/soroban-examples/commit/03d42aa6b973dcf3a453a99d0c6a6e8d25a196e2). See [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) for the rule breakdown and triage.

## Quick start

Download the Linux x86_64 v0.1.2 release and verify the published SHA-256 before extraction:

```bash
gh release download v0.1.2 --repo Stellar-Soroban-Lint/soroban-lint-core \
  --pattern 'soroban-lint-v0.1.2-x86_64-unknown-linux-gnu.tar.gz*'
sha256sum -c soroban-lint-v0.1.2-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -xzf soroban-lint-v0.1.2-x86_64-unknown-linux-gnu.tar.gz
./soroban-lint-v0.1.2-x86_64-unknown-linux-gnu/soroban-lint check ./contracts
```

The verified archive SHA-256 is `c2b2bcf5f01d2591b44a411b7c6724b4cc0c00671a8062e6a82deb26cea89538`. See the [release page](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/tag/v0.1.2) for other platforms and checksums.

Write SARIF to a file by redirecting the command output:

```bash
soroban-lint check ./contracts --format sarif --fail-on never > soroban-lint.sarif
```

A project config can enable the stable rules and select source paths:

```toml
experimental = false
include = ["src/**/*.rs"]
exclude = ["target/**"]

[rules]
SL006 = "off"
[rules.SL001]
severity = "warning"
```

Suppress one finding with `// soroban-lint-ignore: SL005`, or suppress findings on the next line with `// soroban-lint-ignore-next-line: SL001, SL002`. Exit code `0` means no findings at or above `--fail-on`; `1` means at least one such finding; `2` means usage or internal error. The docs cover [CLI options](https://stellar-soroban-lint.github.io/soroban-lint-core/cli) and [configuration and suppressions](https://stellar-soroban-lint.github.io/soroban-lint-core/cli#configuration).

## Architecture

The data flow is `source → syn AST → rule visitors → Diagnostics → text | JSON | SARIF | WASM`. The CLI reads Rust source and parses each file into a `syn` AST; rule visitors produce Diagnostics that the CLI formats or the core compiles to WASM for the browser. The analysis does not expand macros, resolve types, or perform cross-file analysis. Those limits can produce false positives and false negatives, so findings need review. See the [architecture guide](https://stellar-soroban-lint.github.io/soroban-lint-core/architecture/) for details.

## The ecosystem

| Repository | Role |
|---|---|
| [soroban-lint-core](https://github.com/Stellar-Soroban-Lint/soroban-lint-core) | Rust analysis engine, CLI, and WASM build. |
| [soroban-lint-action](https://github.com/Stellar-Soroban-Lint/soroban-lint-action) | Runs the CLI in GitHub Actions. |
| [soroban-lint-portal](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal) | Browser playground source. |
| [PR #1 (closed): annotations on intentionally vulnerable contracts](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/pull/1) | Findings and check annotations; kept for reference. |
| [PR #2 (merged): passing run](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/pull/2) | `fail-on: never`, five inline annotations, and passing checks. |

## Maintainers

| Name | GitHub | Telegram |
|---|---|---|
| ojuotimi932 | [@ojuotimi932](https://github.com/ojuotimi932) | [Telegram](https://t.me/+MrTh9uraIS5jMjhk) |

## Community
- Telegram: https://t.me/+MrTh9uraIS5jMjhk
- Discord: https://discord.gg/xZRZT6TpB

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md), then choose an issue from the [good first issue list](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22).
Run `cargo test --workspace` and `cargo fmt --all --check` before opening a PR.
To add a rule, follow the docs for [Writing Rules](https://stellar-soroban-lint.github.io/soroban-lint-core/writing-rules/).

## Contributors

[![Contributors](https://contrib.rocks/image?repo=Stellar-Soroban-Lint/soroban-lint-core)](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/graphs/contributors)

## Security

See [SECURITY.md](SECURITY.md) to report a vulnerability. soroban-lint aids code review; it is not an audit and does not prove a contract is secure.

## License

Licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
