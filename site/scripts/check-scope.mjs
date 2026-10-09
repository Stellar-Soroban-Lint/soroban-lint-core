#!/usr/bin/env node
/**
 * The scope statement must appear verbatim on every site page.
 *
 * `soroban-lint-core/tests/docs_scope.rs` already enforces the same string on
 * the seven repository documents it knows about. It cannot enforce it on the
 * site, because a page may be generated, and the Rust test has no notion of
 * which files the build produces. This is that missing half.
 *
 * A page that describes what the linter finds without saying what it cannot see
 * is the exact failure this project committed to avoiding. The statement is a
 * word-for-word literal, not a summary, so a reworded or truncated copy fails
 * instead of passing as good enough.
 *
 * Two exemptions, both deliberate and both narrow:
 *
 *   - `rules/index.md` carries it at the foot; a rule *index* is a table, and
 *     repeating it above every rule would bury the table.
 *   - `404.md` does not carry it. A missing page has nothing to caveat.
 *
 * Usage: node scripts/check-scope.mjs
 */

import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SITE = resolve(dirname(fileURLToPath(import.meta.url)), "..");

/** Verbatim from `SPEC.md` §1. Do not reformat: the Rust test reads the same line. */
const STATEMENT =
  "soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.";

// A mangled expectation is reported as a mangled expectation, not as every page
// mysteriously missing it.
if (!STATEMENT.startsWith("soroban-lint performs syntactic, per-file analysis")) {
  throw new Error("the expected statement is malformed: it must open with the analysis clause");
}
if (!STATEMENT.endsWith("this tool is not a substitute for an audit.")) {
  throw new Error("the expected statement is malformed: it must end with the audit clause");
}
if (STATEMENT.includes("\n") || STATEMENT.includes("  ")) {
  throw new Error("the expected statement must be a single line without double spaces");
}

/** The statement itself, as it appears in `SPEC.md`. Cross-checked below. */
function statementFromSpec() {
  const spec = readFileSync(resolve(SITE, "..", "SPEC.md"), "utf8");
  const line = spec
    .split("\n")
    .find((l) => l.includes("soroban-lint performs syntactic,"));
  if (!line) throw new Error("SPEC.md does not contain the scope statement");
  return line.replace(/^>\s*/, "").trim();
}

const fromSpec = statementFromSpec();
if (fromSpec !== STATEMENT) {
  throw new Error(
    [
      "the statement in this script differs from SPEC.md §1.",
      "",
      "The Rust test reads the statement out of SPEC.md and the site must carry the",
      "same words, so the two are one value with two homes. Copy the line from",
      "SPEC.md rather than editing this string.",
    ].join("\n"),
  );
}

/** Pages exempt from the statement, with the reason. */
const EXEMPT = new Map([
  ["rules/index.md", "the statement is carried at the foot, below the catalog table"],
  ["404.md", "a missing page has nothing to caveat"],
]);

function markdownFiles(dir) {
  const found = [];
  for (const entry of readdirSync(dir)) {
    if (entry === "node_modules" || entry === ".vitepress" || entry === ".cache") continue;
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) found.push(...markdownFiles(path));
    else if (entry.endsWith(".md")) found.push(path);
  }
  return found;
}

const pages = markdownFiles(SITE).filter((p) => !p.includes(`${join("rules-prose", "")}`));
const missing = [];

for (const page of pages) {
  const rel = relative(SITE, page);
  if (EXEMPT.has(rel)) continue;
  if (!readFileSync(page, "utf8").includes(STATEMENT)) missing.push(rel);
}

if (missing.length > 0) {
  throw new Error(
    [
      "the scope statement must appear verbatim on every page. Missing from:",
      ...missing.map((p) => `  site/${p}`),
      "",
      "Paste it from SPEC.md §1 rather than rewording it. If a page genuinely does",
      "not need it, add it to EXEMPT with a reason — do not silently drop the page",
      "from the check.",
    ].join("\n"),
  );
}

const exemptNote = [...EXEMPT.entries()]
  .filter(([rel]) => existsSync(join(SITE, rel)))
  .map(([rel]) => `${rel} (${EXEMPT.get(rel)})`);

console.log(
  `scope statement: verbatim on ${pages.length - exemptNote.length} of ${pages.length} pages` +
    (exemptNote.length ? `; exempt: ${exemptNote.join(", ")}` : ""),
);