//! Rule registry and the lint driver.

use crate::context::Context;
use crate::diagnostic::{meta_diagnostic, Diagnostic, RuleMeta, Severity};
use crate::rules;

/// A static analysis rule.
pub trait Rule: Send + Sync {
    /// Static metadata.
    fn meta(&self) -> RuleMeta;
    /// Emit diagnostics. Must not panic on any input.
    fn check(&self, ctx: &Context<'_>, out: &mut Vec<Diagnostic>);
}

/// Holds every compiled-in rule.
pub struct Registry {
    rules: Vec<Box<dyn Rule>>,
}

impl Registry {
    /// Registry with all v1 rules registered.
    #[must_use]
    pub fn default_set() -> Self {
        let mut r = Self { rules: Vec::new() };
        for rule in rules::all() {
            r.register(rule);
        }
        r
    }

    /// Add a rule.
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Look up a rule by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&dyn Rule> {
        self.rules
            .iter()
            .find(|r| r.meta().id == id)
            .map(AsRef::as_ref)
    }

    /// Metadata for every registered rule, sorted by id.
    #[must_use]
    pub fn metadata(&self) -> Vec<RuleMeta> {
        let mut v: Vec<RuleMeta> = self.rules.iter().map(|r| r.meta()).collect();
        v.sort_by_key(|m| m.id);
        v
    }

    /// Ids of every registered rule.
    #[must_use]
    pub fn known_ids(&self) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = self.rules.iter().map(|r| r.meta().id).collect();
        v.sort_unstable();
        v
    }

    /// Run enabled rules over one file, apply suppressions, sort, de-duplicate.
    #[must_use]
    pub fn lint(&self, ctx: &Context<'_>) -> Vec<Diagnostic> {
        let mut out: Vec<Diagnostic> = Vec::new();
        for rule in &self.rules {
            let meta = rule.meta();
            let Some(severity) = ctx.config.effective_severity(&meta) else {
                continue;
            };
            let start = out.len();
            rule.check(ctx, &mut out);
            for d in &mut out[start..] {
                d.file = ctx.file.to_string();
                d.severity = severity;
            }
        }

        // Unknown suppression ids -> SL000 warnings.
        let known = self.known_ids();
        for dir in ctx.suppressions.directives() {
            for id in &dir.ids {
                if !known.contains(&id.as_str()) {
                    out.push(meta_diagnostic(
                        ctx.file,
                        format!("suppression references unknown rule id {id:?}"),
                        dir.line,
                        1,
                        Severity::Warning,
                    ));
                }
            }
        }

        // Suppression filter.
        out.retain(|d| !ctx.suppressions.is_suppressed(&d.rule_id, d.start_line));

        out.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        out.dedup_by(|a, b| a.dedup_key() == b.dedup_key());
        out
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::default_set()
    }
}
