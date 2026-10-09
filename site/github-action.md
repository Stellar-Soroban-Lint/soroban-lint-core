# GitHub Action

The Action lints a repository with the released `soroban-lint` CLI. It downloads a prebuilt binary for the runner's platform from a `soroban-lint-core` release, **verifies the archive's SHA-256 against the digest published beside the release asset**, and only then runs it. It never builds from source and never uses Docker.

```yaml
name: lint

on:
  pull_request:

permissions:
  contents: read

jobs:
  soroban-lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: Stellar-Soroban-Lint/soroban-lint-action@v0
        with:
          path: contracts
          fail-on: error
```

## What it does

Findings become workflow annotations on the changed files, the job summary counts them, and the step fails when a finding meets the `fail-on` threshold. On a pull request it also posts a summary comment and updates that comment in place on later pushes, so the discussion does not accumulate one comment per commit.

## Permissions

| Permission | Needed for | Granted by |
|---|---|---|
| `contents: read` | Downloading the release asset. | Always required. |
| `pull-requests: write` | Posting and updating the summary comment. | Required only if `comment` is on. |

```yaml
permissions:
  contents: read
  pull-requests: write
```

SARIF upload additionally needs `security-events: write`, granted by `github/codeql-action/upload-sarif`, not by this Action.

### Fork pull requests

A pull request from a fork runs with a read-only `GITHUB_TOKEN`, so writing a comment or a check fails. The Action treats that as expected rather than as an error: it logs a warning and continues. Annotations and the job summary still work, which is why a fork PR still shows its findings. The summary comment and any SARIF upload are skipped.

If you fork a repository and open a PR against the upstream, you will see the lint findings and no comment. That is the designed behaviour, not a broken workflow.

## Pinning

`version: latest` is convenient and not reproducible. It resolves to whatever is newest at run time, so two runs on the same commit can disagree.

```yaml
      - uses: Stellar-Soroban-Lint/soroban-lint-action@v0
        with:
          version: v0.1.2
          path: contracts
          # SHA-256 of soroban-lint-v0.1.2-x86_64-unknown-linux-gnu.tar.gz
          checksum: c2b2bcf5f01d2591b44a411b7c6724b4cc0c00671a8062e6a82deb26cea89538
```

Pin both the tag and the digest for your runner platform. The digests published with `v0.1.2`:

| Platform | SHA-256 |
|---|---|
| `x86_64-unknown-linux-gnu` | <code class="sl-digest">c2b2bcf5f01d2591b44a411b7c6724b4cc0c00671a8062e6a82deb26cea89538</code> |
| `aarch64-unknown-linux-gnu` | <code class="sl-digest">14e8d240a2f1b3143daaba60f80927a8831bdf2747dacf7d27262f77095ee743</code> |
| `x86_64-apple-darwin` | <code class="sl-digest">9b4127e0e85e3df1b3a1574ebfe818bcaa3b9479bdd05130a49444e90d4d5760</code> |
| `aarch64-apple-darwin` | <code class="sl-digest">3b0591184b1ac0c8ed85073e13815f55a41d1b28477cd4f45e0a7260facfbca4</code> |
| `x86_64-pc-windows-msvc` | <code class="sl-digest">5e678e6c337f7eb0c0128c1f0ecb855f23dbab05b3b88646de6680f4f5b9f27a</code> |

Digests are platform-specific and change on every re-release, because the archives are rebuilt. Read yours from the release page rather than from a copy of this page. Omitting `checksum` still verifies the archive — against the `.sha256` file owned by the release itself. Supplying `checksum` pins it further.

Supported runners: `x86_64`/`aarch64` Linux, `x86_64`/`aarch64` macOS, and `x86_64` Windows. The Action runs on the `node24` runtime.

## Inputs

| Input | Default | Description |
|---|---|---|
| `version` | `latest` | `soroban-lint-core` release tag to install, e.g. `v0.1.2`. |
| `repository` | `Stellar-Soroban-Lint/soroban-lint-core` | Repository that publishes the binaries. |
| `path` | `.` | File or directory to lint. |
| `fail-on` | `error` | Minimum severity that fails the step: `error`, `warning`, `info`, `never`. |
| `experimental` | `false` | Enable the experimental rules. |
| `config` | – | Path to a `soroban-lint.toml`. Auto-discovered when omitted. |
| `args` | – | Extra whitespace-separated arguments appended to `soroban-lint check`. |
| `annotations` | `true` | Emit one workflow annotation per finding. |
| `max-annotations` | `50` | Cap on emitted annotations. |
| `comment` | `true` | Post a summary comment on a PR and update it in place. |
| `checksum` | – | Expected SHA-256 of the archive, to pin beyond the published digest. |
| `sarif-file` | – | When set, also write a SARIF 2.1.0 report to this path. |
| `token` | – | Token used only to raise API rate limits; not required for public releases. |

Note the Action's `fail-on` default is `error`, while the CLI's own default is `warning`. In CI that is the safer default: a new warning does not break a build.

## Enabling the experimental rules

```yaml
      - uses: Stellar-Soroban-Lint/soroban-lint-action@v0
        with:
          path: contracts
          experimental: "true"
          fail-on: warning
```

Since `fail-on` and `annotations` default conservatively, enabling the experimental rules without changing `fail-on` lets you see what they report before letting them fail anything.

## Outputs

| Output | Description |
|---|---|
| `version` | The installed release tag. |
| `binary` | Absolute path to the installed executable (also added to `PATH`). |
| `findings` | Total number of findings. |
| `errors` / `warnings` / `notices` | Findings per severity. |
| `exit-code` | Exit code returned by `soroban-lint check`. |
| `sarif-file` | Path written when `sarif-file` is set. |

## SARIF

```yaml
jobs:
  soroban-lint:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write
    steps:
      - uses: actions/checkout@v4
      - uses: Stellar-Soroban-Lint/soroban-lint-action@v0
        with:
          path: contracts
          sarif-file: soroban-lint.sarif
          fail-on: never
      - uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: soroban-lint.sarif
```

`fail-on: never` here so that the upload runs even when findings are present; the SARIF upload is what records them. Each result carries a `sorobanLint/v1` partial fingerprint, so GitHub can track an alert across runs instead of filing a new one each time.

For a fork pull request the SARIF upload is skipped along with the comment, since the token cannot write.

## Working within GitHub's limits

The Action is built around GitHub's caps rather than discovering them in production:

| Limit | Value | How it is handled |
|---|---|---|
| Workflow-command annotations | 10 per step, 50 per job | `max-annotations` (default `50`) caps emission; the dropped count is logged. |
| Checks API annotations | 50 per request | Not called directly; SARIF upload is the bulk channel. |
| Pull request comment body | 65,536 characters | The comment lists at most 20 findings, then points at the annotations and SARIF. |
| Default `GITHUB_TOKEN` | Read-only on fork PRs | The comment is skipped rather than failing the run. |

When there are more findings than the cap, nothing is silently dropped. The remainder is counted in the log, listed in the job summary, and included in the SARIF file if `sarif-file` is set.

## See it run

Two open pull requests on the portal repository demonstrate both outcomes against real code:

- **A failing run** with `fail-on: error` — [soroban-lint-portal#1](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/pull/1), branch `demo/soroban-lint-action`.
- **A passing run** with `fail-on: never` and a low `max-annotations` — [soroban-lint-portal#2](https://github.com/Stellar-Soroban-Lint/soroban-lint-portal/pull/2), branch `demo/soroban-lint-action-pass`.

Open either one and read the annotations on the changed lines rather than trusting the summary count.

## What it does not do

soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.

A green Action run means no pattern the eight rules detect was present. It is not an audit, and it is not a security guarantee. Both claims are in the Action's own marketplace description for the same reason.

## Next

- [CLI reference](/cli) — the flags this Action passes through, and the exit codes behind them.
- [Rules](/rules/) — what each rule detects and what it misses.
- [Benchmarks](/benchmarks/) — what these rules found in a real repository.
