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
| `x86_64-unknown-linux-gnu` | `soroban-lint-v0.2.0-x86_64-unknown-linux-gnu.tar.gz` |
| `aarch64-unknown-linux-gnu` | `soroban-lint-v0.2.0-aarch64-unknown-linux-gnu.tar.gz` |
| `x86_64-apple-darwin` | `soroban-lint-v0.2.0-x86_64-apple-darwin.tar.gz` |
| `aarch64-apple-darwin` | `soroban-lint-v0.2.0-aarch64-apple-darwin.tar.gz` |
| `x86_64-pc-windows-msvc` | `soroban-lint-v0.2.0-x86_64-pc-windows-msvc.zip` |

Download from the [releases page](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/tag/v0.2.0), then verify the checksum **before** extracting anything:

```bash
curl -LO https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/download/v0.2.0/soroban-lint-v0.2.0-x86_64-unknown-linux-gnu.tar.gz
curl -LO https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/download/v0.2.0/soroban-lint-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256

sha256sum --check soroban-lint-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```

`--check` prints `OK` only when the archive matches the digest the release published. Verify before extracting, not after.

The expected digests for `v0.2.0`, as published:

| Archive | SHA-256 |
|---|---|
| `…-x86_64-unknown-linux-gnu.tar.gz` | <code class="sl-digest">4600bc00a191eba5b32408b913d042915f5a449300411c2cf8ff89ea09255d5f</code> |
| `…-aarch64-unknown-linux-gnu.tar.gz` | <code class="sl-digest">10f2a7ce2e20d8dfd170e2a082408ceb6994453933047850579b6e20e8990850</code> |
| `…-x86_64-apple-darwin.tar.gz` | <code class="sl-digest">e7d47dfd9566f11d33dfca5259a54ee497f426364d5fc8ad3251bbe7a2a4db18</code> |
| `…-aarch64-apple-darwin.tar.gz` | <code class="sl-digest">c6ae199383a6879ea516605e3138c268796f702e8ab3f1ff939e3921eeb6a24c</code> |
| `…-x86_64-pc-windows-msvc.zip` | <code class="sl-digest">5a75b8941833d5aa8525fd90de66c3cbe892486be648979e30acdeee1b9de3c6</code> |

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
