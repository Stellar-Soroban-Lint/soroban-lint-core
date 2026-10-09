# Community

soroban-lint is developed in the open. Both channels below are active, and neither requires an introduction.

## Discord

[discord.gg/xZRZT6TpB](https://discord.gg/xZRZT6TpB)

The main channel. Good for working through a finding that looks like a false positive, a rule whose scope is unclear, and design discussion about what the linter should catch.

Attaching `soroban-lint --version` output, the command you ran, and the input that triggered the finding makes triage a single round trip instead of three.

## Telegram

[t.me/+MrTh9uraIS5jMjhk](https://t.me/+MrTh9uraIS5jMjhk)

Release announcements and short notices. This is where new versions are posted as they are cut.

## Where to file what

| You want to | Go to |
|---|---|
| Ask a question, discuss a finding | [Discord](https://discord.gg/xZRZT6TpB) |
| Be notified of releases | [Telegram](https://t.me/+MrTh9uraIS5jMjhk) |
| Report a bug or a false positive | An issue on the relevant repository — see the [contributing page](/contributing#issue-board) |
| Report a security vulnerability | The `SECURITY.md` in that repository. **Not** a public issue. |
| Track releases | [soroban-lint-core releases](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases) |
| See the linter in CI | [soroban-lint-portal#1](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/pull/1) (failing) and [#2](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/pull/2) (passing) |

A false-positive report with the version, the command, and the input is the single most useful contribution to this project. The two false positives found during benchmarking were fixed in the linter's code because someone reported and triaged them.

## The repositories

| Repository | What it is |
|---|---|
| [`soroban-lint-action`](https://github.com/Stellar-Soroban-Lint/soroban-lint-action) | The GitHub Action. |
| [`soroban-lint-portal`](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal) | The browser playground running the real linter as WebAssembly. |
| [`soroban-lint-core`](https://github.com/Stellar-Soroban-Lint/soroban-lint-core) | The engine, the CLI, the release binaries, **and this site**, which is published from this repository's Pages. Rule pages are generated from the released binary. |

Maintainer contact: use the contact address listed on the [core repository profile](https://github.com/Stellar-Soroban-Lint/soroban-lint-core).

## A note on what this tool is

Anything asked in either channel will eventually come back to what soroban-lint can and cannot see, so it is worth saying plainly here too:

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

If you ask "is this contract safe?" the answer is no, and this tool cannot tell you either way. If you ask "does this contract have a public function that mutates storage without an authorization check?" that is a question this tool answers well, and the [rule pages](/rules/) say exactly how well.

## License

MIT OR Apache-2.0. Contributing means agreeing to those terms.
