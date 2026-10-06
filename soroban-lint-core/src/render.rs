//! Output renderers: text, JSON, and SARIF 2.1.0.

use std::fmt::Write as _;

use serde_json::json;

use crate::diagnostic::{Diagnostic, RuleMeta};

/// Version string embedded in JSON/SARIF output.
pub const OUTPUT_VERSION: &str = "1";

/// Human-readable, one finding per line.
#[must_use]
pub fn text(diags: &[Diagnostic]) -> String {
    let mut out = String::new();
    for d in diags {
        let _ = writeln!(
            out,
            "{}:{}:{}: {} {} ({}): {}",
            d.file,
            d.start_line,
            d.start_column,
            d.severity.as_str(),
            d.rule_id,
            d.confidence.as_str(),
            d.message,
        );
        if let Some(help) = &d.help {
            let _ = writeln!(out, "  help: {help}");
        }
    }
    out
}

/// JSON document: `{ "version", "diagnostics": [...] }`.
#[must_use]
pub fn json(diags: &[Diagnostic]) -> String {
    let doc = json!({
        "version": OUTPUT_VERSION,
        "diagnostics": diags,
    });
    serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".to_string())
}

/// JSON document describing the rule registry.
#[must_use]
pub fn rules_json(rules: &[RuleMeta]) -> String {
    let doc = json!({
        "version": OUTPUT_VERSION,
        "rules": rules,
    });
    serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".to_string())
}

/// SARIF 2.1.0 document.
#[must_use]
pub fn sarif(diags: &[Diagnostic], rules: &[RuleMeta]) -> String {
    let rules_json: Vec<serde_json::Value> = rules
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": to_pascal(r.name),
                "shortDescription": { "text": r.description },
                "fullDescription": { "text": r.rationale },
                "help": { "text": format!("{}\n\nLimitations: {}", r.rationale, r.limitations) },
                "defaultConfiguration": { "level": r.default_severity.to_sarif_level() },
                "properties": { "stability": r.stability.as_str() },
            })
        })
        .collect();

    let results: Vec<serde_json::Value> = diags
        .iter()
        .map(|d| {
            json!({
                "ruleId": d.rule_id,
                "level": d.severity.to_sarif_level(),
                "message": { "text": d.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": d.file },
                        "region": {
                            "startLine": d.start_line,
                            "startColumn": d.start_column,
                            "endLine": d.end_line,
                            "endColumn": d.end_column,
                        }
                    }
                }],
                "partialFingerprints": {
                    "sorobanLint/v1": fingerprint(&d.rule_id, &d.file, d.start_line)
                }
            })
        })
        .collect();

    let doc = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "soroban-lint",
                    "informationUri": "https://github.com/Stellar-Soroban-Lint/soroban-lint-core",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": rules_json,
                }
            },
            "results": results,
        }]
    });
    serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".to_string())
}

fn to_pascal(kebab: &str) -> String {
    kebab
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Stable, deterministic fingerprint (FNV-1a, 64-bit) for SARIF dedupe.
fn fingerprint(rule_id: &str, file: &str, line: usize) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in format!("{rule_id}\u{1f}{file}\u{1f}{line}").bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}
