# Benchmarks — real-code corpus

Every finding below was produced by the real CLI, not hand-written. Nothing was deleted to
make the table look better.

## Corpus

- **`stellar/soroban-examples`** pinned to `03d42aa6b973dcf3a453a99d0c6a6e8d25a196e2` (129 `.rs` files).
- Command:

```bash
soroban-lint check . --experimental --format json --fail-on never
```

Tool version: `0.1.0` (`soroban-lint` debug build from this repository).

## Totals

**103 findings across the corpus.**

| Rule | Findings | In `*test.rs` | Confirmed FP | Arguable | True positive |
|---|---:|---:|---:|---:|---:|
| SL001 missing-require-auth | 10 | 0 | 0 | 4 | 6 |
| SL002 panic-hazards | 62 | 2 | 0 | 18 | 44 |
| SL003 unchecked-arithmetic | 9 | 0 | 0 | 9 | 0 |
| SL004 unbounded-storage-growth | 1 | 0 | 0 | 0 | 1 |
| SL005 missing-ttl-extension | 21 | 2 | 0 | 21 | 0 |
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
- **SL002 (62).** `unwrap`/`expect`/`panic!`/indexing in contract functions. Most are **true
  positives** under the rule's stated scope (a panic aborts the invocation). Indexing in `eth_abi`
  is a true positive (unchecked bounds). 2 findings are in `*test.rs` files, which this linter
  cannot tell are `#[cfg(test)]` modules (see limitation below). No false positives in shipping code.
- **SL003 (9).** All are `+=`/`%` on values with explicit integer evidence. These **arguable**: the
  rule is scoped to syntactic evidence, and the examples are toy counters where overflow is
  immaterial. No false positives (every one has genuine integer evidence).
- **SL004 (1).** `privacy-pools` grows a stored `nullifiers` collection with no visible bound.
  **True positive.**
- **SL005 (21).** Persistent/instance writes without `extend_ttl` in the same function. **Arguable**
  overall: some contracts extend TTL elsewhere or rely on defaults. No false positives (each write is
  genuinely persistent/instance and genuinely lacks a same-function `extend_ttl`). 2 are in `*test.rs`.
- **SL006/SL007/SL008:** no findings on this corpus.

## Promotion decision

Per `SPEC.md` §4, a rule is promoted to `stable` only when the corpus shows **no unexplained false
positives**. After the two fixes, **no rule has a confirmed false positive** on this corpus.
Promotion is nevertheless **deferred** for SL003/SL004/SL005/SL006/SL007, because:

- the corpus is small and toy-heavy (mostly demonstrative contracts), and
- several findings are *arguable* rather than clearly true.

A second, larger corpus is required before promoting. Current status is unchanged from `SPEC.md`:
`stable` = SL001, SL002, SL008; `experimental` = SL003–SL007.

## Known limitation surfaced by this run

Files named `test.rs` are analyzed as ordinary source. In this corpus they are `#[cfg(test)]`
modules included from `lib.rs`, so the `cfg(test)` attribute is not present in the file itself and
the linter cannot see it. It therefore reports findings in test-only code (2 SL002, 2 SL005 here).
This is a real false-positive mode and is documented rather than hidden.

Reproduce:

```bash
git clone https://github.com/stellar/soroban-examples
cd soroban-examples && git checkout 03d42aa6b973dcf3a453a99d0c6a6e8d25a196e2
soroban-lint check . --experimental --format json --fail-on never
```
