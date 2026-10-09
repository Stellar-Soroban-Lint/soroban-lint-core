# CLI reference

`soroban-lint` has two subcommands: `check` analyzes source, `rules` describes the registry. Everything below is the released `0.1.1` interface.

```
soroban-lint check <path> [--format text|json|sarif] [--config <file>]
                          [--fail-on error|warning|info|never] [--experimental]
soroban-lint rules [--format json|text]
```

## Subcommands

### `check <path>`

`<path>` is a file or a directory. Directories are walked with `walkdir`, honoring the `include` and `exclude` globs from the config.

| Flag | Values | Default | Effect |
|---|---|---|---|
| `--format` | `text`, `json`, `sarif` | `text` | Output format. |
| `--config` | path | `soroban-lint.toml` in the invocation root, if present | Config file to read. |
| `--fail-on` | `error`, `warning`, `info`, `never` | `warning` | Minimum severity that causes a non-zero exit. |
| `--experimental` | flag | off | Force the experimental rules on, overriding the config. |

`--experimental` is an override, not an override *of* a config default in both directions: it can only turn experimental rules on, never off. To disable a rule, set it to `off` in the config or use a [suppression directive](#suppression).

### `rules`

Prints the registry. `--format json` is what this documentation site is generated from, and what the playground calls at load time:

```bash
soroban-lint rules --format json
```

## Output formats

### `text`

One line per finding, `file:line:column: severity RULE (confidence): message`, with `help` indented beneath when present. Paths are relative to the invocation root with forward slashes. When stdout is not a terminal the run ends with a `N finding(s): …` summary.

### `json`

```json
{
  "version": "1",
  "diagnostics": [
    {
      "rule_id": "SL001",
      "severity": "error",
      "confidence": "medium",
      "message": "`set_balance` mutates state without an authorization check",
      "file": "sl001_missing_auth.rs",
      "start_line": 9,
      "start_column": 12,
      "end_line": 9,
      "end_column": 23,
      "help": "call `require_auth()` (or `require_auth_for_args`) for the authorizing address before the first mutation",
      "fix": null
    }
  ]
}
```

`fix` is human-readable suggestion text, not a machine-applicable edit. There is no `--fix` in v1.

### `sarif`

SARIF 2.1.0, with `tool.driver.name` of `soroban-lint`, an `informationUri` pointing at `soroban-lint-core`, and a `rules[]` array mirroring the registry. Each result carries a `partialFingerprints` entry keyed `sorobanLint/v1`, a hash of `rule_id + file + start_line`, so a GitHub alert can be tracked across runs. Locations use relative POSIX paths.

Write it to a file and upload it to code scanning — see [the Action page](/github-action#sarif).

## Configuration

`soroban-lint.toml` in the invocation root is discovered automatically. Pass `--config` to point elsewhere.

```toml
# Whether experimental rules run. Default false.
experimental = false

# Globs, in globset syntax, applied by the CLI's directory walk.
include = ["src/**/*.rs", "contracts/**/*.rs"]
exclude = ["target/**", "tests/**"]

[rules]
# Shorthand form: "off" | "error" | "warning" | "info"
SL006 = "off"

# Table form for explicit control
[rules.SL001]
enabled = true
severity = "error"
```

Semantics:

- A `stable` rule is **enabled by default**; an `experimental` rule is **disabled** unless `experimental = true` or `--experimental` is passed.
- `"off"` and `enabled = false` disable a rule regardless of its stability.
- `severity` overrides the rule's default severity for that run without changing the rule.
- Unknown rule ids in `[rules]` are an `SL000` **error** reported at the config file.
- Unknown top-level keys are an `SL000` **warning**, as a typo guard. This is why a misspelled key does not silently do nothing.

## Suppression

`syn` discards comments, so suppressions are read from raw source and matched against each diagnostic's `start_line`. Both directives are line comments beginning with `//`; leading whitespace is allowed.

```rust
// soroban-lint-ignore: SL005
env.storage().persistent().set(&k, &v);

// soroban-lint-ignore-next-line: SL001, SL002
let v = env.storage().instance().get(&k).unwrap();
```

| Directive | Effect |
|---|---|
| `// soroban-lint-ignore: SL001` | Suppresses findings whose `start_line` equals the comment's own line. |
| `// soroban-lint-ignore-next-line: SL001, SL002` | Suppresses findings whose `start_line` equals `comment_line + 1`. |

The grammar is deliberately narrow:

1. Directive keywords are lowercase and exact; rule ids are uppercase and exact.
2. Ids are comma-separated, surrounding whitespace ignored. At least one is required.
3. A directive applies **only** to its exact target line or lines. It never leaks. `ignore-next-line` targets exactly `N+1` — if that line is blank, or the finding is further down, nothing is suppressed. There is no "scan forward to the next statement".
4. There is no wildcard in v1. `all` and `*` are not accepted; a suppression must name each rule.
5. An **unknown rule id** produces an `SL000` warning at the comment line, so `SL01` is caught rather than silently ignored. The directive is otherwise discarded.
6. Suppressed findings are removed from the output entirely and do **not** affect the exit code.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Clean: no findings at or above `--fail-on`. |
| `1` | At least one finding at or above `--fail-on`. |
| `2` | Usage or internal error: a bad flag, an unreadable or unparseable config, an I/O failure, or a path that does not exist. |

```bash
soroban-lint check contracts/ || echo "exit $?"
```

Note what is **not** an exit code 2. A file that fails to parse yields an `SL000` diagnostic, not a usage error and never a panic:

```
bad.rs:1:12: error SL000 (high): failed to parse: cannot parse string into token stream
1 finding(s): 1 error(s), 0 warning(s), 0 info
```

That is exit `1`, because `SL000` is severity `error` and counts toward `--fail-on`. Analysis continues on the remaining files. This distinction is deliberate: a syntax error in one file should not hide findings in the rest of the repository, and it should not read as "the linter crashed".

## Determinism

After suppression, diagnostics are sorted by the total order `(file, start_line, start_column, rule_id)` and de-duplicated on `(rule_id, file, start_line, start_column)`. Output is byte-stable across runs, platforms, and output formats, so diffing two runs is meaningful and a CI baseline will not churn.

## What the tool does not do

soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

There is no `--fix`, no `--watch`, no diff mode, and no configuration file discovery above the invocation root. Those are v2 items, tracked as issues rather than described as absent.

## Next

- [Rules](/rules/) — what each rule detects and what it misses.
- [GitHub Action](/github-action) — running the same analysis in CI.
- [Architecture](/architecture/) — why the analysis is syntactic, and what that buys.
