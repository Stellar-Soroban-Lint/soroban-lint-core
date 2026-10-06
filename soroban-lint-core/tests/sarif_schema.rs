//! Validate real SARIF output against the vendored official 2.1.0 schema.

use jsonschema::JSONSchema;
use serde_json::Value;
use soroban_lint_core::{lint_source, render, Config, Registry};

fn validate(src_name: &str, src: &str) {
    let schema_doc: Value =
        serde_json::from_str(include_str!("../schema/sarif-2.1.0.json")).expect("schema json");
    let compiled = JSONSchema::compile(&schema_doc).expect("schema compiles");

    let reg = Registry::default_set();
    let cfg = Config {
        experimental: true,
        ..Config::default()
    };
    let diags = lint_source(src_name, src, &cfg, &reg);
    let sarif: Value = serde_json::from_str(&render::sarif(&diags, &reg.metadata()))
        .expect("sarif is valid json");

    let errs: Vec<String> = match compiled.validate(&sarif) {
        Ok(()) => Vec::new(),
        Err(errors) => errors.map(|e| e.to_string()).collect(),
    };
    assert!(
        errs.is_empty(),
        "SARIF failed schema validation for {src_name}: {errs:#?}"
    );
}

#[test]
fn sarif_vulnerable_validates() {
    validate(
        "sl001_missing_auth.rs",
        include_str!("fixtures/vulnerable/sl001_missing_auth.rs"),
    );
}

#[test]
fn sarif_clean_validates() {
    validate(
        "sl008_clean.rs",
        include_str!("fixtures/safe/sl008_clean.rs"),
    );
}

#[test]
fn sarif_empty_findings_validates() {
    validate("empty.rs", "");
}
