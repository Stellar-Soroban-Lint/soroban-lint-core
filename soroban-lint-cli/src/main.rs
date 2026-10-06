//! `soroban-lint` — command-line interface.
//!
//! Exit codes: `0` clean, `1` findings at or above `--fail-on`, `2` usage/internal error.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use globset::{Glob, GlobSet, GlobSetBuilder};
use soroban_lint_core::diagnostic::meta_diagnostic;
use soroban_lint_core::{cargo, lint_source, render, Config, Diagnostic, Registry, Severity};

#[derive(Parser)]
#[command(
    name = "soroban-lint",
    version,
    about = "Syntactic, per-file static analysis for Soroban smart contracts.\n\
             A clean report is not evidence a contract is secure; this tool is not an audit."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Lint a file or directory.
    Check(CheckArgs),
    /// List the registered rules.
    Rules(RulesArgs),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    Text,
    Json,
    Sarif,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum FailOn {
    Error,
    Warning,
    Info,
    Never,
}

impl FailOn {
    fn threshold(self) -> Option<Severity> {
        match self {
            Self::Error => Some(Severity::Error),
            Self::Warning => Some(Severity::Warning),
            Self::Info => Some(Severity::Info),
            Self::Never => None,
        }
    }
}

#[derive(Args)]
struct CheckArgs {
    /// File or directory to lint.
    path: PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    /// Path to a `soroban-lint.toml` file.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Minimum severity that causes a non-zero exit.
    #[arg(long, value_enum, default_value_t = FailOn::Warning)]
    fail_on: FailOn,
    /// Enable experimental rules.
    #[arg(long)]
    experimental: bool,
}

#[derive(Args)]
struct RulesArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Check(args) => check(&args),
        Command::Rules(args) => rules(&args),
    }
}

fn load_config(path: &Option<PathBuf>) -> Result<Config, String> {
    let file = match path {
        Some(p) => Some(p.clone()),
        None => {
            let default = PathBuf::from("soroban-lint.toml");
            default.exists().then_some(default)
        }
    };
    match file {
        None => Ok(Config::default()),
        Some(p) => {
            let text = fs::read_to_string(&p)
                .map_err(|e| format!("cannot read config {}: {e}", p.display()))?;
            Config::from_toml_str(&text).map_err(|e| format!("invalid config {}: {e}", p.display()))
        }
    }
}

fn walk(path: &Path, include: &Option<GlobSet>, exclude: &Option<GlobSet>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if path.is_file() {
        files.push(path.to_path_buf());
        return files;
    }
    for entry in walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        if p.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        let rel = p
            .strip_prefix(path)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        if let Some(inc) = include {
            if !inc.is_match(&rel) {
                continue;
            }
        }
        if let Some(exc) = exclude {
            if exc.is_match(&rel) {
                continue;
            }
        }
        files.push(p.to_path_buf());
    }
    files.sort();
    files
}

fn build_globs(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        if let Ok(g) = Glob::new(p) {
            b.add(g);
        }
    }
    b.build().ok()
}

fn check(args: &CheckArgs) -> ExitCode {
    if !args.path.exists() {
        eprintln!("error: path does not exist: {}", args.path.display());
        return ExitCode::from(2);
    }
    let mut config = match load_config(&args.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    if args.experimental {
        config = config.with_experimental(true);
    }

    let registry = Registry::default_set();
    let include = build_globs(&config.include);
    let exclude = build_globs(&config.exclude);
    let root = if args.path.is_dir() {
        args.path.clone()
    } else {
        args.path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
    };

    let mut diags: Vec<Diagnostic> = Vec::new();

    for key in &config.unknown_keys {
        diags.push(meta_diagnostic(
            "soroban-lint.toml",
            format!("unknown config key {key:?}"),
            1,
            1,
            Severity::Warning,
        ));
    }

    for file in walk(&args.path, &include, &exclude) {
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        match fs::read_to_string(&file) {
            Ok(src) => diags.extend(lint_source(&rel, &src, &config, &registry)),
            Err(e) => diags.push(meta_diagnostic(
                &rel,
                format!("cannot read file: {e}"),
                1,
                1,
                Severity::Error,
            )),
        }
    }

    // Project-level check: release profile overflow-checks (SL003).
    let manifest = root.join("Cargo.toml");
    if manifest.is_file() {
        if let Ok(doc) = fs::read_to_string(&manifest) {
            if let Some(d) = cargo::overflow_checks_diagnostic("Cargo.toml", &doc) {
                diags.push(d);
            }
        }
    }

    diags.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    diags.dedup_by(|a, b| a.dedup_key() == b.dedup_key());

    match args.format {
        Format::Text => {
            print!("{}", render::text(&diags));
            let errors = diags
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count();
            let warnings = diags
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count();
            let infos = diags
                .iter()
                .filter(|d| d.severity == Severity::Info)
                .count();
            if diags.is_empty() {
                println!("no findings");
            } else {
                println!(
                    "{} finding(s): {} error(s), {} warning(s), {} info",
                    diags.len(),
                    errors,
                    warnings,
                    infos
                );
            }
        }
        Format::Json => println!("{}", render::json(&diags)),
        Format::Sarif => println!("{}", render::sarif(&diags, &registry.metadata())),
    }

    match args.fail_on.threshold() {
        Some(threshold) if diags.iter().any(|d| d.severity >= threshold) => ExitCode::from(1),
        _ => ExitCode::SUCCESS,
    }
}

fn rules(args: &RulesArgs) -> ExitCode {
    let registry = Registry::default_set();
    let meta = registry.metadata();
    match args.format {
        Format::Json => println!("{}", render::rules_json(&meta)),
        _ => {
            for r in &meta {
                println!(
                    "{}  {}  [{} / {} / {}]\n    {}",
                    r.id,
                    r.name,
                    r.default_severity.as_str(),
                    r.default_confidence.as_str(),
                    r.stability.as_str(),
                    r.description,
                );
            }
        }
    }
    ExitCode::SUCCESS
}
