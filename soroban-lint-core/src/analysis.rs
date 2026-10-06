//! Syntactic analysis helpers shared by the rules.
//!
//! Everything here is heuristic: it uses names and shapes only, because there
//! is no type resolution (see `SPEC.md` §1).

use std::collections::HashSet;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Block, Expr, Macro, Meta, Signature, Type, Visibility};

/// A 1-based source region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    /// 1-based inclusive start line.
    pub start_line: usize,
    /// 1-based inclusive start column.
    pub start_column: usize,
    /// 1-based inclusive end line.
    pub end_line: usize,
    /// 1-based exclusive end column.
    pub end_column: usize,
}

/// Convert a `proc_macro2` span into a 1-based location.
#[must_use]
pub fn loc(span: Span) -> Location {
    let s = span.start();
    let e = span.end();
    Location {
        start_line: s.line,
        start_column: s.column + 1,
        end_line: e.line,
        end_column: e.column + 1,
    }
}

/// A method call with its full receiver method chain (outermost first).
#[derive(Debug, Clone)]
pub struct Call {
    /// Method name.
    pub name: String,
    /// Receiver chain of method names.
    pub chain: Vec<String>,
    /// Root identifier of the receiver, if any.
    pub receiver: Option<String>,
    /// Identifier names appearing in the arguments.
    pub arg_idents: Vec<String>,
    /// Span of the method identifier.
    pub span: Span,
}

/// A macro invocation, identified by the last path segment only.
#[derive(Debug, Clone)]
pub struct MacroCall {
    /// Macro name (last path segment).
    pub name: String,
    /// Span of the macro path.
    pub span: Span,
}

/// Facts gathered from one function body.
#[derive(Debug, Default, Clone)]
pub struct FnAnalysis {
    /// All method calls.
    pub calls: Vec<Call>,
    /// All macro invocations.
    pub macros: Vec<MacroCall>,
    /// All plain function call names.
    pub callees: Vec<String>,
    /// Span of the first indexing expression, if any.
    pub index_span: Option<Span>,
}

#[derive(Default)]
struct Collector {
    calls: Vec<Call>,
    macros: Vec<MacroCall>,
    callees: Vec<String>,
    index_span: Option<Span>,
}

impl<'ast> Visit<'ast> for Collector {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let receiver_expr = Expr::MethodCall(node.clone());
        let chain = method_chain(&receiver_expr);
        let mut arg_idents = Vec::new();
        for arg in &node.args {
            arg_idents.extend(ident_names(arg));
        }
        self.calls.push(Call {
            name: node.method.to_string(),
            chain,
            receiver: root_ident(&node.receiver),
            arg_idents,
            span: node.method.span(),
        });
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.macros.push(MacroCall {
            name: node
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            span: node.path.span(),
        });
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(p) = &*node.func {
            if let Some(seg) = p.path.segments.last() {
                self.callees.push(seg.ident.to_string());
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_index(&mut self, node: &'ast syn::ExprIndex) {
        if self.index_span.is_none() {
            self.index_span = Some(node.span());
        }
        syn::visit::visit_expr_index(self, node);
    }
}

/// Collect the method-name chain of an expression, outermost first.
#[must_use]
pub fn method_chain(expr: &Expr) -> Vec<String> {
    match expr {
        Expr::MethodCall(m) => {
            let mut v = vec![m.method.to_string()];
            v.extend(method_chain(&m.receiver));
            v
        }
        Expr::Field(f) => method_chain(&f.base),
        Expr::Paren(p) => method_chain(&p.expr),
        Expr::Group(g) => method_chain(&g.expr),
        _ => Vec::new(),
    }
}

/// Analyze a function body.
#[must_use]
pub fn analyze(block: &Block) -> FnAnalysis {
    let mut c = Collector::default();
    c.visit_block(block);
    FnAnalysis {
        calls: c.calls,
        macros: c.macros,
        callees: c.callees,
        index_span: c.index_span,
    }
}

/// Root identifier of an expression's receiver chain.
#[must_use]
pub fn root_ident(expr: &Expr) -> Option<String> {
    match expr {
        Expr::MethodCall(m) => root_ident(&m.receiver),
        Expr::Field(f) => root_ident(&f.base),
        Expr::Path(p) => p.path.get_ident().map(ToString::to_string),
        Expr::Paren(p) => root_ident(&p.expr),
        Expr::Group(g) => root_ident(&g.expr),
        _ => None,
    }
}

/// Local bindings initialized from a `.get(..)` (storage load).
#[must_use]
pub fn loaded_names(block: &Block) -> HashSet<String> {
    struct LocalCollector<'a> {
        set: &'a mut HashSet<String>,
    }
    impl<'ast> Visit<'ast> for LocalCollector<'_> {
        fn visit_local(&mut self, node: &'ast syn::Local) {
            if let Some(init) = &node.init {
                if expr_has_method(&init.expr, "get") {
                    if let Some(name) = binding_name(&node.pat) {
                        self.set.insert(name);
                    }
                }
            }
            syn::visit::visit_local(self, node);
        }
    }
    let mut set = HashSet::new();
    let mut c = LocalCollector { set: &mut set };
    c.visit_block(block);
    set
}

/// Whether an expression contains a method call with the given name.
#[must_use]
pub fn expr_has_method(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            m.method == name
                || expr_has_method(&m.receiver, name)
                || m.args.iter().any(|a| expr_has_method(a, name))
        }
        Expr::Paren(p) => expr_has_method(&p.expr, name),
        Expr::Group(g) => expr_has_method(&g.expr, name),
        _ => false,
    }
}

/// Whether any call or macro in `fa` has one of `names`.
#[must_use]
pub fn has_named(fa: &FnAnalysis, names: &[&str]) -> bool {
    fa.calls.iter().any(|c| names.contains(&c.name.as_str()))
        || fa.macros.iter().any(|m| names.contains(&m.name.as_str()))
}

/// Whether any **auth** call or macro is present.
#[must_use]
pub fn has_auth(fa: &FnAnalysis) -> bool {
    has_named(fa, &["require_auth", "require_auth_for_args"])
}

const STORAGE_MARKERS: [&str; 4] = ["storage", "persistent", "temporary", "instance"];
const STORAGE_MUTATORS: [&str; 3] = ["set", "remove", "update"];
const TOKEN_OPS: [&str; 4] = ["transfer", "transfer_from", "mint", "burn"];

/// A storage mutation or token operation, if present.
#[must_use]
pub fn mutation(calls: &[Call]) -> Option<&Call> {
    calls.iter().find(|c| {
        (STORAGE_MUTATORS.contains(&c.name.as_str())
            && c.chain
                .iter()
                .any(|n| STORAGE_MARKERS.contains(&n.as_str())))
            || TOKEN_OPS.contains(&c.name.as_str())
    })
}

/// Whether the receiver chain includes any storage marker.
#[must_use]
pub fn chain_has(call: &Call, name: &str) -> bool {
    call.chain.iter().any(|c| c == name)
}

/// Whether a call's receiver chain includes any storage marker.
#[must_use]
pub fn touches_storage(call: &Call) -> bool {
    call.chain
        .iter()
        .any(|n| STORAGE_MARKERS.contains(&n.as_str()))
}

/// Whether a call touches persistent or instance storage (but not temporary).
#[must_use]
pub fn is_persistent_or_instance(call: &Call) -> bool {
    let is_storage_write = STORAGE_MUTATORS.contains(&call.name.as_str())
        && call
            .chain
            .iter()
            .any(|n| STORAGE_MARKERS.contains(&n.as_str()));
    is_storage_write && !chain_has(call, "temporary")
}

/// Whether the item carries an attribute with the given path.
#[must_use]
pub fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

/// Whether the item is `#[cfg(test)]`.
#[must_use]
pub fn is_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && matches!(&a.meta, Meta::List(l) if l.tokens.to_string().contains("test"))
    })
}

/// Public visibility.
#[must_use]
pub fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

const INT_TYPES: [&str; 12] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
];

/// Whether a type is one of the primitive integer types.
#[must_use]
pub fn is_int_type(ty: &Type) -> bool {
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            return INT_TYPES.contains(&seg.ident.to_string().as_str());
        }
    }
    false
}

/// Syntactic evidence that an expression is an integer.
#[must_use]
// The set is always constructed locally with the default hasher; generalizing the
// hasher would add noise without a use case.
#[allow(clippy::implicit_hasher)]
pub fn int_evidence(expr: &Expr, ints: &HashSet<String>) -> bool {
    match expr {
        Expr::Lit(l) => matches!(l.lit, syn::Lit::Int(_)),
        Expr::Cast(c) => is_int_type(&c.ty),
        Expr::Path(p) => p
            .path
            .get_ident()
            .is_some_and(|i| ints.contains(&i.to_string())),
        Expr::Paren(p) => int_evidence(&p.expr, ints),
        Expr::Group(g) => int_evidence(&g.expr, ints),
        Expr::Reference(r) => int_evidence(&r.expr, ints),
        _ => false,
    }
}

/// Name bound by a pattern, unwrapping `let x: T = ..` type ascriptions.
#[must_use]
pub fn binding_name(pat: &syn::Pat) -> Option<String> {
    match pat {
        syn::Pat::Ident(pi) => Some(pi.ident.to_string()),
        syn::Pat::Type(pt) => binding_name(&pt.pat),
        _ => None,
    }
}

/// Identifiers in a function that have explicit integer type evidence.
#[must_use]
pub fn int_idents(sig: &Signature, block: &Block) -> HashSet<String> {
    struct LocalCollector<'a> {
        set: &'a mut HashSet<String>,
    }
    impl<'ast> Visit<'ast> for LocalCollector<'_> {
        fn visit_local(&mut self, node: &'ast syn::Local) {
            // `let x: T = ..` stores the annotation inside the pattern.
            if let syn::Pat::Type(pt) = &node.pat {
                if is_int_type(&pt.ty) {
                    if let Some(name) = binding_name(&pt.pat) {
                        self.set.insert(name);
                    }
                }
            }
            syn::visit::visit_local(self, node);
        }
    }

    let mut set = HashSet::new();
    for arg in &sig.inputs {
        if let syn::FnArg::Typed(pt) = arg {
            if is_int_type(&pt.ty) {
                if let Some(name) = binding_name(&pt.pat) {
                    set.insert(name);
                }
            }
        }
    }
    let mut c = LocalCollector { set: &mut set };
    c.visit_block(block);
    set
}

/// Whether an expression is the integer literal `0`.
#[must_use]
pub fn is_zero_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(l) => {
            matches!(&l.lit, syn::Lit::Int(i) if i.base10_parse::<u128>().is_ok_and(|v| v == 0))
        }
        Expr::Paren(p) => is_zero_literal(&p.expr),
        Expr::Group(g) => is_zero_literal(&g.expr),
        _ => false,
    }
}

/// Collect identifier names referenced inside an expression.
#[must_use]
pub fn ident_names(expr: &Expr) -> Vec<String> {
    struct I {
        names: Vec<String>,
    }
    impl<'ast> Visit<'ast> for I {
        fn visit_ident(&mut self, node: &'ast proc_macro2::Ident) {
            self.names.push(node.to_string());
        }
    }
    let mut i = I { names: Vec::new() };
    i.visit_expr(expr);
    i.names
}
