# Security policy

## This tool aids review; it is not an audit

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

No output of this tool is a security guarantee. Treat every finding as a prompt
for human review, and every clean report as "no findings from this set of syntactic
checks", not "no vulnerabilities".

## Reporting a bug in the linter

If you find a **false negative** (a pattern we should flag but do not), a **false
positive**, a crash/panic, or an incorrect span, open an issue:

https://github.com/Stellar-Soroban-Lint/soroban-lint-core/issues

Include the rule id, a minimal source snippet, the command you ran, and the output.

## Reporting a vulnerability you find in a third-party contract

The maintainers of this project do **not** audit contracts and cannot triage
third-party vulnerabilities. Use the contract vendor's responsible-disclosure
channel or the platform's:

- Stellar/Soroban security resources: https://developers.stellar.org/docs/tools/developer-tools/security-tools
- Soroban Security Portal: https://stellarsecurityportal.com

Do not open a public issue on this repository containing a live exploit against a
deployed contract.

## A vulnerability in soroban-lint itself

If you find a way to make the linter or its CI integration cause harm (for example,
code execution via crafted input, or a checksum-verification bypass in the action),
report it privately by opening a GitHub security advisory on the relevant repository,
or by email to the maintainer listed on the repository profile.
