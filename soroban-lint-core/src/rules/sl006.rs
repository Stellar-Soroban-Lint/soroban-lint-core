//! SL006 — questionable storage type.

use crate::analysis::{analyze, chain_has, is_pub, is_test, loc};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

use super::{for_each_contract_fn, push_diag};

/// SL006.
pub struct Sl006;

const PERSISTENT_NAMES: [&str; 6] = ["balance", "owner", "allowance", "admin", "supply", "total"];

impl Rule for Sl006 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL006",
            name: "questionable-storage-type",
            description: "Temporary storage holds balance/ownership-like data",
            default_severity: Severity::Warning,
            default_confidence: Confidence::Low,
            stability: Stability::Experimental,
            rationale: "Temporary storage is evicted aggressively. Holding balances or ownership \
there can silently lose or reset accounting.",
            limitations: "Detection is a name heuristic on the key/value identifiers, so it can \
flag genuinely ephemeral data that happens to be named similarly, and it misses mis-typed data \
under innocuous names.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        for_each_contract_fn(ctx.ast, |m| {
            if !is_pub(&m.vis) || is_test(&m.attrs) {
                return;
            }
            let fa = analyze(&m.block);
            for c in &fa.calls {
                if c.name == "set" && chain_has(c, "temporary") {
                    if let Some(hit) = c
                        .arg_idents
                        .iter()
                        .find(|a| PERSISTENT_NAMES.contains(&a.as_str()))
                    {
                        push_diag(
                            out,
                            "SL006",
                            Severity::Warning,
                            Confidence::Low,
                            format!("`{hit}`-like data written to temporary storage"),
                            loc(c.span),
                            Some("use persistent storage for balance/ownership data"),
                            None,
                        );
                    }
                }
            }
        });
    }
}
