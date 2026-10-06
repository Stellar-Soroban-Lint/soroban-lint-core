//! Diagnostic model shared by every output format.

use serde::{Deserialize, Serialize};

/// Severity of a finding. Ordering is `Info < Warning < Error` so that
/// `--fail-on` comparisons are a plain `>=`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational.
    Info,
    /// Warning.
    Warning,
    /// Error.
    Error,
}

impl Severity {
    /// Lowercase wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// Parse a config value (`off` is handled separately by [`crate::config`]).
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "info" => Some(Self::Info),
            "warning" | "warn" => Some(Self::Warning),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// SARIF `result.level`.
    #[must_use]
    pub fn to_sarif_level(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "note",
        }
    }
}

/// How much syntactic evidence backs a finding. Independent of severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Weak evidence; likely to include false positives.
    Low,
    /// Moderate evidence.
    Medium,
    /// Strong syntactic evidence.
    High,
}

impl Confidence {
    /// Lowercase wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Whether a rule is on by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stability {
    /// Enabled by default.
    Stable,
    /// Opt-in via config `experimental = true` or `--experimental`.
    Experimental,
}

impl Stability {
    /// Lowercase wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Experimental => "experimental",
        }
    }
}

/// A single finding. Field names are part of the public JSON contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Permanent rule identifier, e.g. `SL001`.
    pub rule_id: String,
    /// Effective severity (after config overrides).
    pub severity: Severity,
    /// Evidence strength.
    pub confidence: Confidence,
    /// One-line description, no trailing period.
    pub message: String,
    /// Path relative to the invocation root, forward slashes.
    pub file: String,
    /// 1-based inclusive start line.
    pub start_line: usize,
    /// 1-based inclusive start column.
    pub start_column: usize,
    /// 1-based inclusive end line.
    pub end_line: usize,
    /// 1-based exclusive end column.
    pub end_column: usize,
    /// Optional remediation text.
    pub help: Option<String>,
    /// Optional human-readable fix suggestion (not a machine edit).
    pub fix: Option<String>,
}

impl Diagnostic {
    /// Total-order sort key: `(file, start_line, start_column, rule_id)`.
    #[must_use]
    pub fn sort_key(&self) -> (&str, usize, usize, &str) {
        (
            &self.file,
            self.start_line,
            self.start_column,
            &self.rule_id,
        )
    }

    /// Identity used for de-duplication.
    #[must_use]
    pub fn dedup_key(&self) -> (&str, &str, usize, usize) {
        (
            &self.rule_id,
            &self.file,
            self.start_line,
            self.start_column,
        )
    }
}

/// Static metadata describing a rule. Emitted by `soroban-lint rules --format json`.
#[derive(Debug, Clone, Serialize)]
pub struct RuleMeta {
    /// Permanent id, e.g. `SL001`.
    pub id: &'static str,
    /// Kebab-case short name.
    pub name: &'static str,
    /// One-line description.
    pub description: &'static str,
    /// Default severity before config overrides.
    pub default_severity: Severity,
    /// Default confidence.
    pub default_confidence: Confidence,
    /// Default enablement.
    pub stability: Stability,
    /// Why the pattern matters.
    pub rationale: &'static str,
    /// Known false-positive/false-negative modes, stated with the claim.
    pub limitations: &'static str,
}

/// Reserved meta rule id for usage/parse/config/suppression diagnostics.
pub const META_RULE_ID: &str = "SL000";

/// Build an `SL000` meta diagnostic starting at the given position.
#[must_use]
pub fn meta_diagnostic(
    file: &str,
    message: impl Into<String>,
    line: usize,
    column: usize,
    severity: Severity,
) -> Diagnostic {
    Diagnostic {
        rule_id: META_RULE_ID.to_string(),
        severity,
        confidence: Confidence::High,
        message: message.into(),
        file: file.to_string(),
        start_line: line,
        start_column: column,
        end_line: line,
        end_column: column,
        help: None,
        fix: None,
    }
}
