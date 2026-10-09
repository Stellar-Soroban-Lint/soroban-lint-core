# Getting Started

soroban-lint is a Rust CLI. It analyzes Soroban contract source syntactically, so it needs no toolchain, no SDK, and no network access to analyze a file.

## Install

### From crates.io

```bash
cargo install soroban-lint-cli
```

This builds from source and needs a Rust toolchain. The binary is `soroban-lint`.

### A prebuilt binary

Every release publishes archives for six platform targets, each with a sibling `.sha256` file:

| Target | Archive |
|---|---|
| `x86_64-unknown-linux-gnu` | `soroban-lint-v0.1.1-x86_64-unknown-linux-gnu.tar.gz` |
| `aarch64-unknown-linux-gnu` | `soroban-lint-v0.1.1-aarch64-unknown-linux-gnu.tar.gz` |
| `x86_64-apple-darwin` | `soroban-lint-v0.1.1-x86_64-apple-darwin.tar.gz` |
| `aarch64-apple-darwin` | `soroban-lint-v0.1.1-aarch64-apple-darwin.tar.gz` |
| `x86_64-pc-windows-msvc` | `soroban-lint-v0.1.1-x86_64-pc-windows-msvc.zip` |

Download from the [releases page](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/tag/v0.1.1), then verify the checksum **before** extracting anything:

```bash
curl -LO https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/download/v0.1.1/soroban-lint-v0.1.1-x86_64-unknown-linux-gnu.tar.gz
curl -LO https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/download/v0.1.1/soroban-lint-v0.1.1-x86_64-unknown-linux-gnu.tar.gz.sha256

sha256sum --check soroban-lint-v0.1.1-x86_64-unknown-linux-gnu.tar.gz.sha256
```

`--check` prints `OK` only when the archive matches the digest the release published. Verify before extracting, not after.

The expected digests for `v0.1.1`, as published:

| Archive | SHA-256 |
|---|---|
| `…-x86_64-unknown-linux-gnu.tar.gz` | <code class="sl-digest">3b438d69726f2623adb4b9e8daf20c43741a1b1f778392e3d38401f6b0dc41d5</code> |
| `…-aarch64-unknown-linux-gnu.tar.gz` | <code class="sl-digest">846724c31219638d79cbb247df625f4839883edd4716dc963d8300a840def75d</code> |
| `…-x86_64-apple-darwin.tar.gz` | <code class="sl-digest">22caf9ab1283dd804bcd79508ec3c290ce0fa5101157449a17afa6e576d1ddfa</code> |
| `…-aarch64-apple-darwin.tar.gz` | <code class="sl-digest">1a13c896babd573b1750fa73d952227bc381e5a27ddfc8a8bb66eb1c00f299e0</code> |
| `…-x86_64-pc-windows-msvc.zip` | <code class="sl-digest">d9a67c94605fb0cba778b1606b1628be3961c93f0348d825085e3eeb74a103fb</code> |

Digests are platform-specific. Pin the one for the platform you install on, and re-read it from the release page rather than from a blog post or a lockfile — every re-release rebuilds the archives and changes them.

On macOS, check a Gatekeeper-blocked binary with `xattr -d com.apple.quarantine` after verifying it.

## First run

Confirm the install:

```bash
soroban-lint --version
# soroban-lint 0.1.1
```

Point it at a contract directory:

```bash
soroban-lint check contracts/
```

On a clean contract:

```
no findings
```

Otherwise the text format is one line per finding — `file:line:column: severity RULE (confidence): message` — with any `help` text indented beneath it. This is real output from the released `0.1.1` binary against the linter's own `sl001_missing_auth.rs` fixture:

```
sl001_missing_auth.rs:9:12: error SL001 (medium): `set_balance` mutates state without an authorization check
  help: call `require_auth()` (or `require_auth_for_args`) for the authorizing address before the first mutation
1 finding(s): 1 error(s), 0 warning(s), 0 info
```

The final line is the summary. Paths are relative to the invocation root and use forward slashes, so they are the same on every platform.

Now run it the way CI would, with the experimental rules on and a non-zero exit reserved for real errors:

```bash
soroban-lint check contracts/ --experimental --fail-on error
```

Three rules are on by default ([SL001](/rules/SL001), [SL002](/rules/SL002), [SL008](/rules/SL008)). The other five are experimental and stay off until you pass `--experimental` or set `experimental = true`, because each is scoped to syntactic evidence that has not yet been validated on a large real-world corpus.

## Reading the output

The default text format is one line per finding, with the severity, rule id, name, and message. For anything other than reading by eye, use JSON or SARIF:

```bash
soroban-lint check contracts/ --format json
soroban-lint check contracts/ --format sarif --fail-on never > soroban-lint.sarif
```

Diagnostics are sorted by `(file, start_line, start_column, rule_id)` and de-duplicated, so output is byte-stable across runs and platforms. A diff of two runs on the same input is meaningful.

## What a finding is worth

soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

Read the finding, then read the contract. The [rule pages](/rules/) state each rule's limitation next to what it claims, and the [benchmarks](/benchmarks/) page shows what the rules actually found in 129 files of real code, including the two false positives that were fixed rather than explained away.

## Next

- [CLI reference](/cli) — every flag, the config schema, the suppression grammar, and the exit codes.
- [Rules](/rules/) — what each rule does and does not detect.
- [GitHub Action](/github-action) — the same analysis in CI, with SARIF upload.
- [Playground](/playground) — the real linter running in your browser.
