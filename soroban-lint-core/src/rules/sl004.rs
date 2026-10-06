//! SL004 — unbounded storage growth.

use crate::analysis::{analyze, is_pub, is_test, loaded_names, loc, touches_storage};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

use super::{for_each_contract_fn, push_diag};

/// SL004.
pub struct Sl004;

const GROWTH: [&str; 5] = ["push", "push_back", "insert", "append", "extend"];

impl Rule for Sl004 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL004",
            name: "unbounded-storage-growth",
            description:
                "A collection loaded from storage is grown and written back without a bound",
            default_severity: Severity::Warning,
            default_confidence: Confidence::Low,
            stability: Stability::Experimental,
            rationale: "Growing a stored collection without a visible cap lets a caller inflate \
ledger entries until writes exceed resource limits, which can brick the entrypoint.",
            limitations:
                "The length bound is detected only as a `.len()` call in the same function; \
bounds enforced by a helper, a macro, or configuration are invisible, and the storage write is \
matched by method name rather than type.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        for_each_contract_fn(ctx.ast, |m| {
            if !is_pub(&m.vis) || is_test(&m.attrs) {
                return;
            }
            let fa = analyze(&m.block);
            let loaded = loaded_names(&m.block);
            if loaded.is_empty() {
                return;
            }
            let writes_back = fa
                .calls
                .iter()
                .any(|c| c.name == "set" && touches_storage(c));
            if !writes_back {
                return;
            }
            let bounded = fa.calls.iter().any(|c| c.name == "len");
            if bounded {
                return;
            }
            for c in &fa.calls {
                if GROWTH.contains(&c.name.as_str()) {
                    if let Some(recv) = &c.receiver {
                        if loaded.contains(recv) {
                            push_diag(
                                out,
                                "SL004",
                                Severity::Warning,
                                Confidence::Low,
                                format!(
                                    "`{recv}` is loaded from storage and grown without a visible length bound"
                                ),
                                loc(c.span),
                                Some("enforce a maximum length before growing and return an error when exceeded"),
                                None,
                            );
                        }
                    }
                }
            }
        });
    }
}
