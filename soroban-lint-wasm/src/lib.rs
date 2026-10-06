//! WebAssembly bindings for soroban-lint. Populated in Phase 1.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String {
    soroban_lint_core::version().to_string()
}
