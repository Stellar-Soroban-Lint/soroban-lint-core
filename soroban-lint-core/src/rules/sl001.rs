//! SL001 — missing `require_auth`.

use std::collections::HashMap;

use crate::analysis::{analyze, has_auth, is_pub, is_test, loc, mutation, FnAnalysis};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

use super::{for_each_contract_fn, push_diag};

/// SL001.
pub struct Sl001;

impl Rule for Sl001 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL001",
            name: "missing-require-auth",
            description: "Contract function mutates state without an authorization check",
            default_severity: Severity::Error,
            default_confidence: Confidence::Medium,
            stability: Stability::Stable,
            rationale: "A public contract function that writes storage or moves tokens without \
calling require_auth may let any caller mutate state or move funds they do not own.",
            limitations: "Syntactic and per-file: auth is considered present only if it appears \
directly in the function or in a same-file helper it calls within one level. Mutations performed \
inside macro bodies are invisible, and auth delegated more than one call away is missed.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        // Same-file helpers: free functions and non-public methods, including
        // non-`pub` methods of `#[contractimpl]` blocks (one-level auth following).
        let mut helpers: HashMap<String, FnAnalysis> = HashMap::new();
        for item in &ctx.ast.items {
            match item {
                syn::Item::Fn(f) => {
                    if !is_test(&f.attrs) {
                        helpers.insert(f.sig.ident.to_string(), analyze(&f.block));
                    }
                }
                syn::Item::Impl(imp) => {
                    if !crate::analysis::has_attr(&imp.attrs, "contractimpl") {
                        continue;
                    }
                    for it in &imp.items {
                        if let syn::ImplItem::Fn(method) = it {
                            if !is_test(&method.attrs) && !is_pub(&method.vis) {
                                helpers
                                    .insert(method.sig.ident.to_string(), analyze(&method.block));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        for_each_contract_fn(ctx.ast, |m| {
            if !is_pub(&m.vis) || is_test(&m.attrs) {
                return;
            }
            let fa = analyze(&m.block);
            if mutation(&fa.calls).is_none() {
                return;
            }
            if has_auth(&fa) {
                return;
            }
            // One level into same-file helpers.
            let via_helper = fa
                .callees
                .iter()
                .any(|c| helpers.get(c).is_some_and(has_auth));
            if via_helper {
                return;
            }
            push_diag(
                out,
                "SL001",
                Severity::Error,
                Confidence::Medium,
                format!(
                    "`{}` mutates state without an authorization check",
                    m.sig.ident
                ),
                loc(m.sig.ident.span()),
                Some(
                    "call `require_auth()` (or `require_auth_for_args`) for the authorizing \
address before the first mutation",
                ),
                Some("addr.require_auth()"),
            );
        });
    }
}
