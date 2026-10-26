#!/usr/bin/env node
/**
 * Print the per-rule corpus table from a `soroban-lint check --format json` report.
 *
 * `scripts/verify.sh` runs the CLI and pipes the report through this, so the numbers in
 * `docs/BENCHMARKS.md` can be compared with a measurement instead of trusted. The last
 * line is `TOTAL=<n>` so the caller can read the count without re-parsing JSON.
 *
 * Exits 1 when a finding lands in a `test.rs`/`tests.rs` file: the walker is supposed to
 * skip those, so a finding there means the skip regressed.
 *
 * Usage: node scripts/corpus-report.mjs <report.json>
 */

import { readFileSync } from "node:fs";

const reportPath = process.argv[2];
if (!reportPath) {
  console.error("usage: corpus-report.mjs <report.json>");
  process.exit(2);
}

const { diagnostics } = JSON.parse(readFileSync(reportPath, "utf8"));
const byRule = new Map();
for (const diagnostic of diagnostics) {
  byRule.set(diagnostic.rule_id, (byRule.get(diagnostic.rule_id) ?? 0) + 1);
}

const fileCount = new Set(diagnostics.map((diagnostic) => diagnostic.file)).size;
const inTestModules = diagnostics.filter((diagnostic) =>
  /(^|\/)tests?\.rs$/.test(diagnostic.file),
);

for (const id of [...byRule.keys()].sort()) {
  console.log(`  ${id}  ${String(byRule.get(id)).padStart(3)}`);
}
console.log(`  ---  ${String(diagnostics.length).padStart(3)} across ${fileCount} files`);
console.log(`  test-module findings: ${inTestModules.length}`);
console.log(`TOTAL=${diagnostics.length}`);

if (inTestModules.length > 0) {
  console.error("FAIL: findings were reported in test modules");
  process.exit(1);
}
