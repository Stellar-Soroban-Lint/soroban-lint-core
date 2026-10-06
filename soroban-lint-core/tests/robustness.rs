//! Robustness: the parse-and-lint path must never panic, on any input.

use proptest::prelude::*;
use soroban_lint_core::{lint_source, Config, Registry};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn never_panics_on_arbitrary_unicode(s in "\\PC*") {
        let cfg = Config { experimental: true, ..Config::default() };
        let _ = lint_source("f.rs", &s, &cfg, &Registry::default_set());
    }

    #[test]
    fn never_panics_on_rustish_input(s in "[a-zA-Z0-9_#!\\[\\](){};:,\\.<>+\\-*/&|=\"' \n]{0,400}") {
        let cfg = Config { experimental: true, ..Config::default() };
        let _ = lint_source("f.rs", &s, &cfg, &Registry::default_set());
    }
}
