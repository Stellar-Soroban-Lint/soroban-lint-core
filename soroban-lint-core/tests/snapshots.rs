//! Snapshot tests for the text, JSON, and SARIF renderers.

use soroban_lint_core::{lint_source, render, Config, Diagnostic, Registry};

const SRC: &str = r#"#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn set_balance(env: Env, addr: Address, amount: i128) -> i128 {
        let current: i128 = env.storage().persistent().get(&addr).unwrap();
        let next = current + amount;
        env.storage().persistent().set(&addr, &next);
        next
    }
}
"#;

fn run() -> Vec<Diagnostic> {
    let cfg = Config {
        experimental: true,
        ..Config::default()
    };
    lint_source("token.rs", SRC, &cfg, &Registry::default_set())
}

#[test]
fn text_output_snapshot() {
    insta::assert_snapshot!(render::text(&run()));
}

#[test]
fn json_output_snapshot() {
    insta::assert_snapshot!(render::json(&run()));
}

#[test]
fn sarif_output_snapshot() {
    let reg = Registry::default_set();
    insta::assert_snapshot!(render::sarif(&run(), &reg.metadata()));
}

#[test]
fn rules_metadata_snapshot() {
    insta::assert_snapshot!(render::rules_json(&Registry::default_set().metadata()));
}
