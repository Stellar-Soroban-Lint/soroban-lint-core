---
layout: home
title: soroban-lint
titleTemplate: syntactic static analysis for Soroban

hero:
  name: soroban-lint
  text: Syntactic static analysis for Soroban contracts
  tagline: Eight rules over the Rust AST, in a terminal, in CI, and in your browser.
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started
    - theme: alt
      text: Browse the rules
      link: /rules/
    - theme: alt
      text: Use in CI
      link: /github-action
  # No hero image: the organisation ships no logo or screenshot asset, and
  # pointing at an avatar or a stock illustration would be inventing artwork
  # rather than documenting the project.
  image:
    alt: ""
---

<div class="sl-features">

## Eight rules, three surfaces

The same engine runs in three places: a Rust CLI, a GitHub Action, and a WebAssembly build in the browser. They share one JSON contract, so a finding in CI is the same finding in the playground.

<div class="sl-grid">

<div class="sl-card">

### Terminal

`cargo install soroban-lint-cli`, then lint a path. Text, JSON, or SARIF 2.1.0 output, three exit codes, and a config file with per-rule severity overrides.

[CLI reference](/cli)

</div>

<div class="sl-card">

### CI

One step. The Action downloads a prebuilt binary, verifies its SHA-256 against the digest published beside the release, and reports findings as workflow annotations.

[GitHub Action](/github-action)

</div>

<div class="sl-card">

### Browser

`soroban-lint-core` compiled to WebAssembly and vendored into the portal. No server, no reimplementation, and no source leaving the tab.

[Playground](/playground)

</div>

</div>

## What it does, and what it cannot do

soroban-lint reads Soroban contract source through the Rust AST and flags four families of pattern: missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions.

Those are the limits of the technique, stated here rather than in a footnote:

- **No macro expansion.** A macro body is an opaque token stream. Rules see a macro's name and top-level text, never the expressions inside it.
- **No type resolution.** Type evidence is syntactic: literals, explicit annotations, and `as` casts. A value loaded from storage and added to another carries no evidence and is not flagged.
- **No cross-file analysis.** SL001 follows a helper call exactly one level deep, and only within the same file. Nothing else follows calls.

Both failure modes are real and both are expected: the linter can miss genuine issues (**false negatives**) and can flag safe code (**false positives**). On the pinned benchmark corpus it produced 103 findings across 129 files, and four of those were test-module findings it could not identify as tests.

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

If you need assurance rather than triage, read the findings, fix the code, and still get it reviewed.

## Where to go next

<div class="sl-grid">

<div class="sl-card">

### [Getting Started](/getting-started)

Install from crates.io or a checksummed release binary, and run the first check.

</div>

<div class="sl-card">

### [Rules](/rules/)

One page per rule, each generated from the released binary's own metadata.

</div>

<div class="sl-card">

### [Benchmarks](/benchmarks/)

The 103-finding corpus run, with per-rule triage and the findings that were fixed in code.

</div>

</div>
</div>
