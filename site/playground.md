# Playground

The [soroban-lint portal](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal) runs the real linter in your browser. There is no server component and no reimplementation of the rules.

## What it actually runs

`soroban-lint-core` — the same crate that produces the CLI and that the GitHub Action downloads — is compiled to WebAssembly with `wasm-pack`, optimized with `wasm-opt -Oz`, and vendored into the portal's `src/wasm/` directory. The bytes come from the published `v0.1.1` release asset, not from a local build, so the browser runs what a `cargo install` would give you.

There are two exports, and they mirror the CLI's JSON contract exactly:

| Export | Equivalent CLI command |
|---|---|
| `lintSource(path, source, experimental)` | `soroban-lint check <path> --format json` |
| `rulesJson()` | `soroban-lint rules --format json` |

Because the contracts are identical, the portal cannot drift away from the CLI quietly. The build of this documentation site is guarded by the same command — the [rule pages](/rules/) are generated from `rulesJson()`'s CLI equivalent, not hand-typed.

::: warning The v0.1.0 tag was a stub
The `soroban-lint-wasm-v0.1.0.tar.gz` asset predates the WebAssembly implementation and exports only `version()`. It is the `v0.1.1` asset that contains the real linter. If you vendor the WASM package yourself, take `v0.1.1` or later. This is recorded in the [changelog](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/blob/main/CHANGELOG.md).
:::

## The parity guarantee

A playground that reimplemented the rules would be a demonstration of a different program. This one is gated against the real thing.

`tests/unit/parity.test.ts` loads the vendored `.wasm`, runs `lintSource` over **every `.rs` file in the benchmark corpus**, runs the native CLI over the same files, and asserts the two agree on the whole `Diagnostic` object — not just rule ids, but message, severity, confidence, and span. A message or a column difference is a real divergence between what CI reports and what the browser shows, and it fails the build.

So: if the playground shows you a finding, the CLI would show it too, on the same line, for the same reason. And a stale or stubbed `.wasm` cannot pass CI.

Provenance for the vendored binary is recorded in the portal's `src/wasm/PROVENANCE.md`:

| | |
|---|---|
| Release | [`v0.1.1`](https://github.com/Stellar-Soroban-Lint/soroban-lint-core/releases/tag/v0.1.1) |
| Asset | `soroban-lint-wasm-v0.1.1.tar.gz` |
| Source commit | `9c665ace6c1b2494ea4ab9373064b699b2a88789` |
| Vendored `.wasm` size | 714,866 bytes |
| Vendored `.wasm` SHA-256 | <code class="sl-digest">bd2f09d0618aee3819223ee5a905b2864f0fdfaa3a9f089432b492dc8f589ecb</code> |

## Privacy

Source you paste into the editor is analyzed in the page. There is no server to send it to, and no request leaves the origin while linting: the Monaco editor distribution is self-hosted from `public/monaco/vs`, so the page runs under a strict `script-src 'self'` policy with no third-party fetch.

This is a property of the architecture, not a promise. There is no backend to leak to.

## Running it locally

```bash
git clone https://github.com/Stellar-Soroban-Lint/soroban-lint-portal
cd soroban-lint-portal
npm ci
npm run dev          # http://localhost:3000
```

`npm run dev` and `npm run build` run generators first. `npm run catalog` regenerates the portal's rule catalog from `soroban-lint rules --format json`, and `npm run monaco` copies Monaco's distribution out of `node_modules` so the editor is served from this origin.

```bash
npm run verify       # catalog + typecheck + lint + unit tests + production build
npm run e2e          # builds, serves, and drives the portal in Chromium
```

`npm run e2e` needs a browser: `npx playwright install chromium`. Set `E2E_BASE_URL=https://…` to run the same suite against a deployed instance, which is what makes the parity and accessibility assertions meaningful outside a developer's laptop.

## What the playground does not add

The portal adds **no analysis of its own**. It shows whatever the linter reports, including each rule's stated limitations. The samples it offers are the linter's own fixtures from `soroban-lint-core`, and each one's description names the rules the linter actually reports for it.

So the playground inherits every limit of the technique:

> soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

A clean run in the browser tab is exactly as meaningful as a clean run in your terminal, which is to say: it means these eight syntactic patterns were not found, and nothing more.

## Next

- [Rules](/rules/) — what each rule detects and what it misses.
- [Architecture](/architecture/) — how the WASM build stays in step with the CLI.
- [Benchmarks](/benchmarks/) — the corpus the parity test runs against.
