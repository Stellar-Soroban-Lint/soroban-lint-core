//! Inline suppression grammar.
//!
//! `syn` discards comments, so suppression is parsed from the raw source text
//! and matched against diagnostic start lines. See `SPEC.md` §2.2.

use std::collections::{HashMap, HashSet};

const SAME_LINE: &str = "// soroban-lint-ignore:";
const NEXT_LINE: &str = "// soroban-lint-ignore-next-line:";

/// A parsed suppression directive, retained for unknown-id reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    /// 1-based line the directive appears on.
    pub line: usize,
    /// Rule ids named by the directive.
    pub ids: Vec<String>,
}

/// Suppressions parsed from one source file.
#[derive(Debug, Clone, Default)]
pub struct Suppressions {
    same_line: HashMap<usize, HashSet<String>>,
    next_line: HashMap<usize, HashSet<String>>,
    directives: Vec<Directive>,
}

impl Suppressions {
    /// Parse directives from raw source text.
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let mut s = Self::default();
        for (idx, raw) in source.lines().enumerate() {
            let line_no = idx + 1;
            // The directive may be a whole-line comment or trail code on the same line.
            let (target, rest) = if let Some(pos) = raw.find(NEXT_LINE) {
                (line_no + 1, &raw[pos + NEXT_LINE.len()..])
            } else if let Some(pos) = raw.find(SAME_LINE) {
                (line_no, &raw[pos + SAME_LINE.len()..])
            } else {
                continue;
            };
            let ids: Vec<String> = rest
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            if ids.is_empty() {
                continue;
            }
            let bucket = if target == line_no {
                s.same_line.entry(target)
            } else {
                s.next_line.entry(target)
            };
            bucket.or_default().extend(ids.iter().cloned());
            s.directives.push(Directive { line: line_no, ids });
        }
        s
    }

    /// Whether `rule_id` is suppressed at `line`.
    #[must_use]
    pub fn is_suppressed(&self, rule_id: &str, line: usize) -> bool {
        self.same_line
            .get(&line)
            .is_some_and(|s| s.contains(rule_id))
            || self
                .next_line
                .get(&line)
                .is_some_and(|s| s.contains(rule_id))
    }

    /// All directives, for unknown-id reporting.
    #[must_use]
    pub fn directives(&self) -> &[Directive] {
        &self.directives
    }
}
