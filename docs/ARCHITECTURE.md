# Architecture

## Scope statement (normative — reproduced verbatim across all artifacts)

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

## Data flow

```
source text
  → syn::parse_file            (comments discarded)
  → raw-source suppression scan (comments, mapped to line numbers)
  → rule visitors (syn::visit::Visit, read-only)
  → Vec<Diagnostic>            (sorted by file, line, col, rule_id; deduped)
  → text | JSON | SARIF        (CLI)
  → WASM (wasm-bindgen)        (portal, later phase)
```

## Workspace layout (the `soroban-lint-core` repository)

| Crate | Role | Dependencies |
|---|---|---|
| `soroban-lint-core` | Library. Parsing, rules, config, suppression, renderers. **No filesystem, no CLI, WASM-clean.** | `syn`, `proc-macro2`, `serde`, `serde_json`, `toml`, `thiserror`, `globset` |
| `soroban-lint-cli` | Binary `soroban-lint`. Argument parsing, file walking, I/O, exit codes. | `clap`, `walkdir`, `globset`, `anyhow` |
| `soroban-lint-wasm` | `wasm-bindgen` wrapper around the core library. | `wasm-bindgen`, `console_error_panic_hook` |

`globset` is in core only for pattern *matching* of already-listed paths; the CLI performs the
directory walk. Core never touches the filesystem.

## Analysis mechanism

- `syn` 2.x with features `full`, `visit`, `extra-traits`; spans via `proc-macro2` with `span-locations`.
- Operates on **source text**, not rustc HIR/MIR, not rust-analyzer, not macro expansion. Rationale:
  fast, dependency-light, and compilable to `wasm32-unknown-unknown`.
- Read-only traversal (`syn::visit::Visit`). `visit_mut`/`Fold` are unused (no `--fix` in v1).
- **Macro bodies are opaque token streams.** Rules inspect a macro's path/name and top-level text
  only; they never descend into tokens to reconstruct expressions.
- **No type resolution.** Type evidence is syntactic (see SL003 in `SPEC.md`).
- **No cross-file call following.** SL001 follows exactly one level into same-file, same-impl/helper
  functions. Nothing else follows calls.

## Supported Soroban SDK range

Fixtures target `soroban-sdk = "=28.0.0"` (released 2026-09-18). The supported, tested range for v1
is **27.0.x–28.0.0**. Fixture crates compile against the pinned SDK in a dedicated CI job so that
"vulnerable" and "safe" samples are provably valid Soroban code. `soroban-sdk` is **not** a
dependency of `soroban-lint-core`.

## WASM build (WASM-cleanliness gate)

Measured on this repository (toolchain 1.99.0, `release` profile, `panic = "abort"`, `opt-level = "s"`, `lto = true`):

| Artifact | Size |
|---|---:|
| `cargo build --target wasm32-unknown-unknown -p soroban-lint-wasm --release` | 38,203 bytes |
| `wasm-pack build --target web` → `pkg/soroban_lint_wasm_bg.wasm` | 20,462 bytes |
| after `wasm-opt -Oz` (binaryen version_123) | 14,306 bytes |

This build succeeding is the check that no filesystem, `clap`, or `walkdir` dependency has
leaked into `soroban-lint-core`.

## Determinism

Diagnostics are sorted by the total order `(file, start_line, start_column, rule_id)` and
de-duplicated on `(rule_id, file, start_line, start_column)`, so text, JSON, and SARIF output is
byte-stable across runs and platforms.

## Suppression

`syn` discards comments, so suppression is parsed from raw source and matched against diagnostic
start lines. See `SPEC.md` §2.2 for the grammar. Directives apply to their exact target line(s) and
never leak to other lines or other rules.

## Related documents

- `SPEC.md` — the full specification (interfaces, CLI contract, per-rule algorithms).
- `docs/WRITING_RULES.md` — how to add a rule, with a compile-checked template.
- `docs/BENCHMARKS.md` — real-corpus results and per-finding triage.
