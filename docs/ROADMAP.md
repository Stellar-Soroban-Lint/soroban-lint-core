# Roadmap

Work that is deliberately **not** filed as an issue yet, and **not scheduled**. Each item is real and held back until its precondition is met; the precondition, not a date, is what is being waited on. Nothing here is a commitment of date or scope — the issue tracker is the contract, this file is the direction.

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

## Held items

### Promote SL003–SL007 from experimental to stable

**Precondition:** a second, larger benchmark corpus with the same per-finding triage as `docs/BENCHMARKS.md` shows no unexplained false positives. The current corpus (`stellar/soroban-examples` @ `03d42aa6`) is too small and toy-heavy to carry the promotion — several findings are arguable rather than clearly true, which is exactly why promotion was deferred in the first place. `SPEC.md` §4 defines the bar: no unexplained FPs on the pinned corpus, or the rule is demoted.

### `--fix` suggestion output for SL002 (`unwrap` → `panic_with_error!`, `.expect` → typed error)

**Precondition:** a decision to take on `visit_mut`/`Fold` in the rule infrastructure. v1 deliberately uses read-only `syn::visit::Visit` (see `docs/ARCHITECTURE.md`); auto-rewriting changes that contract, adds an edit-application engine, and needs its own safety story (overlapping fixes, macro boundaries). The `fix:` field in `Diagnostic` is already the human-readable suggestion channel this would emit into.

### VS Code extension reusing the WASM crate

**Precondition:** the extension's packaging and update story. The heavy lifting exists — `soroban-lint-wasm` is the same binary the portal runs, and its exports mirror the CLI's JSON contract exactly. What does not exist yet is the extension host glue (diagnostic collection, file watching, settings → config mapping) and a release pipeline for the `.vsix`.

### SARIF `fixes` and `help.markdown` in the CLI's SARIF output

**Precondition:** nothing blocking; held because SARIF consumers tolerate the current shape and each addition must round-trip through the schema validator in `soroban-lint-core/tests/sarif_schema.rs` without breaking existing uploaders (`github/codeql-action/upload-sarif` is the reference consumer). Pairs naturally with the human-readable `fix:` strings already on every diagnostic.

### Extend mutation testing (`cargo-mutants`) from SL001/SL002 to SL003–SL008

**Precondition:** nothing blocking; held because SL001/SL002 were mutation-tested first as the security-critical rules, and the triage discipline (every survivor killed by a fixture, never waived — see `docs/BENCHMARKS.md`) does not scale to six more rules in one pass. Natural sequence: one rule per PR, fixtures for each survivor, the triage table extended in `docs/BENCHMARKS.md`.

## How items leave this file

An item moves from here to a filed issue when its precondition is met or when a contributor wants to own it and the precondition turns out to be lighter than we feared. Opening an issue that references this file is enough; do not wait for permission.
