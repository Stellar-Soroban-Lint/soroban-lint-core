//! WebAssembly bindings for soroban-lint.
//!
//! The portal runs this module in the browser, so the exports deliberately
//! mirror the CLI's JSON contract: [`lint_source`] returns exactly what
//! `soroban-lint check --format json` prints, and [`rules_json`] returns
//! exactly what `soroban-lint rules --format json` prints.

use wasm_bindgen::prelude::*;

use soroban_lint_core::{lint_source, render, Config, Registry};

/// Install a panic hook so Rust panics surface as console errors instead of
/// `unreachable executed`. Runs automatically when the module is initialised.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// The crate version, e.g. `0.1.0`.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    soroban_lint_core::version().to_string()
}

/// Lint one source file and return the JSON diagnostics document.
///
/// `path` is only used to label findings, exactly as in the CLI. `experimental`
/// enables SL003-SL007.
#[wasm_bindgen(js_name = lintSource)]
#[must_use]
pub fn lint_source_json(path: &str, source: &str, experimental: bool) -> String {
    let config = Config {
        experimental,
        ..Config::default()
    };
    let diagnostics = lint_source(path, source, &config, &Registry::default_set());
    render::json(&diagnostics)
}

/// The rule catalog as JSON, matching `soroban-lint rules --format json`.
#[wasm_bindgen(js_name = rulesJson)]
#[must_use]
pub fn rules_json() -> String {
    render::rules_json(&Registry::default_set().metadata())
}
