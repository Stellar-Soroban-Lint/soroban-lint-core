# Rule prose

One file per rule, `<ID>.md`, holding what a human wrote and the binary cannot know: worked examples, suppression guidance, and the corpus triage.

This directory is the human half of the rule pages. The other half comes from `soroban-lint rules --format json` of the release pinned in `site/SOROBAN_LINT_VERSION` — description, rationale, limitations, severity, confidence, stability. The two are merged at build time by `scripts/gen-rules.mjs` and neither can overwrite the other.

**Do not duplicate a fact the binary already supplies here.** A severity, a confidence, or a limitation written in this file will not be compared against the binary; it will simply disagree with it the first time the rule changes. That is the whole failure mode this split exists to prevent.

What belongs here:

- `## When it fires` — the precise condition, including what is deliberately *not* flagged.
- `## Examples` — code, with the flagged and not-flagged forms side by side.
- `## Suppressing this rule` — when a suppression is the right answer, and what to write in the comment.
- `## On the benchmark corpus` — this rule's slice of the [corpus run](/benchmarks/), and the findings that were fixed in code.
- `## See also` — links.

Every file is optional except the existence of the directory. The generator fails if a file here names a rule the pinned binary does not have, because that means either the file is stale or `SOROBAN_LINT_VERSION` is behind — and silently generating nothing would hide both.