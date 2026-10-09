//! The scope statement is a promise the project makes on every surface.
//!
//! `SPEC.md` is normative: this test reads the statement out of it and requires
//! the same words, on one line, everywhere else. A surface that drops or rewords
//! it fails here rather than shipping.

use std::fs;
use std::path::{Path, PathBuf};

/// Repository root: the crate lives in the `<repo>/soroban-lint-core` directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("repository root is reachable from the crate manifest directory")
}

/// The normative statement, taken from `SPEC.md` rather than duplicated here.
fn normative_statement() -> String {
    let spec = fs::read_to_string(repo_root().join("SPEC.md")).expect("SPEC.md is readable");
    spec.lines()
        .find(|line| line.contains("soroban-lint performs syntactic,"))
        .expect("SPEC.md contains the scope statement")
        .trim_start_matches('>')
        .trim()
        .to_string()
}

#[test]
fn the_normative_statement_is_well_formed() {
    let statement = normative_statement();
    assert!(
        statement.starts_with("soroban-lint performs syntactic, per-file analysis"),
        "unexpected statement: {statement}"
    );
    assert!(
        statement.ends_with("this tool is not a substitute for an audit."),
        "unexpected statement: {statement}"
    );
    assert!(!statement.contains('\n'), "the statement must be one line");
    assert!(
        !statement.contains("  "),
        "the statement must not contain double spaces"
    );
}

#[test]
fn every_documentation_surface_carries_the_statement_verbatim() {
    let statement = normative_statement();
    let surfaces = [
        "SPEC.md",
        // The landing page makes the main capability claim and must carry its
        // limits in the same section, just like the repository docs.
        "site/index.md",
        "README.md",
        "CONTRIBUTING.md",
        "SECURITY.md",
        "docs/ARCHITECTURE.md",
        "docs/BENCHMARKS.md",
        // Published as the release notes, so it is a user-facing surface too.
        ".github/workflows/release.yml",
    ];

    for surface in surfaces {
        let path = repo_root().join(surface);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        assert!(
            text.contains(&statement),
            "{surface} does not carry the scope statement verbatim"
        );
    }
}
