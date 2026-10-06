//! Project-level checks that are not tied to a Rust source file.

use crate::diagnostic::{Confidence, Diagnostic, Severity};

/// Check that `[profile.release] overflow-checks = true` is set.
///
/// Returns a diagnostic on `Cargo.toml` when overflow checks are absent or
/// disabled. This complements SL003, which can only flag arithmetic where
/// syntactic integer evidence exists.
#[must_use]
pub fn overflow_checks_diagnostic(file: &str, doc: &str) -> Option<Diagnostic> {
    let value: toml::Value = toml::from_str(doc).ok()?;
    let enabled = value
        .get("profile")
        .and_then(|p| p.get("release"))
        .and_then(|r| r.get("overflow-checks"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    if enabled {
        return None;
    }
    let line = line_of(doc, "overflow-checks")
        .or_else(|| line_of(doc, "[profile.release]"))
        .unwrap_or(1);
    Some(Diagnostic {
        rule_id: "SL003".to_string(),
        severity: Severity::Warning,
        confidence: Confidence::Medium,
        message: "release profile does not enable integer overflow checks".to_string(),
        file: file.to_string(),
        start_line: line,
        start_column: 1,
        end_line: line,
        end_column: 1,
        help: Some(
            "add `overflow-checks = true` under [profile.release] so checked arithmetic traps"
                .to_string(),
        ),
        fix: Some("overflow-checks = true".to_string()),
    })
}

fn line_of(doc: &str, needle: &str) -> Option<usize> {
    doc.lines()
        .position(|l| l.trim_start().starts_with(needle))
        .map(|i| i + 1)
}
