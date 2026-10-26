//! Walker tests for the `#[cfg(test)]` module skip.
//!
//! `src/test.rs` is included from its parent as `#[cfg(test)] mod test;`, so the
//! attribute is not in the file and a per-file analyzer cannot see it. The walker
//! therefore infers it from the path — but only for a directory walk, and only
//! when the configuration has not opted back in. These tests pin all three
//! behaviours, because each one is a plausible way to break the skip.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// A contract function that mutates storage without an authorization check, so
/// it produces exactly one SL001 finding wherever it is analyzed.
const CONTRACT: &str = r#"use soroban_sdk::{contractimpl, Env, Symbol};

pub struct Counter;

#[contractimpl]
impl Counter {
    pub fn set(env: Env, key: Symbol, value: u32) {
        env.storage().persistent().set(&key, &value);
    }
}
"#;

/// A throwaway crate layout: `src/lib.rs` (shipping) and `src/test.rs` (a test
/// module), plus a config file. Removed on drop.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("soroban-lint-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).expect("create temp src directory");
        fs::write(root.join("src/lib.rs"), CONTRACT).expect("write lib.rs");
        fs::write(root.join("src/test.rs"), CONTRACT).expect("write test.rs");
        Self { root }
    }

    fn write(&self, rel: &str, body: &str) {
        let path = self.root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, body).expect("write file");
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Run the CLI, expecting success, and return the parsed JSON and the stderr.
fn lint(args: &[&str], cwd: Option<&Path>) -> (Value, String) {
    let mut command = assert_cmd::Command::cargo_bin("soroban-lint").expect("binary is built");
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    let output = command.output().expect("the CLI runs");
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    (
        serde_json::from_str(&stdout).expect("stdout is JSON"),
        stderr,
    )
}

fn findings(json: &Value) -> &Vec<Value> {
    json["diagnostics"]
        .as_array()
        .expect("diagnostics is an array")
}

fn check_directory(dir: &Path) -> (Value, String) {
    lint(
        &[
            "check",
            dir.to_str().expect("temp path is UTF-8"),
            "--format",
            "json",
            "--fail-on",
            "never",
        ],
        None,
    )
}

#[test]
fn a_directory_walk_skips_cfg_test_modules() {
    let tree = TempTree::new("walk-skips");
    let (json, stderr) = check_directory(&tree.root);

    let found = findings(&json);
    assert_eq!(found.len(), 1, "only the shipping file is analyzed: {json}");
    assert_eq!(found[0]["file"], "src/lib.rs");
    assert!(
        stderr.contains("skipped 1 test module file"),
        "the skip is reported rather than silent: {stderr}"
    );
}

#[test]
fn an_explicitly_named_test_module_is_still_analyzed() {
    let tree = TempTree::new("explicit-target");
    let target = tree.root.join("src/test.rs");
    let (json, stderr) = lint(
        &[
            "check",
            target.to_str().expect("temp path is UTF-8"),
            "--format",
            "json",
            "--fail-on",
            "never",
        ],
        None,
    );

    let found = findings(&json);
    assert_eq!(found.len(), 1, "a named file is always analyzed: {json}");
    assert!(
        !stderr.contains("skipped"),
        "no skip notice for a single-file target: {stderr}"
    );
}

#[test]
fn include_tests_restores_analysis_of_test_modules() {
    let tree = TempTree::new("include-tests");
    tree.write("soroban-lint.toml", "include-tests = true\n");

    let (json, stderr) = lint(
        &["check", ".", "--format", "json", "--fail-on", "never"],
        Some(&tree.root),
    );

    let found = findings(&json);
    assert_eq!(found.len(), 2, "both files are analyzed: {json}");
    assert!(
        !stderr.contains("skipped"),
        "nothing was skipped, so nothing is reported: {stderr}"
    );
}

#[test]
fn the_skip_is_scoped_to_files_under_src() {
    let tree = TempTree::new("not-under-src");
    tree.write("examples/test.rs", CONTRACT);

    let (json, _stderr) = check_directory(&tree.root);

    let found = findings(&json);
    assert_eq!(
        found.len(),
        2,
        "only a `src` test module is inferred to be cfg(test): {json}"
    );
    assert!(found.iter().any(|d| d["file"] == "src/lib.rs"));
    assert!(found.iter().any(|d| d["file"] == "examples/test.rs"));
}
