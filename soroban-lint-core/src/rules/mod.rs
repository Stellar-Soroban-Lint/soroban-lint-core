//! The v1 rule set.

use crate::analysis::Location;
use crate::diagnostic::{Confidence, Diagnostic, Severity};
use crate::registry::Rule;

pub mod sl001;
pub mod sl002;
pub mod sl003;
pub mod sl004;
pub mod sl005;
pub mod sl006;
pub mod sl007;
pub mod sl008;

/// Instantiate every v1 rule.
#[must_use]
pub fn all() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(sl001::Sl001),
        Box::new(sl002::Sl002),
        Box::new(sl003::Sl003),
        Box::new(sl004::Sl004),
        Box::new(sl005::Sl005),
        Box::new(sl006::Sl006),
        Box::new(sl007::Sl007),
        Box::new(sl008::Sl008),
    ]
}

/// Push a diagnostic; the registry fills in the file path and severity override.
#[allow(clippy::too_many_arguments)] // A diagnostic legitimately has this many fields.
pub(crate) fn push_diag(
    out: &mut Vec<Diagnostic>,
    rule_id: &str,
    severity: Severity,
    confidence: Confidence,
    message: impl Into<String>,
    at: Location,
    help: Option<&str>,
    fix: Option<&str>,
) {
    out.push(Diagnostic {
        rule_id: rule_id.to_string(),
        severity,
        confidence,
        message: message.into(),
        file: String::new(),
        start_line: at.start_line,
        start_column: at.start_column,
        end_line: at.end_line,
        end_column: at.end_column,
        help: help.map(str::to_string),
        fix: fix.map(str::to_string),
    });
}

/// Compile-checked rule template referenced by `docs/WRITING_RULES.md`.
///
/// This module is only built under `cfg(test)`, which guarantees the template
/// actually compiles rather than rotting in documentation.
#[cfg(test)]
mod template {
    use super::{for_each_contract_fn, push_diag};
    use crate::analysis::{analyze, is_pub, is_test, loc};
    use crate::context::Context;
    use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
    use crate::registry::Rule;

    /// Example rule. Replace the metadata and `check` body.
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
                // Scope guard: public contract functions only.
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

    #[test]
    fn example_rule_compiles_and_reports_metadata() {
        assert_eq!(ExampleRule.meta().id, "SL900");
    }
}

/// Call `f` for every function inside a `#[contractimpl]` impl block.
pub(crate) fn for_each_contract_fn<'a>(ast: &'a syn::File, mut f: impl FnMut(&'a syn::ImplItemFn)) {
    for item in &ast.items {
        if let syn::Item::Impl(imp) = item {
            if !crate::analysis::has_attr(&imp.attrs, "contractimpl") {
                continue;
            }
            for it in &imp.items {
                if let syn::ImplItem::Fn(m) = it {
                    f(m);
                }
            }
        }
    }
}
