//! SL002 — panic hazards.

use crate::analysis::{analyze, is_pub, is_test, loc};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

use super::{for_each_contract_fn, push_diag};

/// SL002.
pub struct Sl002;

impl Rule for Sl002 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL002",
            name: "panic-hazards",
            description: "Contract function contains a call that can panic",
            default_severity: Severity::Warning,
            default_confidence: Confidence::High,
            stability: Stability::Stable,
            rationale: "In Soroban an unhandled panic aborts the invocation. Explicit panics and \
indexing are better expressed as Result returns or panic_with_error! so callers get a typed error.",
            limitations: "Syntactic: indexing is flagged even when the surrounding code proves it \
in-bounds (hence medium confidence), and panics raised by called functions in other files are not \
seen. panics inside macro bodies are not expanded.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        for_each_contract_fn(ctx.ast, |m| {
            if !is_pub(&m.vis) || is_test(&m.attrs) {
                return;
            }
            let fa = analyze(&m.block);

            for c in &fa.calls {
                if matches!(c.name.as_str(), "unwrap" | "expect") {
                    push_diag(
                        out,
                        "SL002",
                        Severity::Warning,
                        Confidence::High,
                        format!("`{}` can panic in a contract function", c.name),
                        loc(c.span),
                        Some("return `Result<_, ContractError>` and use `?`, or use `panic_with_error!`"),
                        None,
                    );
                }
            }

            for mac in &fa.macros {
                if matches!(
                    mac.name.as_str(),
                    "panic" | "unreachable" | "todo" | "unimplemented"
                ) {
                    push_diag(
                        out,
                        "SL002",
                        Severity::Warning,
                        Confidence::High,
                        format!("`{}!` aborts the invocation", mac.name),
                        loc(mac.span),
                        Some("prefer `panic_with_error!` with a `#[contracterror]` type"),
                        None,
                    );
                }
            }

            if let Some(span) = fa.index_span {
                push_diag(
                    out,
                    "SL002",
                    Severity::Warning,
                    Confidence::Medium,
                    "indexing can panic and is not bounds-checked at compile time",
                    loc(span),
                    Some("use `get(..)` with explicit handling, or a `checked_*` accessor"),
                    None,
                );
            }
        });
    }
}
