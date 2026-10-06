//! soroban-lint-core — syntactic, per-file static analysis for Soroban contracts.
//!
//! See `SPEC.md` at the repository root for the full specification, including the
//! scope statement and the limitations of every rule.
#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

pub mod analysis;
pub mod cargo;
pub mod config;
pub mod context;
pub mod diagnostic;
pub mod error;
pub mod registry;
pub mod render;
pub mod rules;
pub mod suppress;

pub use config::Config;
pub use context::Context;
pub use diagnostic::{Confidence, Diagnostic, RuleMeta, Severity, Stability};
pub use error::ConfigError;
pub use registry::{Registry, Rule};
pub use suppress::Suppressions;

/// Crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Parse and lint a single source file.
///
/// A parse failure is returned as a single `SL000` error diagnostic rather than
/// a panic or an `Err`, so callers can keep processing other files.
#[must_use]
pub fn lint_source(
    file: &str,
    source: &str,
    config: &Config,
    registry: &Registry,
) -> Vec<Diagnostic> {
    match syn::parse_file(source) {
        Ok(ast) => {
            let suppressions = Suppressions::parse(source);
            let ctx = Context {
                file,
                source,
                ast: &ast,
                config,
                suppressions: &suppressions,
            };
            registry.lint(&ctx)
        }
        Err(e) => {
            let start = e.span().start();
            vec![diagnostic::meta_diagnostic(
                file,
                format!("failed to parse: {e}"),
                start.line.max(1),
                start.column + 1,
                Severity::Error,
            )]
        }
    }
}
