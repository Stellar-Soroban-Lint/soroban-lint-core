//! SL007 — unprotected initializer.

use crate::analysis::{analyze, has_attr, has_auth, is_pub, is_test, loc, mutation};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

use super::{for_each_contract_fn, push_diag};

/// SL007.
pub struct Sl007;

impl Rule for Sl007 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL007",
            name: "unprotected-initializer",
            description: "Initializer writes state with no already-initialized guard and no auth",
            default_severity: Severity::Warning,
            default_confidence: Confidence::Medium,
            stability: Stability::Experimental,
            rationale:
                "An initializer that can run more than once, or that anyone can call, lets an \
attacker re-set admin/owner state after deployment.",
            limitations:
                "Guards expressed via macros are invisible, and a guard delegated more than \
one call away is missed. This is a syntactic check, not a reachability analysis.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        for_each_contract_fn(ctx.ast, |m| {
            if !is_pub(&m.vis) || is_test(&m.attrs) {
                return;
            }
            let name = m.sig.ident.to_string();
            let init_like = matches!(name.as_str(), "init" | "initialize" | "__init")
                || has_attr(&m.attrs, "constructor");
            if !init_like {
                return;
            }
            let fa = analyze(&m.block);
            if mutation(&fa.calls).is_none() || has_auth(&fa) {
                return;
            }
            if fa
                .calls
                .iter()
                .any(|c| matches!(c.name.as_str(), "has" | "get"))
            {
                return;
            }
            push_diag(
                out,
                "SL007",
                Severity::Warning,
                Confidence::Medium,
                format!("`{name}` writes initialization state with no guard and no auth"),
                loc(m.sig.ident.span()),
                Some("add an already-initialized check (e.g. `if storage.has(&INIT) { panic_with_error!(..) }`) or require auth"),
                None,
            );
        });
    }
}
