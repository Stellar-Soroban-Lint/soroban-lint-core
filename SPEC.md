# soroban-lint — Specification (v1, DRAFT — awaiting approval)

Status: **DRAFT.** No implementation begins until this document is approved by the maintainer.

---

## 0. Scope statement (normative — reproduced verbatim across all artifacts)

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

This paragraph, word for word, must appear in `docs/ARCHITECTURE.md`, `SPEC.md`, all three READMEs, the GitHub Action Marketplace description, and the portal landing page. Phase 4 verifies it with pasted grep/diff output.

---

## 1. Analysis mechanism (settled)

- **Parser:** `syn` 2.x with features `full`, `visit`, `extra-traits`; spans via `proc-macro2` with `span-locations`.
- **Input:** Soroban contract **source text**. Not rustc HIR/MIR, not rust-analyzer, not macro expansion. Rationale: fast, dependency-light, compilable to `wasm32-unknown-unknown`.
- **Traversal:** read-only `syn::visit::Visit`. `visit_mut`/`Fold` are not used (no `--fix` in v1).
- **Macro bodies are opaque token streams.** `panic_with_error!`, `vec![]`, `symbol_short!`, `require_auth_for_args!(...)` and every other invocation parse as `syn::Macro`. Each rule below states its macro treatment. In general, a rule inspects the macro's *path/name* and top-level token text only; it never descends into macro tokens to reconstruct expressions.
- **No type resolution.** Type evidence is syntactic only (see SL003).
- **No cross-file call following.** SL001 follows exactly one level into same-file, same-`impl`/same-module helper functions (see SL001).

### 1.1 Supported Soroban SDK range

Fixtures target `soroban-sdk = "=28.0.0"` (released 2026-09-18, newest stable at time of writing). The supported, tested range for v1 is **27.0.x–28.0.0**. Fixture crates compile against the pinned SDK in a dedicated CI job so "vulnerable" and "safe" samples are provably valid Soroban code. `soroban-sdk` is **not** a dependency of `soroban-lint-core`.

---

## 2. Core interfaces (`soroban-lint-core`)

```rust
/// Severity ordering is Error > Warning > Info.
pub enum Severity { Error, Warning, Info }

/// How much syntactic evidence backs a finding. Independent of severity.
pub enum Confidence { High, Medium, Low }

/// Stability governs default enablement. `Stable` => on by default.
pub enum Stability { Stable, Experimental }

pub struct Diagnostic {
    pub rule_id: &'static str,      // "SL001" .. "SL008", or "SL000" (meta)
    pub severity: Severity,
    pub confidence: Confidence,
    pub message: String,            // one line, no trailing period
    pub file: String,               // path RELATIVE to the invocation root, forward slashes
    pub start_line: usize,          // 1-based, inclusive
    pub start_column: usize,        // 1-based, inclusive
    pub end_line: usize,            // 1-based, inclusive
    pub end_column: usize,          // 1-based, exclusive
    pub help: Option<String>,       // remediation text
    pub fix: Option<String>,        // human-readable suggestion text; NOT a machine edit
}

pub struct RuleMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub default_severity: Severity,
    pub default_confidence: Confidence,
    pub stability: Stability,
    pub rationale: &'static str,    // why the pattern matters
    pub limitations: &'static str,  // FP/FN modes, in the same paragraph as the claim
}

pub trait Rule: Send + Sync {
    fn meta(&self) -> RuleMeta;
    /// Push zero or more diagnostics. MUST NOT panic on any input.
    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>);
}

pub struct Context<'a> {
    pub file: &'a str,                    // relative path
    pub source: &'a str,                  // raw text (comments intact)
    pub ast: &'a syn::File,               // parsed, comments stripped
    pub config: &'a Config,
    pub suppressions: &'a Suppressions,   // parsed from `source`
}

pub struct Registry { /* Vec<Box<dyn Rule>> */ }
impl Registry {
    pub fn default_set() -> Self;                      // all rules compiled in
    pub fn register(&mut self, rule: Box<dyn Rule>);
    pub fn get(&self, id: &str) -> Option<&dyn Rule>;
    pub fn metadata(&self) -> Vec<RuleMeta>;           // deterministic, sorted by id
    /// Runs enabled rules, applies suppressions, sorts, returns diagnostics.
    pub fn lint(&self, ctx: &Context<'_>) -> Vec<Diagnostic>;
}
```

**Deterministic ordering.** After suppression, diagnostics are sorted by `(file, start_line, start_column, rule_id)` — a total order — so text/JSON/SARIF output is byte-stable across runs and platforms. Duplicate `(rule_id, file, start_line, start_column)` diagnostics are de-duplicated.

**Panic-freedom.** Rule code, span conversion, and parsing must never panic. Panics surface as `SL000` diagnostics or as a returned `Error`. A `console_error_panic_hook` is compiled into the WASM build only (feature-gated).

### 2.1 Configuration (`soroban-lint.toml`)

Parsed by `Config::from_toml_str` (core owns the schema; the CLI reads the file so core stays filesystem-free).

```toml
# Whether `experimental` rules run. Default false.
experimental = false

# Optional include/exclude globs (globset syntax). Applied by the CLI's walker.
include = ["src/**/*.rs", "contracts/**/*.rs"]
exclude = ["target/**", "tests/**"]

[rules]
# Shorthand: "off" | "error" | "warning" | "info"
SL006 = "off"

# Table form for explicit control
[rules.SL001]
enabled = true
severity = "error"
```

Semantics:

- A rule with `Stability::Stable` is **enabled by default**; `Stability::Experimental` is **disabled unless** `experimental = true` (config) or `--experimental` (CLI).
- `severity = "off"` (or `enabled = false`) disables a rule regardless of stability.
- `severity` overrides `RuleMeta::default_severity` for that rule.
- Unknown rule IDs in `[rules]` produce an `SL000` error at the config file.
- Unknown top-level keys produce an `SL000` warning (typo guard).

```rust
pub struct Config {
    pub experimental: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub rules: std::collections::BTreeMap<String, RuleSettings>,
}
pub struct RuleSettings { pub enabled: Option<bool>, pub severity: Option<Severity> }

impl Config {
    pub fn from_toml_str(s: &str) -> Result<Config, ConfigError>;
    pub fn is_enabled(&self, meta: &RuleMeta) -> bool;
    pub fn effective_severity(&self, meta: &RuleMeta) -> Option<Severity>; // None => disabled
}
```

### 2.2 Suppression grammar

`syn` discards comments, so suppression is a **raw-source line scan** mapped onto 1-based line numbers, evaluated against diagnostic `start_line`.

Two directives, both line comments beginning with `//` (leading whitespace allowed):

| Directive | Effect |
|---|---|
| `// soroban-lint-ignore: SL001` | Suppresses findings whose `start_line` equals the comment's line. |
| `// soroban-lint-ignore-next-line: SL001, SL002` | Suppresses findings whose `start_line` equals `comment_line + 1`. |

Rules:

1. Directive keywords are lowercase and exact; IDs are uppercase and exact.
2. IDs are comma-separated, surrounding whitespace ignored. At least one ID is required.
3. A directive applies **only** to its exact target line(s). It never leaks to other lines. `ignore-next-line` targets exactly `N+1`, even if that line is blank or the finding is further away — no "scan forward".
4. No wildcard (`all`, `*`) in v1; a suppression must name each rule.
5. An **unknown rule ID** in a suppression produces an `SL000` warning at the comment line (catches typos such as `SL01`). The directive is otherwise ignored.
6. Suppressed findings are removed from output and do **not** affect the exit code.

```rust
pub struct Suppressions { /* line -> set<&str> */ }
impl Suppressions {
    pub fn parse(source: &str) -> Suppressions;
    pub fn is_suppressed(&self, rule_id: &str, line: usize) -> bool;
    pub fn unknown_ids(&self, known: &dyn Fn(&str) -> bool) -> Vec<(usize, String)>;
}
```

Test matrix (Phase 1): same-line hit; next-line hit; two IDs on one directive; unknown ID warning; a directive for `SL001` on a line where only `SL002` fires (must not suppress); blank line between `ignore-next-line` and finding (must not suppress).

---

## 3. CLI contract (`soroban-lint-cli`)

```
soroban-lint check <path> [--format text|json|sarif] [--config <file>]
                          [--fail-on error|warning|info|never] [--experimental]
soroban-lint rules [--format json|text]
```

- `<path>` is a file or directory. Directories are walked with `walkdir`, honoring `include`/`exclude`; `.gitignore` honored only if `ignore` is added (deferred; documented).
- `--config` defaults to `soroban-lint.toml` in the invocation root if present.
- `--experimental` forces experimental rules on, overriding config.
- `--fail-on` defaults to `warning`; `never` means the tool never exits non-zero on findings.

**Exit codes**

| Code | Meaning |
|---|---|
| 0 | Clean (no findings at or above `--fail-on`). |
| 1 | At least one finding at or above `--fail-on`. |
| 2 | Usage error or internal error (bad flag, unreadable/unparseable config, I/O failure). |

**Parse failures** are **not** panics and **not** exit 2. A file that fails to parse yields an `SL000` diagnostic (severity `Error`, confidence `High`, message includes the parse error) and analysis continues with remaining files. Such a diagnostic counts toward `--fail-on`, so a default run exits 1.

**JSON** (`--format json`):

```json
{
  "version": "1",
  "diagnostics": [
    {
      "rule_id": "SL001", "severity": "error", "confidence": "medium",
      "message": "...", "file": "contracts/lib.rs",
      "start_line": 42, "start_column": 5, "end_line": 42, "end_column": 38,
      "help": "...", "fix": null
    }
  ]
}
```

**`soroban-lint rules --format json`** emits registry metadata (used by the portal to generate the rule catalog at build time):

```json
{ "version": "1", "rules": [ { "id": "SL001", "name": "...", "description": "...",
  "default_severity": "error", "default_confidence": "medium",
  "stability": "stable", "rationale": "...", "limitations": "..." } ] }
```

**SARIF 2.1.0** (`--format sarif`): `tool.driver.name = "soroban-lint"`, `informationUri`, `version`, and `rules[]` mirroring registry metadata. Each result: `ruleId`, `level` (`error|warning|note`), `message.text`, `locations[0].physicalLocation.artifactLocation.uri` as a **relative** POSIX path, `region.{startLine,startColumn,endLine,endColumn}`, and `partialFingerprints` (`sorobanLint/v1` = hash of `rule_id + file + start_line`). Output is validated against the vendored `schemas/sarif-2.1.0.json` in tests.

---

## 4. Rules

Eight rules ship in v1. `SL000` is reserved for meta/usage/parse/config diagnostics and is never reused. **Rule IDs are permanent; retired IDs stay reserved.**

**Initial stability.** `stable` (default-on): SL001, SL002, SL008. `experimental` (opt-in): SL003–SL007. A rule is promoted to `stable` only when the Phase 1 benchmark shows **no unexplained false positives** across the pinned corpus; a `stable` rule with an unexplained FP is demoted. Promotion/demotion is recorded in `docs/BENCHMARKS.md`.

Common scoping terms used below:

- **contract function** = a `pub fn` inside an `impl` block carrying `#[contractimpl]`. Non-`pub` fns, other impls, free fns, `#[cfg(test)]` modules, and `#[contracttype]`/`#[contracterror]` derives are out of scope unless stated.
- **storage mutation call** = a method call whose name is `set`, `remove`, or `update`, or `extend_ttl` (see SL001 exclusions), reached through any receiver chain containing `.storage()`, `.persistent()`, `.temporary()`, or `.instance()`.
- **auth call** = a call to `require_auth` or `require_auth_for_args`, or a `require_auth!`/`require_auth_for_args!` macro invocation.

### SL000 — meta (reserved)

Produces diagnostics for: file parse failures, unreadable/invalid config, unknown rule IDs, unknown config keys, and unknown suppression IDs. Not documented as a "security rule".

### SL001 — Missing `require_auth`

- **Default:** severity `Error`, confidence `Medium`, stability `Stable`.
- **Flag:** a contract function that contains ≥1 **storage mutation call** or a **token operation** (method named `transfer`, `transfer_from`, `mint`, or `burn`) and in which no **auth call** is reachable within one level of same-file, same-`impl`/same-module helper calls.
- **Pseudocode:**
  ```
  for impl in contractimpl_impls(ast):
    helpers = { fn_name -> FnBody } for non-pub fns in impl and same-file free fns
    for f in impl.pub_fns:
      if not has_mutation(f.body): continue
      if has_auth(f.body): continue
      if any called helper h in helpers has_auth(h.body): continue   // one level only
      emit(fn span, "no authorization check before state mutation")
  ```
- **Explicitly not flagged:** read-only fns (only `.get`/`.has`); `extend_ttl` calls (not privileged — callable without auth); any fn containing a direct auth call; `admin.require_auth()` where `admin` is loaded from storage (it *is* an auth call — correct pattern); non-contract fns.
- **Macros:** `require_auth!`/`require_auth_for_args!` count as auth by macro name. Mutation calls written *inside* macros are not seen (documented FN).
- **Known FP:** a mutation guarded by a custom auth helper in a different file (not reached). **Known FN:** two-or-more levels of helper indirection; mutations performed inside macro bodies.
- **Fix:** insert `addr.require_auth()` (or `require_auth_for_args`) before the first mutation, addressing the actored `Address`.

### SL002 — Panic hazards

- **Default:** severity `Warning`, confidence `High` (explicit panics) / `Medium` (indexing, slicing), stability `Stable`.
- **Flag:** in a contract function — `panic!`, `.unwrap()`, `.expect(..)`, `unreachable!`, `todo!`, `unimplemented!`, and `Expr::Index` / slicing expressions.
- **Explicitly not flagged:** `panic_with_error!` (recommended pattern), `Result` propagation via `?`, `.unwrap_or`, `.unwrap_or_default`, `.unwrap_or_else`, indexing in `#[cfg(test)]` code, const contexts.
- **Macros:** `panic_with_error!` and other macro invocations are treated as safe; their tokens are not descended into.
- **Known FP:** indexing proven in-bounds by surrounding code (hence `Medium`). **Known FN:** panics raised by called functions in other files.
- **Fix:** return `Result<_, ContractError>` where `ContractError` is `#[contracterror]`, or use `panic_with_error!`; replace `.expect` with explicit error handling.

### SL003 — Unchecked arithmetic

- **Default:** severity `Warning`, confidence `Low`, stability `Experimental`.
- **Flag (syntactic type evidence required):** a binary op `+ - * / %` or compound assign where **both** operands carry integer evidence from syntax alone. Evidence = integer literal suffix (`10u64`), explicit `as` cast to an integer type, an explicit integer type annotation on the binding (`let x: u128 = ..`, fn param `amount: i128`), or a `const`/`static` with an integer type. Division/modulo by a literal `0` is always flagged.
- **Explicitly not flagged:** hardware-independent operations `wrapping_*`, `checked_*`, `saturating_*`; operands with **no** syntactic integer evidence (this is most storage-loaded values — see limitations); floating point.
- **Project check:** when the input is a directory or `Cargo.toml` is discoverable, emit one `SL003` diagnostic on `Cargo.toml` (severity `Warning`) if the resolved release profile lacks `overflow-checks = true`. File = `"Cargo.toml"`.
- **Macros:** arithmetic inside macros is not seen.
- **Stated limitation (same paragraph as the claim):** because there is no type resolution, a value loaded from storage and added to another has no syntactic integer evidence and is **not** flagged; this is the dominant false-negative mode for SL003.
- **Fix:** use `checked_add`/`saturating_*`, or set `overflow-checks = true` in `[profile.release]`.

### SL004 — Unbounded storage growth

- **Default:** severity `Warning`, confidence `Low`, stability `Experimental`.
- **Flag:** in a contract function, a value **loaded from storage** (`.get(..)`) is grown (`push`, `insert`, `append`, `extend`) and written back (`.set(..)`), with no visible length bound: no `if`/`match` guard comparing `.len()` to a constant/limit, no early return on capacity, and no constant cap in the fn.
- **Explicitly not flagged:** growth on locals not loaded from storage; growth inside a loop whose bound is a constant.
- **Known FP:** bounds enforced in caller/helper or by config. **Known FN:** bounds via macros or external call.
- **Fix:** enforce an explicit maximum length before growing, and return an error when exceeded.

### SL005 — Missing TTL extension

- **Default:** severity `Info`, confidence `Medium`, stability `Experimental`.
- **Flag:** a contract function performs a **persistent** or **instance** storage `.set`/`.remove`/`.update` with no `extend_ttl` call anywhere in the fn.
- **Explicitly not flagged:** `.temporary()` writes; read-only fns; fns already containing `extend_ttl`; Soroban `#[constructor]`/`__constructor`.
- **Known FP:** TTL extended by a helper or a different entrypoint. **Known FN:** indirect extension.
- **Fix:** call `env.storage().persistent().extend_ttl(&key, threshold, extend_to)` after the write.

### SL006 — Questionable storage type

- **Default:** severity `Warning`, confidence `Low`, stability `Experimental`.
- **Flag:** a `.temporary().set(key, value)` where the key/value identifier name matches balance/ownership heuristics (`balance`, `owner`, `allowance`, `admin`, `supply`, `total`) — temporary storage is inappropriate for data expected to persist.
- **Explicitly not flagged:** temporary storage for genuinely ephemeral data (nonces, per-tx caches) whose identifiers do not match the heuristics.
- **Macros:** not analyzed.
- **Known FP:** high (naming heuristic) — hence `Low` confidence and experimental. **Known FN:** mis-typed data under innocuous names.
- **Fix:** use persistent storage for balance/ownership data.

### SL007 — Unprotected initializer

- **Default:** severity `Warning`, confidence `Medium`, stability `Experimental`.
- **Flag:** a contract function whose name is `init`, `initialize`, or `__init`, **or** carrying a `#[constructor]`-style role attribute, that writes an `initialized`/`admin`-like storage key, and that has **neither** an "already initialized" guard (a prior `.has()`/`.get()` check on an initialized flag with an early return/panic) **nor** an auth call.
- **Explicitly not flagged:** Soroban's native `__constructor` (runs once at deploy); fns without initialization writes.
- **Known FP:** guard expressed via macros. **Known FN:** guard delegated to a helper more than one level away.
- **Fix:** add an initialization guard (e.g. `if env.storage().instance().has(&INIT) { panic_with_error!(..) }`) or require auth.

### SL008 — `unsafe` / missing `#![no_std]`

- **Default:** severity `Warning`, confidence `High` (`unsafe`) / `Low` (no_std), stability `Stable`.
- **Flag:** (a) any `unsafe` block, `unsafe fn`, `unsafe impl`, or `unsafe trait` in a contract crate; (b) a file containing a `#[contract]` attribute but lacking `#![no_std]` at crate root.
- **Explicitly not flagged:** `unsafe` inside `#[cfg(test)]`.
- **Known FP (no_std):** valid `#![cfg_attr(not(test), no_std)]` patterns gated behind features — hence `Low` confidence for the no_std case only.
- **Fix:** remove `unsafe`; add `#![no_std]` to the contract crate root.

---

## 5. Test strategy (Phase 1)

Per rule: a true-positive fixture, a safe/negative fixture, ≥1 tricky negative (auth in a helper; `admin` loaded from storage), and a documented false-negative fixture proving the stated limitation is real. Fixtures live in `tests/fixtures/{vulnerable,safe}/`. `insta` snapshots cover text, JSON, and SARIF; CI runs `cargo insta test --check` (no pending snapshots). Suppression matrix per §2.2. Robustness: `proptest` (or `cargo fuzz`) over the parse-and-lint path; the linter must not panic on empty files, the corpus, or syntactically invalid Rust. `cargo-mutants` runs on SL001 and SL002 with every survivor triaged.

## 6. Out of scope for v1

Inter-procedural/cross-file analysis beyond SL001's one level, `--fix` auto-rewriting, VS Code extension, Docker action variant, reentrancy/cross-contract analysis, any claim of audit coverage or security guarantee. These are documented v2 items and seeded issues.

## 7. Open decisions for maintainer approval

1. **Default-on set** = {SL001, SL002, SL008}; SL003–SL007 experimental. Confirm.
2. **`SL000` reservation** for meta/parse/config/suppression diagnostics. Confirm.
3. **Exit code for parse failures:** `SL000` Error counts toward `--fail-on` (default exit 1), not exit 2. Confirm.
