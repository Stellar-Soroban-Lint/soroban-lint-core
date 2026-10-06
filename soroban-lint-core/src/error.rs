//! Typed errors for `soroban-lint-core`.

use thiserror::Error;

/// Errors produced while loading or validating configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The TOML document could not be parsed.
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    /// A severity string was not one of the accepted values.
    #[error("invalid severity {0:?} (expected \"error\", \"warning\", \"info\", or \"off\")")]
    Severity(String),
}
