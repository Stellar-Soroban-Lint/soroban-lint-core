# Contributing

Thanks for helping. This is a security tool, so precision matters more than volume.

Keep this statement of scope in mind in every change, issue, and review:

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

## Ground rules

- **No overclaiming.** Do not describe a rule as detecting "all" of anything, and do
  not present a limitation as a footnote. State the limitation in the same paragraph
  as the capability claim.
- **Every rule needs a true-positive fixture** plus a safe negative. See
  `docs/WRITING_RULES.md`.
- **Do not weaken a test to make it pass.** Fix the cause.
- **Rule ids are permanent.** Never reuse or renumber.

## Setup

```bash
git clone https://github.com/Stellar-Soroban-Lint/soroban-lint-core
cd soroban-lint-core
cargo build --workspace
```

## Before opening a PR

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

All three must be clean. CI runs the same, plus a `wasm32-unknown-unknown` build and
an MSRV build.

## Commits

Conventional commits: `feat(rules): …`, `fix(cli): …`, `docs: …`. One logical change per
commit. Commit your fixtures and snapshots with the rule.

## Adding a rule

See `docs/WRITING_RULES.md`. Reserve the next free `SLNNN` id.

## Preparing a release

See [`docs/RELEASING.md`](docs/RELEASING.md) for the release pull request checklist,
including the docs site's pinned CLI version.

## Branch protection

`main` is protected (pull request + one review + required status checks). Don't push
directly unless you are the bypass actor.

## License

By contributing you agree your contribution is licensed under the repository's
`MIT OR Apache-2.0` terms.
