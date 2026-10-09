# Writing a rule

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

Rules are cheap to add and easy to get wrong. A rule that "works" but has no
true-positive fixture is worse than no rule. Follow every step.

## 1. Reserve an id

Ids are **permanent**. `SL001`–`SL008` are taken (`SL000` is reserved for meta
diagnostics). Never reuse or renumber an id; retired ids stay reserved. Add the
next free number.

## 2. Copy the template

A compile-checked template lives in the crate at
`src/rules/mod.rs`, in the `#[cfg(test)] mod template` module. It is compiled by
`cargo test`, so it cannot rot. Copy its shape into a new `src/rules/slNNN.rs`:

```rust
use super::{for_each_contract_fn, push_diag};
use crate::analysis::{analyze, is_pub, is_test, loc};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

pub struct ExampleRule;

impl Rule for ExampleRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL900",
            name: "example-rule",
            description: "Example rule used as a template in the docs",
            default_severity: Severity::Warning,
            default_confidence: Confidence::Low,
            stability: Stability::Experimental,
            rationale: "Why the pattern matters, in the same paragraph as the limitation.",
            limitations: "What this rule does not see, and the dominant false-negative mode.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        for_each_contract_fn(ctx.ast, |m| {
            if !is_pub(&m.vis) || is_test(&m.attrs) {
                return;
            }
            let fa = analyze(&m.block);
            if fa.calls.iter().any(|c| c.name == "example_trigger") {
                push_diag(
                    out,
                    "SL900",
                    Severity::Warning,
                    Confidence::Low,
                    "example finding",
                    loc(m.sig.ident.span()),
                    Some("how to fix it"),
                    None,
                );
            }
        });
    }
}
```

Then register it in `rules::all()`.

## 3. Constraints you must respect

- **Do not panic.** Rules run over arbitrary, sometimes invalid input. Return no
  diagnostic rather than unwrapping.
- **Macros are opaque.** You may look at a macro's name; do not try to reconstruct
  expressions from its tokens.
- **There is no type resolution.** Use only syntactic evidence (literals, `as`
  casts, explicit annotations/signatures).
- **Per-file only.** Do not assume other files exist. One level into same-file
  helpers (as SL001 does) is the maximum.
- **State the limitation** in `RuleMeta::limitations`, in the same paragraph as
  the capability claim. Do not bury it.

## 4. Fixtures and tests (required)

Add, under `tests/fixtures/`:

- `vulnerable/<rule>.rs` — a **true positive**.
- `safe/<rule>.rs` — a **negative** that must not fire.
- a **tricky negative** (for example, authorization in a helper) where relevant.
- a **documented false negative** proving the stated limitation is real.

Wire them into `tests/lint.rs` with the `positive!` / `negative!` macros.

## 5. Priorities

1. `Positive`
2. `Severity` and `Confidence` defaults
3. `Stability` (`experimental` unless the corpus justifies `stable`)
4. Fixtures, then the rule

## 6. Before you open a PR

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

All three must be clean. Update `docs/BENCHMARKS.md` if the rule changes corpus
results.
