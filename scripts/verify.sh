#!/usr/bin/env bash
# Reproduce the project's own claims with one command.
#
# Three things are asserted here, and each one has been a claim in prose before:
#
#   1. the workspace formats, lints clean, and passes its tests;
#   2. the pinned benchmark corpus produces the per-rule counts in
#      `docs/BENCHMARKS.md`, with no findings in `#[cfg(test)]` module files;
#   3. the total in `docs/BENCHMARKS.md` is not stale — the script fails if the
#      document and the binary disagree.
#
# The corpus belongs to the portal repository and is not vendored twice. Point
# `SOROBAN_LINT_CORPUS` at a checkout of `stellar/soroban-examples` at commit
# 03d42aa6b973dcf3a453a99d0c6a6e8d25a196e2 to run the check without the sibling
# repository; without either, checks 2 and 3 are skipped and said to be skipped.
#
# Usage:
#   scripts/verify.sh            # formatting, lint, tests, corpus report
#   scripts/verify.sh --no-corpus
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_CORPUS="$ROOT/../soroban-lint-portal/tests/corpus/soroban-examples"
CORPUS="${SOROBAN_LINT_CORPUS:-$DEFAULT_CORPUS}"
RUN_CORPUS=1

for arg in "$@"; do
  case "$arg" in
    --no-corpus) RUN_CORPUS="" ;;
    -h|--help)
      sed -n '2,24p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

step() { printf '\n== %s\n' "$1"; }

step "cargo fmt --all --check"
( cd "$ROOT" && cargo fmt --all --check )

step "cargo clippy --workspace --all-targets --all-features"
( cd "$ROOT" && cargo clippy --workspace --all-targets --all-features -- -D warnings )

step "cargo test --workspace"
( cd "$ROOT" && cargo test --workspace )

if [[ -z "$RUN_CORPUS" ]]; then
  echo
  echo "corpus report: skipped (--no-corpus)"
  exit 0
fi

if [[ ! -d "$CORPUS" ]]; then
  echo
  echo "corpus report: skipped (no corpus at $CORPUS)"
  echo "  set SOROBAN_LINT_CORPUS to a stellar/soroban-examples checkout at 03d42aa6"
  exit 0
fi

step "cargo build -p soroban-lint-cli"
( cd "$ROOT" && cargo build -p soroban-lint-cli )

BIN="$ROOT/target/debug/soroban-lint"
OUTPUT="$(mktemp)"
NOTICE="$(mktemp)"
trap 'rm -f "$OUTPUT" "$NOTICE"' EXIT

step "corpus report ($CORPUS)"
"$BIN" check "$CORPUS" --experimental --format json --fail-on never \
  >"$OUTPUT" 2>"$NOTICE"
sed -n '1p' "$NOTICE"

REPORT="$(node "$ROOT/scripts/corpus-report.mjs" "$OUTPUT")"

printf '%s\n' "$REPORT" | sed '/^TOTAL=/d'
TOTAL="$(printf '%s\n' "$REPORT" | sed -n 's/^TOTAL=//p')"

# The heading the report has to match, so the document cannot drift from the binary.
EXPECTED="**${TOTAL} findings across the corpus.**"
if ! grep -Fq "$EXPECTED" "$ROOT/docs/BENCHMARKS.md"; then
  echo
  echo "FAIL: docs/BENCHMARKS.md does not contain: $EXPECTED" >&2
  echo "Re-run the corpus command and update the document with the measured counts." >&2
  exit 1
fi

echo
echo "docs/BENCHMARKS.md matches the measured total ($TOTAL)."
