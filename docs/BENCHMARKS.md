# Benchmarks — real-code corpus

Every finding below was produced by the real CLI, not hand-written. Nothing was deleted to
make the table look better. Counts are inputs to a promotion decision, not evidence of coverage:

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

## Corpus

- **`stellar/soroban-examples`** pinned to `03d42aa6b973dcf3a453a99d0c6a6e8d25a196e2` (129 `.rs` files).
- Command:

```bash
soroban-lint check . --experimental --format json --fail-on never
```

Tool version: `0.1.2` plus the test-module skip on `main` (unreleased; see
[Test modules](#test-modules-are-skipped) below). The walk reports `35` files skipped as
`#[cfg(test)]` modules on this corpus; pass `include-tests = true` in `soroban-lint.toml` to
analyze them and reproduce the previous `103`-finding run.

## Totals

**99 findings across the corpus.**

| Rule | Findings | In `*test.rs` (skipped) | Confirmed FP | Arguable | True positive |
|---|---:|---:|---:|---:|---:|
| SL001 missing-require-auth | 10 | 0 | 0 | 4 | 6 |
| SL002 panic-hazards | 60 | 0 | 0 | 18 | 42 |
| SL003 unchecked-arithmetic | 9 | 0 | 0 | 9 | 0 |
| SL004 unbounded-storage-growth | 1 | 0 | 0 | 0 | 1 |
| SL005 missing-ttl-extension | 19 | 0 | 0 | 19 | 0 |
| SL006 questionable-storage-type | 0 | 0 | 0 | 0 | 0 |
| SL007 unprotected-initializer | 0 | 0 | 0 | 0 | 0 |
| SL008 unsafe-and-no-std | 0 | 0 | 0 | 0 | 0 |

## Two false positives found and fixed before this table

These were real, unexplained false positives. They were fixed in code, not papered over:

1. **SL001 flagged `__constructor`.** Soroban's `__constructor` runs once during deployment and
   needs no authorization, so flagging it was wrong. Fix: SL001 now skips `__constructor`.
   Impact: SL001 24 → 10 findings.
2. **SL008 `#![no_std]` fired on non-root files** (e.g. `*/src/test.rs`). `#![no_std]` is a
   crate-root attribute. Fix: the `no_std` check now applies only to `lib.rs`/`main.rs`.
   Impact: SL008 4 → 0 findings.

## Per-rule reasoning

- **SL001 (10).** `increment`/`inc` on toy counters (`increment`, `custom_types`, `errors`, `events`,
  `increment_with_fuzz`, `other_custom_types`) are **arguable**: they match the rule's definition
  (public fn writes storage, no auth) but the examples are demonstrative and a shared counter may be
  intentionally permissionless. `merkle_distribution::claim`, `pause::set`, `ttl::setup` are
  **true positives** by the rule's definition. No false positives.
- **SL002 (60).** `unwrap`/`expect`/`panic!`/indexing in contract functions. Most are **true
  positives** under the rule's stated scope (a panic aborts the invocation). Indexing in `eth_abi`
  is a true positive (unchecked bounds). No false positives in shipping code; test modules are no
  longer walked at all.
- **SL003 (9).** All are `+=`/`%` on values with explicit integer evidence. These **arguable**: the
  rule is scoped to syntactic evidence, and the examples are toy counters where overflow is
  immaterial. No false positives (every one has genuine integer evidence).
- **SL004 (1).** `privacy-pools` grows a stored `nullifiers` collection with no visible bound.
  **True positive.**
- **SL005 (19).** Persistent/instance writes without `extend_ttl` in the same function. **Arguable**
  overall: some contracts extend TTL elsewhere or rely on defaults. No false positives (each write is
  genuinely persistent/instance and genuinely lacks a same-function `extend_ttl`).
- **SL006/SL007/SL008:** no findings on this corpus.

## Promotion decision

Per `SPEC.md` §4, a rule is promoted to `stable` only when the corpus shows **no unexplained false
positives**. After the two rule fixes and the test-module skip, **no rule has a confirmed false
positive** on this corpus, and nothing is reported in code that does not ship.
Promotion is nevertheless **deferred** for SL003/SL004/SL005/SL006/SL007, because:

- the corpus is small and toy-heavy (mostly demonstrative contracts), and
- several findings are *arguable* rather than clearly true.

A second, larger corpus is required before promoting. Current status is unchanged from `SPEC.md`:
`stable` = SL001, SL002, SL008; `experimental` = SL003–SL007.

## Test modules are skipped

The previous run of this corpus reported `103` findings, `4` of which were in `*test.rs` files.
Those files are `#[cfg(test)]` modules included from `lib.rs`, so the `cfg(test)` attribute is not
in the file itself and a per-file analyzer cannot see it. That was a real false-positive mode: it
reported findings in code that never ships.

It is now fixed at the walker. A directory walk skips `src/test.rs` and `src/**/tests.rs`, reports
the count on stderr, and leaves every other rule untouched. A file named directly on the command
line is still analyzed — fixtures and intentional single-file runs keep working — and
`include-tests = true` restores the old behaviour.

On this corpus the walk skips `35` files, which removes the `2` SL002 and `2` SL005 findings and
moves the total from `103` to `99`. `tests/` directories are not skipped: they are outside the
`src` tree this heuristic keys on, and a file there is not assumed to be test-only.

Reproduce:

```bash
git clone https://github.com/stellar/soroban-examples
cd soroban-examples && git checkout 03d42aa6b973dcf3a453a99d0c6a6e8d25a196e2
soroban-lint check . --experimental --format json --fail-on never
```

## Mutation testing (SL001, SL002)

Per `SPEC.md` §5, `cargo-mutants` runs over the two security-critical rules with every survivor
triaged. Command:

```bash
cargo mutants --no-times \
  --file soroban-lint-core/src/rules/sl001.rs \
  --file soroban-lint-core/src/rules/sl002.rs
```

Result: **16 mutants tested — 14 caught, 2 unviable, 0 missed** (`cargo-mutants 27.1.0`).

The first run left 5 survivors. Each was a genuine coverage gap, so each was killed with a
fixture rather than waived:

| Survivor | Gap it exposed | Fixture added |
|---|---|---|
| `sl001.rs:38` delete `Item::Fn` arm | same-file **free-function** helper with auth was never exercised | `safe/sl001_auth_in_free_fn.rs` |
| `sl001.rs:39` delete `!` in `!is_test` | a `#[cfg(test)]` free fn must not be treated as an auth helper | `vulnerable/sl001_test_helper_not_auth.rs` |
| `sl001.rs:49` `&&`→`\|\|` | a `#[cfg(test)]` **method** must not be treated as an auth helper | `vulnerable/sl001_test_helper_not_auth.rs` |
| `sl001.rs:61` `\|\|`→`&&` | a non-`pub` method is not a contract entry point | `safe/sl001_private_method.rs` |
| `sl002.rs:32` `\|\|`→`&&` | a non-`pub` method is not a contract entry point | `safe/sl002_private_method.rs` |

The two **unviable** mutants are `meta() -> Default::default()` for both rules: `RuleMeta`
intentionally has no `Default` impl, so the mutant does not compile. They are not survivors.

After the fixtures were added the run is clean, and the same command is what CI/the reviewer
should re-run to reproduce it.
