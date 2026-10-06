//! SL005 — missing TTL extension.

use crate::analysis::{analyze, is_persistent_or_instance, is_pub, is_test, loc};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

use super::{for_each_contract_fn, push_diag};

/// SL005.
pub struct Sl005;

impl Rule for Sl005 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL005",
            name: "missing-ttl-extension",
            description: "Persistent or instance storage is written without extending its TTL",
            default_severity: Severity::Info,
            default_confidence: Confidence::Medium,
            stability: Stability::Experimental,
            rationale:
                "Persistent and instance entries expire. A write that never extends the TTL \
can let live state lapse and be archived.",
            limitations: "Only checks the same function; a TTL extended by a helper or a separate \
entrypoint is not seen, and `temporary` storage (which is meant to expire) is not flagged.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        for_each_contract_fn(ctx.ast, |m| {
            if !is_pub(&m.vis) || is_test(&m.attrs) {
                return;
            }
            if m.sig.ident == "__constructor" {
                return;
            }
            let fa = analyze(&m.block);
            let Some(write) = fa.calls.iter().find(|c| is_persistent_or_instance(c)) else {
                return;
            };
            if fa.calls.iter().any(|c| c.name == "extend_ttl") {
                return;
            }
            push_diag(
                out,
                "SL005",
                Severity::Info,
                Confidence::Medium,
                "persistent/instance write without an `extend_ttl` in the same function",
                loc(write.span),
                Some("call `extend_ttl(&key, threshold, extend_to)` after the write"),
                None,
            );
        });
    }
}
