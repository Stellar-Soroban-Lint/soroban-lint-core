//! Rule and suppression tests. Every rule has at least one true-positive fixture.

use soroban_lint_core::{lint_source, Config, Diagnostic, Registry};

fn cfg() -> Config {
    Config {
        experimental: true,
        ..Config::default()
    }
}

fn lint(path: &str, src: &str) -> Vec<Diagnostic> {
    lint_source(path, src, &cfg(), &Registry::default_set())
}

fn has(d: &[Diagnostic], id: &str) -> bool {
    d.iter().any(|x| x.rule_id == id)
}

macro_rules! positive {
    ($name:ident, $rule:literal, $file:literal) => {
        #[test]
        fn $name() {
            let src = include_str!(concat!("fixtures/vulnerable/", $file));
            let d = lint(concat!("tests/fixtures/vulnerable/", $file), src);
            assert!(
                has(&d, $rule),
                "expected {} in {}; got {d:#?}",
                $rule,
                $file
            );
        }
    };
}

macro_rules! negative {
    ($name:ident, $rule:literal, $file:literal) => {
        #[test]
        fn $name() {
            let src = include_str!(concat!("fixtures/safe/", $file));
            let d = lint(concat!("tests/fixtures/safe/", $file), src);
            assert!(
                !has(&d, $rule),
                "unexpected {} in {}; got {d:#?}",
                $rule,
                $file
            );
        }
    };
}

// SL001 — missing require_auth
positive!(sl001_true_positive, "SL001", "sl001_missing_auth.rs");
negative!(sl001_with_auth_ok, "SL001", "sl001_with_auth.rs");
negative!(sl001_auth_in_helper_ok, "SL001", "sl001_auth_in_helper.rs");
negative!(
    sl001_admin_from_storage_ok,
    "SL001",
    "sl001_admin_from_storage.rs"
);
negative!(
    sl001_auth_in_free_fn_ok,
    "SL001",
    "sl001_auth_in_free_fn.rs"
);
// A non-`pub` method of a `#[contractimpl]` block is not a contract entry point.
negative!(sl001_private_method_ok, "SL001", "sl001_private_method.rs");
// Auth living only in `#[cfg(test)]` code is not production authorization.
positive!(
    sl001_test_helper_is_not_auth,
    "SL001",
    "sl001_test_helper_not_auth.rs"
);

// SL002 — panic hazards
positive!(sl002_true_positive, "SL002", "sl002_panics.rs");
negative!(
    sl002_panic_with_error_ok,
    "SL002",
    "sl002_panic_with_error.rs"
);
// A non-`pub` method of a `#[contractimpl]` block is not a contract entry point.
negative!(sl002_private_method_ok, "SL002", "sl002_private_method.rs");

// SL003 — unchecked arithmetic
positive!(sl003_true_positive, "SL003", "sl003_arith.rs");
negative!(sl003_checked_ok, "SL003", "sl003_checked.rs");

// SL004 — unbounded storage growth
positive!(sl004_true_positive, "SL004", "sl004_growth.rs");
negative!(sl004_bounded_ok, "SL004", "sl004_bounded.rs");

// SL005 — missing TTL extension
positive!(sl005_true_positive, "SL005", "sl005_ttl.rs");
negative!(sl005_extend_ttl_ok, "SL005", "sl005_extend_ttl.rs");

// SL006 — questionable storage type
positive!(sl006_true_positive, "SL006", "sl006_temp.rs");
negative!(sl006_persistent_ok, "SL006", "sl006_persistent.rs");

// SL007 — unprotected initializer
positive!(sl007_true_positive, "SL007", "sl007_init.rs");
negative!(sl007_guarded_ok, "SL007", "sl007_guarded.rs");

// SL008 — unsafe / no_std
positive!(sl008_true_positive, "SL008", "sl008_unsafe.rs");
negative!(sl008_clean_ok, "SL008", "sl008_clean.rs");

const HEADER: &str = "#![no_std]\nuse soroban_sdk::{contract, contractimpl, Env};\n\
#[contract]\npub struct C;\n#[contractimpl]\nimpl C {\n";

fn wrap(body: &str) -> String {
    format!("{HEADER}{body}}}\n")
}

#[test]
fn suppression_same_line() {
    let src = wrap(
        "    pub fn get(env: Env, key: u32) -> u32 {\n\
         \x20       let v: u32 = env.storage().instance().get(&key).unwrap(); // soroban-lint-ignore: SL002\n\
         \x20       v\n    }\n",
    );
    let d = lint("same_line.rs", &src);
    assert!(!has(&d, "SL002"), "same-line suppression failed: {d:#?}");
}

#[test]
fn suppression_next_line() {
    let src = wrap(
        "    pub fn get(env: Env, key: u32) -> u32 {\n\
         \x20       // soroban-lint-ignore-next-line: SL002\n\
         \x20       let v: u32 = env.storage().instance().get(&key).unwrap();\n\
         \x20       v\n    }\n",
    );
    let d = lint("next_line.rs", &src);
    assert!(!has(&d, "SL002"), "next-line suppression failed: {d:#?}");
}

#[test]
fn suppression_multiple_ids() {
    let src = wrap(
        "    pub fn get(env: Env, key: u32) -> u32 {\n\
         \x20       // soroban-lint-ignore-next-line: SL001, SL002\n\
         \x20       let v: u32 = env.storage().instance().get(&key).unwrap();\n\
         \x20       v\n    }\n",
    );
    let d = lint("multi.rs", &src);
    assert!(!has(&d, "SL002"), "multi-id suppression failed: {d:#?}");
}

#[test]
fn suppression_unknown_id_warns() {
    let src = wrap(
        "    pub fn get(env: Env, key: u32) -> u32 {\n\
         \x20       // soroban-lint-ignore-next-line: SL999\n\
         \x20       let v: u32 = env.storage().instance().get(&key).unwrap();\n\
         \x20       v\n    }\n",
    );
    let d = lint("unknown.rs", &src);
    assert!(
        has(&d, "SL000"),
        "expected SL000 for unknown suppression id: {d:#?}"
    );
}

#[test]
fn suppression_does_not_leak_across_rules() {
    // Directive names SL001 on a line whose only finding is SL002; SL002 must survive.
    let src = wrap(
        "    pub fn get(env: Env, key: u32) -> u32 {\n\
         \x20       // soroban-lint-ignore-next-line: SL001\n\
         \x20       let v: u32 = env.storage().instance().get(&key).unwrap();\n\
         \x20       v\n    }\n",
    );
    let d = lint("leak.rs", &src);
    assert!(
        has(&d, "SL002"),
        "SL002 must not be suppressed by an SL001 directive: {d:#?}"
    );
}

#[test]
fn parse_failure_is_a_diagnostic_not_a_panic() {
    let d = lint("bad.rs", "fn (");
    assert!(
        has(&d, "SL000"),
        "expected SL000 for a parse failure: {d:#?}"
    );
}

#[test]
fn empty_file_is_clean() {
    assert!(lint("empty.rs", "").is_empty());
}

#[test]
fn invalid_rust_never_panics() {
    for src in ["{", "impl", "#[contractimpl] impl C { fn", "f("] {
        let _ = lint("invalid.rs", src);
    }
}
