# Contributing

soroban-lint is a security tool, so precision matters more than volume. A rule that fires often and is right half the time trains people to ignore the output.

Keep this statement of scope in mind in every change, issue, and review:

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

## Ground rules

- **No overclaiming.** Do not describe a rule as detecting "all" of anything, and do not file a limitation as a footnote. State the limitation in the same paragraph as the capability claim — the rule pages in this site are laid out that way for the same reason.
- **Every rule needs a true-positive fixture**, plus a safe negative, at least one tricky negative, and a documented false-negative fixture proving the stated limitation is real.
- **Do not weaken a test to make it pass.** Fix the cause. The two false positives found during benchmarking were fixed in the linter's code, not reclassified in the benchmark table.
- **Rule ids are permanent.** Never reuse or renumber one. A retired id stays reserved.

## Setup

```bash
git clone https://github.com/Stellar-Soroban-Lint/soroban-lint-core
cd soroban-lint-core
cargo build --workspace
```

Before opening a pull request:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

All three must be clean. CI runs the same, plus a `wasm32-unknown-unknown` build and an MSRV build.

### Writing a rule

A compile-checked template lives in the crate at `src/rules/mod.rs` in the `#[cfg(test)] mod template` module. It is compiled by `cargo test`, so it cannot rot.

The steps, in order:

1. **Reserve an id.** `SL001`–`SL008` are taken and `SL000` is reserved for meta diagnostics. Take the next free number; never renumber.
2. **Copy the template** into `src/rules/slNNN.rs` and register it.
3. **Fill in `RuleMeta`.** `rationale` is why the pattern matters; `limitations` is what the rule does not see and its dominant false-negative mode. Both are required and both are shown to users — a rule with an empty `limitations` is not finished.
4. **Write the fixtures**, in this order:
   - `tests/fixtures/vulnerable/` — a true positive that must be reported.
   - `tests/fixtures/safe/` — the correct shape that must not be.
   - A tricky negative — the near-miss that has caught people out before. For [SL001](/rules/SL001) that was auth in a free function rather than a method, and a `#[cfg(test)]` helper that must *not* count as an auth helper.
   - A documented false-negative fixture, proving the limitation you wrote in `limitations` is real rather than hypothetical.
5. **Never panic.** No rule may panic on any input. Parse failures surface as `SL000`.
6. **Update the docs.** Add human-written examples and guidance to `site/rules-prose/SLNNN.md`. The site generator merges this with metadata from the pinned release and fails if the release does not contain the rule.

Fixtures compile against the pinned SDK in a dedicated CI job, so "vulnerable" and "safe" samples are provably valid Soroban code rather than plausible-looking Rust.

Promoting a rule from `experimental` to `stable` is a separate decision, not part of adding it. See the [promotion policy](/benchmarks/#promotion-decision).

## Reporting a false positive

False-positive reports are the most valuable issue anyone can file, and the project has a template for them because the alternative is a round trip per report.

Include:

- the exact `soroban-lint --version` output,
- the rule id,
- the minimal input, or the command and file you ran,
- what it reported, and what you expected instead.

Then the triage path is: confirm it reproduces on the pinned release → classify it as a false positive (fix the code), arguable (document it), or a false negative (a rule limitation) → land a fixture pair with any code fix → update the corpus counts if the fix moves them.

If you are unsure whether something is a false positive or a genuine finding, open the issue anyway. Both the slug and the test-file limitation described in [Benchmarks](/benchmarks/#the-test-file-limitation) are worth reporting.

## Issue board

The repositories carry seeded, triaged issues. Each body states the problem, why it matters, and acceptance criteria, so you can pick one up without asking a question first.

| Repository | Issues | What lives there |
|---|---|---|
| [`soroban-lint-core`](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/issues) | 5 | Rule work and CLI ergonomics: the false-positive template, shell completions, skipping test modules, an `SL009` for hardcoded addresses, and baseline files. |
| [`soroban-lint-action`](https://github.com/Stellar-Soroban-Lint/soroban-lint-action/issues) | 5 | CI integration: checksum-mismatch tests, a workflow cookbook, quoting-safe `args`, changed-files-only input, and a Docker variant. |
| [`soroban-lint-portal`](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/issues) | 5 | Playground: catalog search, Open Graph and SEO metadata, downloading diagnostics, keyboard accessibility, and a multi-file workspace. |

Two of these are the most valuable contributions available, because they fix a limitation this project has already confessed to in writing:

- **Skipping test modules.** Files named `test.rs` that are really `#[cfg(test)]` modules produce 2 SL002 and 2 SL005 findings on the pinned corpus. The fix skips such files during a directory walk while still honoring an explicitly named single-file target.
- **The false-positive template and triage guide.** Two real false positives were caught during benchmarking, but only because someone triaged them by hand.

## Pull request checklist

- [ ] `cargo fmt --all --check` clean
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` clean
- [ ] `cargo test --workspace` passes
- [ ] Fixtures committed alongside the rule, including the tricky negative and the false-negative fixture
- [ ] No existing test weakened; new behaviour has a test
- [ ] `RuleMeta::limitations` written, and it names the dominant false-negative mode
- [ ] Corpus counts in `docs/BENCHMARKS.md` updated if the change moves them
- [ ] Docs note added to `scripts/rule-notes.json`
- [ ] Conventional commit: `feat(rules):`, `fix(cli):`, `docs:`, `build(ci):`
- [ ] One logical change per commit

`main` is protected: pull request, one review, required status checks.

## Working on this site

```bash
git clone https://github.com/Stellar-Soroban-Lint/soroban-lint-core
cd soroban-lint-core/site
npm ci
npm run dev
```

The [rule pages](/rules/) are generated and must not be hand-edited. `scripts/gen-rules.mjs` reads `soroban-lint rules --format json` from the release pinned in `SOROBAN_LINT_VERSION` and merges in prose from `rules-prose/`; `npm run build` regenerates them. Editing a generated page directly is overwritten by the next build.

## Community

Questions, findings to discuss, and release notes go to [Discord](https://discord.gg/xZRZT6TpB) and [Telegram](https://t.me/+MrTh9uraIS5jMjhk). The [Community page](/community) has the details.

By contributing you agree your contribution is licensed under the repository's `MIT OR Apache-2.0` terms.

## Security

Do not open a public issue containing a live exploit against a deployed contract. Each repository has a `SECURITY.md` with the reporting path.

## Next

- [Architecture](/architecture/) — the guarantees a change must not break.
- [Benchmarks](/benchmarks/) — the corpus standard a rule is measured against.
