//! `soroban-lint.toml` schema and resolution.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::diagnostic::{RuleMeta, Severity, Stability};
use crate::error::ConfigError;

const KNOWN_TOP_LEVEL: [&str; 4] = ["experimental", "include", "exclude", "rules"];

/// Top-level configuration.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Enable `experimental` rules globally.
    pub experimental: bool,
    /// Include globs applied by the CLI walker.
    pub include: Vec<String>,
    /// Exclude globs applied by the CLI walker.
    pub exclude: Vec<String>,
    /// Per-rule settings keyed by rule id.
    pub rules: BTreeMap<String, RuleSettings>,
    /// Unknown top-level keys found while parsing (reported as `SL000`).
    pub unknown_keys: Vec<String>,
}

/// A rule's configuration entry: shorthand string or table.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RuleSettings {
    /// `SL006 = "off"` or `SL001 = "error"`.
    Shorthand(String),
    /// `[rules.SL001] enabled = true, severity = "error"`.
    Table(RuleTable),
}

/// Table form of a per-rule setting.
#[derive(Debug, Clone, Deserialize)]
pub struct RuleTable {
    /// Explicit enable/disable.
    pub enabled: Option<bool>,
    /// Severity override, or `"off"`.
    pub severity: Option<String>,
}

impl RuleSettings {
    /// Resolve to `(enabled, severity)`. `None` means "unspecified".
    fn resolve(&self) -> Result<(Option<bool>, Option<Severity>), ConfigError> {
        match self {
            Self::Shorthand(s) => {
                if s == "off" {
                    Ok((Some(false), None))
                } else {
                    let sev = Severity::parse(s).ok_or_else(|| ConfigError::Severity(s.clone()))?;
                    Ok((Some(true), Some(sev)))
                }
            }
            Self::Table(t) => {
                let sev = match t.severity.as_deref() {
                    None => None,
                    Some("off") => return Ok((Some(false), None)),
                    Some(s) => Some(
                        Severity::parse(s).ok_or_else(|| ConfigError::Severity(s.to_string()))?,
                    ),
                };
                Ok((t.enabled, sev))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    experimental: bool,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    rules: BTreeMap<String, RuleSettings>,
}

impl Config {
    /// Parse and validate a `soroban-lint.toml` document.
    ///
    /// # Errors
    /// Returns [`ConfigError`] on TOML parse failure or an invalid severity string.
    pub fn from_toml_str(s: &str) -> Result<Config, ConfigError> {
        let value: toml::Value = toml::from_str(s)?;
        let mut unknown_keys = Vec::new();
        if let toml::Value::Table(t) = &value {
            for k in t.keys() {
                if !KNOWN_TOP_LEVEL.contains(&k.as_str()) {
                    unknown_keys.push(k.clone());
                }
            }
        }
        let raw: RawConfig = toml::from_str(s)?;
        // Validate every rule severity eagerly so errors surface at load time.
        for settings in raw.rules.values() {
            let _ = settings.resolve()?;
        }
        Ok(Config {
            experimental: raw.experimental,
            include: raw.include,
            exclude: raw.exclude,
            rules: raw.rules,
            unknown_keys,
        })
    }

    /// Force experimental rules on (CLI `--experimental`).
    #[must_use]
    pub fn with_experimental(mut self, on: bool) -> Self {
        self.experimental = on;
        self
    }

    /// Whether a rule should run under this configuration.
    #[must_use]
    pub fn is_enabled(&self, meta: &RuleMeta) -> bool {
        if let Some(s) = self.rules.get(meta.id) {
            if let Ok((enabled, severity)) = s.resolve() {
                match (enabled, severity) {
                    (Some(enabled), _) => return enabled,
                    (None, Some(_)) => return true,
                    (None, None) => {}
                }
            }
        }
        meta.stability == Stability::Stable || self.experimental
    }

    /// Effective severity for a rule, or `None` if it is disabled.
    #[must_use]
    pub fn effective_severity(&self, meta: &RuleMeta) -> Option<Severity> {
        if !self.is_enabled(meta) {
            return None;
        }
        if let Some(s) = self.rules.get(meta.id) {
            if let Ok((_, Some(sev))) = s.resolve() {
                return Some(sev);
            }
        }
        Some(meta.default_severity)
    }
}
