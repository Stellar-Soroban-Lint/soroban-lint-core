//! SL003 — unchecked arithmetic.

use std::collections::HashSet;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::BinOp;

use crate::analysis::{int_evidence, int_idents, is_pub, is_test, is_zero_literal, loc};
use crate::context::Context;
use crate::diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
use crate::registry::Rule;

use super::{for_each_contract_fn, push_diag};

/// SL003.
pub struct Sl003;

/// `(display, is_compound_assignment)` for arithmetic operators.
fn arith_op(op: &BinOp) -> Option<(&'static str, bool)> {
    match op {
        BinOp::Add(_) => Some(("+", false)),
        BinOp::Sub(_) => Some(("-", false)),
        BinOp::Mul(_) => Some(("*", false)),
        BinOp::Div(_) => Some(("/", false)),
        BinOp::Rem(_) => Some(("%", false)),
        BinOp::AddAssign(_) => Some(("+=", true)),
        BinOp::SubAssign(_) => Some(("-=", true)),
        BinOp::MulAssign(_) => Some(("*=", true)),
        BinOp::DivAssign(_) => Some(("/=", true)),
        BinOp::RemAssign(_) => Some(("%=", true)),
        _ => None,
    }
}

struct ArithVisitor<'a> {
    ints: &'a HashSet<String>,
    out: &'a mut Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for ArithVisitor<'_> {
    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if let Some((op, assign)) = arith_op(&node.op) {
            let typed = int_evidence(&node.left, self.ints)
                && (assign || int_evidence(&node.right, self.ints));
            let div_by_zero = !assign
                && matches!(node.op, BinOp::Div(_) | BinOp::Rem(_))
                && is_zero_literal(&node.right);
            if typed || div_by_zero {
                push_diag(
                    self.out,
                    "SL003",
                    Severity::Warning,
                    Confidence::Low,
                    format!("unchecked `{op}` on integer operands can overflow or divide by zero"),
                    loc(node.span()),
                    Some("use `checked_add`/`checked_sub`/`checked_mul` or `saturating_*`, or enable `overflow-checks` in the release profile"),
                    None,
                );
            }
        }
        syn::visit::visit_expr_binary(self, node);
    }
}

impl Rule for Sl003 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "SL003",
            name: "unchecked-arithmetic",
            description: "Arithmetic on syntactically-integer operands without overflow checks",
            default_severity: Severity::Warning,
            default_confidence: Confidence::Low,
            stability: Stability::Experimental,
            rationale:
                "Soroban contracts handle token amounts in fixed-width integers. Wrap-around \
or division by zero can corrupt accounting.",
            limitations:
                "There is no type resolution, so this only fires when both operands carry \
syntactic integer evidence (literals, explicit annotations, `as` casts). Most values loaded from \
storage have no such evidence and are NOT flagged, which is the dominant false-negative mode.",
        }
    }

    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>) {
        for_each_contract_fn(ctx.ast, |m| {
            if !is_pub(&m.vis) || is_test(&m.attrs) {
                return;
            }
            let ints = int_idents(&m.sig, &m.block);
            let mut v = ArithVisitor { ints: &ints, out };
            v.visit_block(&m.block);
        });
    }
}
