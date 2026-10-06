//! SL008 — `unsafe` / missing `#![no_std]`.

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::analysis::{has_attr, is_test, loc};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

use super::push_diag;

/// SL008.
pub struct Sl008;

#[derive(Default)]
struct UnsafeVisitor {
    found: Vec<(Span, &'static str)>,
}

impl<'ast> Visit<'ast> for UnsafeVisitor {
    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.found.push((node.unsafe_token.span, "unsafe block"));
        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.unsafety.is_some() {
            self.found.push((node.sig.ident.span(), "unsafe fn"));
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if node.unsafety.is_some() {
            self.found.push((node.self_ty.span(), "unsafe impl"));
        }
        syn::visit::visit_item_impl(self, node);
    }
}

fn item_is_test(item: &syn::Item) -> bool {
    match item {
        syn::Item::Mod(m) => is_test(&m.attrs),
        syn::Item::Fn(f) => is_test(&f.attrs),
        syn::Item::Impl(i) => is_test(&i.attrs),
        _ => false,
    }
}

fn item_has_attr(item: &syn::Item, name: &str) -> bool {
    match item {
        syn::Item::Struct(s) => has_attr(&s.attrs, name),
        syn::Item::Enum(e) => has_attr(&e.attrs, name),
        syn::Item::Fn(f) => has_attr(&f.attrs, name),
        syn::Item::Impl(i) => has_attr(&i.attrs, name),
        syn::Item::Trait(t) => has_attr(&t.attrs, name),
        _ => false,
    }
}

impl Rule for Sl008 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL008",
            name: "unsafe-and-no-std",
            description: "Contract crate uses `unsafe` or omits `#![no_std]`",
            default_severity: Severity::Warning,
            default_confidence: Confidence::High,
            stability: Stability::Stable,
            rationale:
                "Soroban contracts run in a constrained WASM sandbox; `unsafe` bypasses the \
guarantees that make the contract auditable, and a missing `#![no_std]` links a std runtime that \
the host does not provide.",
            limitations: "`unsafe` detection is high-confidence and syntactic. The `#![no_std]` \
check is low-confidence: valid `#![cfg_attr(not(test), no_std)]` patterns are reported as missing.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        for item in &ctx.ast.items {
            if item_is_test(item) {
                continue;
            }
            let mut v = UnsafeVisitor::default();
            v.visit_item(item);
            for (span, kind) in v.found {
                push_diag(
                    out,
                    "SL008",
                    Severity::Warning,
                    Confidence::High,
                    format!("contract crate uses `{kind}`"),
                    loc(span),
                    Some("remove `unsafe`; Soroban contracts should not need it"),
                    None,
                );
            }
        }

        // `#![no_std]` check: only when a `#[contract]` item is present.
        let has_contract = ctx.ast.items.iter().any(|it| item_has_attr(it, "contract"));
        let has_no_std = ctx.ast.attrs.iter().any(|a| {
            matches!(a.style, syn::AttrStyle::Inner(_))
                && (a.path().is_ident("no_std")
                    || (a.path().is_ident("cfg_attr")
                        && matches!(&a.meta, syn::Meta::List(l) if l.tokens.to_string().contains("no_std"))))
        });
        // `#![no_std]` is a crate-root attribute; only check crate roots.
        let is_crate_root = ctx.file.ends_with("lib.rs") || ctx.file.ends_with("main.rs");
        if has_contract && is_crate_root && !has_no_std {
            push_diag(
                out,
                "SL008",
                Severity::Warning,
                Confidence::Low,
                "`#[contract]` crate does not declare `#![no_std]`",
                crate::analysis::Location {
                    start_line: 1,
                    start_column: 1,
                    end_line: 1,
                    end_column: 1,
                },
                Some("add `#![no_std]` (or `#![cfg_attr(not(test), no_std)]`) to the crate root"),
                None,
            );
        }
    }
}
