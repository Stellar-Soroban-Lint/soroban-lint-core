//! Inputs handed to every rule.

use crate::config::Config;
use crate::suppress::Suppressions;

/// Everything a rule may read while checking one file.
pub struct Context<'a> {
    /// Path relative to the invocation root, forward slashes.
    pub file: &'a str,
    /// Raw source text (comments intact).
    pub source: &'a str,
    /// Parsed AST (comments stripped).
    pub ast: &'a syn::File,
    /// Effective configuration.
    pub config: &'a Config,
    /// Parsed inline suppressions.
    pub suppressions: &'a Suppressions,
}
