#!/usr/bin/env node
/**
 * Generate the rule pages and the rules index from the pinned release.
 *
 * The rule pages cannot be hand-written. `soroban-lint rules --format json` is
 * the released binary's own description of itself; if the prose were typed by
 * hand it would drift from the binary the moment a severity, a confidence, or a
 * limitation changed, and the site would begin describing something that does
 * not ship. So every factual field comes from that JSON, and the build fails if
 * a field is missing, malformed, or duplicated.
 *
 * Hand-written prose is not thrown away. It lives in `rules-prose/<ID>.md` and
 * is merged in as a verbatim block. That split is the point: the binary owns the
 * facts, a human owns the examples, and neither can silently overwrite the other.
 *
 * The binary is downloaded from the release rather than built. A site built
 * during a release PR describes what the maintainer has locally, not what users
 * can install — and a new rule would appear documented before its tag existed.
 * The archive is checked against the `.sha256` published beside it before it is
 * unpacked.
 *
 * Usage:
 *   node scripts/gen-rules.mjs
 *   node scripts/gen-rules.mjs --check   # fail if the pages are stale
 */

import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SITE = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const OUT_DIR = join(SITE, "rules");
const PROSE_DIR = join(SITE, "rules-prose");
const VERSION_FILE = join(SITE, "SOROBAN_LINT_VERSION");
const CACHE_DIR = join(SITE, ".cache");
const CHECK_ONLY = process.argv.includes("--check");

const REPO_SLUG = "Stellar-Soroban-Lint/soroban-lint-core";
const PLAYGROUND = "https://stellar-soroban-lint.github.io/soroban-lint-portal/";
const TARGET = "x86_64-unknown-linux-gnu";

const SEVERITIES = ["error", "warning", "info"];
const CONFIDENCES = ["high", "medium", "low"];
const STABILITIES = ["stable", "experimental"];

/**
 * Fields the binary must supply, each with a predicate rather than a bare
 * presence check: `description: "   "` is present and useless, and a page built
 * from it would be blank exactly where the claim belongs.
 */
const REQUIRED = {
  id: (v) => typeof v === "string" && /^SL\d{3}$/.test(v),
  name: (v) => typeof v === "string" && v.length > 0,
  description: (v) => typeof v === "string" && v.trim().length > 0,
  rationale: (v) => typeof v === "string" && v.trim().length > 0,
  limitations: (v) => typeof v === "string" && v.trim().length > 0,
  default_severity: (v) => SEVERITIES.includes(v),
  default_confidence: (v) => CONFIDENCES.includes(v),
  stability: (v) => STABILITIES.includes(v),
};

/** The pinned release, read from the file a release PR bumps. */
const releaseTag = readFileSync(VERSION_FILE, "utf8").trim();
if (!/^v\d+\.\d+\.\d+$/.test(releaseTag)) {
  throw new Error(
    `SOROBAN_LINT_VERSION contains "${releaseTag}", which is not a release tag like v0.1.1.`,
  );
}
const expectedVersion = releaseTag.replace(/^v/, "");

/** Download the release archive, or use the cached one. Verifies its digest. */
async function resolveBinary() {
  const archive = `soroban-lint-${releaseTag}-${TARGET}.tar.gz`;
  const base = `https://github.com/${REPO_SLUG}/releases/download/${releaseTag}`;
  const dir = join(CACHE_DIR, releaseTag);
  const archivePath = join(dir, archive);

  if (process.platform !== "linux" || process.arch !== "x64") {
    // Rather than pretend, say which platform failed and how to fix it.
    const local = process.env.SOROBAN_LINT_BIN;
    if (local && existsSync(local)) {
      return { path: local, source: `SOROBAN_LINT_BIN (${local})`, verified: false };
    }
    throw new Error(
      [
        `no pinned release archive for ${process.platform}/${process.arch}.`,
        "",
        "The rule pages are generated from the released linux binary. On another",
        "platform, set SOROBAN_LINT_BIN to a soroban-lint executable of version",
        `${expectedVersion} and re-run.`,
      ].join("\n"),
    );
  }

  mkdirSync(dir, { recursive: true });

  if (!existsSync(archivePath)) {
    process.stderr.write(`downloading ${archive}\n`);
    const response = await fetch(`${base}/${archive}`);
    if (!response.ok) {
      throw new Error(
        `cannot download ${archive} from ${base}: HTTP ${response.status}.\n` +
          `Is ${releaseTag} published? If the site is ahead of a release, bump SOROBAN_LINT_VERSION.`,
      );
    }
    writeFileSync(archivePath, Buffer.from(await response.arrayBuffer()));
  }

  const bytes = readFileSync(archivePath);
  const actual = createHash("sha256").update(bytes).digest("hex");

  // Cross-check against the digest the release publishes, not a digest pasted
  // into this file. A pasted digest goes stale the moment the release is
  // rebuilt, and then it is a value nobody re-verified.
  const published = await fetch(`${base}/${archive}.sha256`);
  if (!published.ok) {
    throw new Error(
      `${archive}.sha256 was not published alongside ${archive} (HTTP ${published.status}). ` +
        "Every release asset ships with a sibling checksum; this one does not.",
    );
  }
  const stated = (await published.text()).trim().split(/\s+/)[0];
  if (stated !== actual) {
    throw new Error(
      [
        `${archive} does not match the checksum published with the release.`,
        "",
        `  published  ${stated}`,
        `  actual     ${actual}`,
        "",
        "Refusing to generate rule pages from an unverified binary.",
      ].join("\n"),
    );
  }

  execFileSync("tar", ["-xzf", archivePath, "-C", dir], { stdio: "pipe" });
  const binary = join(dir, `soroban-lint-${releaseTag}-${TARGET}`, "soroban-lint");
  if (!existsSync(binary)) {
    throw new Error(`${archive} unpacked but no executable at ${binary}`);
  }
  return { path: binary, source: `release ${releaseTag} (checksum verified)`, verified: true };
}

const { path: binary, source } = await resolveBinary();

const probe = spawnSync(binary, ["--version"], { encoding: "utf8" });
if (probe.status !== 0) {
  throw new Error(`\`${binary} --version\` failed: ${probe.stderr || probe.error}`);
}
const binaryVersion = (probe.stdout.match(/(\d+\.\d+\.\d+)/) ?? [])[1];
if (binaryVersion !== expectedVersion) {
  throw new Error(
    [
      `SOROBAN_LINT_VERSION pins ${releaseTag}, but the binary at ${source} reports ${binaryVersion}.`,
      "",
      "Either the pin or the release is wrong. Refusing to document a binary the",
      "site is not pinned to.",
    ].join("\n"),
  );
}

const emitted = spawnSync(binary, ["rules", "--format", "json"], { encoding: "utf8" });
if (emitted.status !== 0) {
  throw new Error(`\`soroban-lint rules --format json\` failed: ${emitted.stderr || emitted.error}`);
}
const catalog = JSON.parse(emitted.stdout);
if (!Array.isArray(catalog.rules) || catalog.rules.length === 0) {
  throw new Error("`soroban-lint rules --format json` returned no rules");
}

/** Collect every problem before throwing, so one run reports them all. */
function validate(rules) {
  const problems = [];
  for (const rule of rules) {
    const where = typeof rule?.id === "string" ? rule.id : JSON.stringify(rule);
    for (const [field, check] of Object.entries(REQUIRED)) {
      if (!(field in rule)) {
        problems.push(`${where}: missing required field \`${field}\``);
      } else if (!check(rule[field])) {
        problems.push(`${where}: field \`${field}\` has an unusable value ${JSON.stringify(rule[field])}`);
      }
    }
  }
  const ids = rules.map((r) => r.id);
  for (const id of new Set(ids.filter((id, i) => ids.indexOf(id) !== i))) {
    problems.push(`duplicate rule id \`${id}\`: ids are permanent and must be unique`);
  }
  if (problems.length > 0) {
    throw new Error(`\`soroban-lint rules --format json\` is unusable:\n  ${problems.join("\n  ")}`);
  }
}

const rules = [...catalog.rules].sort((a, b) => a.id.localeCompare(b.id));
validate(rules);

/** Prose files, keyed by rule id. */
const prose = new Map();
if (existsSync(PROSE_DIR)) {
  for (const entry of readdirSync(PROSE_DIR)) {
    if (!entry.endsWith(".md")) continue;
    // `README.md` explains the directory; it is not prose about a rule.
    if (entry.toLowerCase() === "readme.md") continue;
    prose.set(entry.replace(/\.md$/, ""), readFileSync(join(PROSE_DIR, entry), "utf8").trim());
  }
}

// Prose for a rule the binary does not have is a documentation bug in the
// other direction: the page would vanish while its guidance stayed in the
// repository looking current.
for (const id of prose.keys()) {
  if (!rules.some((r) => r.id === id)) {
    throw new Error(
      `rules-prose/${id}.md documents a rule the ${releaseTag} binary does not have. ` +
        "Remove the file, or check whether SOROBAN_LINT_VERSION is behind.",
    );
  }
}

for (const rule of rules) {
  const notes = prose.get(rule.id) ?? "";
  for (const [field, pattern] of [
    ["examples", /^## Examples\s*$/m],
    ["false-positive mode", /\*\*False positives?\.\*\*/i],
    ["false-negative mode", /\*\*False negatives?\.\*\*/i],
  ]) {
    if (!pattern.test(notes)) {
      throw new Error(
        `rules-prose/${rule.id}.md is missing a required ${field} section. ` +
          "Each rule needs examples and explicit false-positive and false-negative modes.",
      );
    }
  }
  if (!/^## Examples\s*$[\s\S]*?```/m.test(notes)) {
    throw new Error(`rules-prose/${rule.id}.md has an Examples heading but no code example.`);
  }
}

const SCOPE_STATEMENT =
  "soroban-lint performs syntactic, per-file analysis of Soroban contract source using the Rust AST. It flags patterns associated with missing authorization checks, panic paths, unchecked arithmetic, and storage hazards in `#[contractimpl]` functions. It does not expand macros, resolve types, or follow calls across files, so it can miss real issues (false negatives) and flag safe code (false positives). A clean report is not evidence a contract is secure, and this tool is not a substitute for an audit.";

const BADGE_TYPE = { error: "danger", warning: "warning", info: "info" };

function renderRulePage(rule) {
  const notes = prose.get(rule.id);
  const severity = `<Badge type="${BADGE_TYPE[rule.default_severity]}" text="${rule.default_severity}" />`;
  const confidence = `<Badge type="info" text="${rule.default_confidence} confidence" />`;
  const stability =
    rule.stability === "stable"
      ? '<Badge type="tip" text="stable" />'
      : '<Badge text="experimental" />';

  const badges = [
    severity,
    confidence,
    stability,
    rule.stability === "stable"
      ? '<Badge type="tip" text="on by default" />'
      : '<Badge text="off by default" />',
  ].join(" ");

  const sections = [];

  sections.push(`# ${rule.id} — ${rule.name}`);
  sections.push(
    rule.stability === "experimental"
      ? `${badges}\n\nSL003–SL007 are **off by default**. Enable them with \`--experimental\` or \`experimental = true\` in \`soroban-lint.toml\`.`
      : badges,
  );

  sections.push(
    [
      "| | |",
      "|---|---|",
      `| Rule id | \`${rule.id}\` |`,
      `| Name | \`${rule.name}\` |`,
      `| Default severity | ${severity} |`,
      `| Default confidence | ${confidence} |`,
      `| Stability | ${stability} |`,
      `| Documented release | \`${releaseTag}\` |`,
    ].join("\n"),
  );

  sections.push(["## What it flags", rule.description].join("\n\n"));
  sections.push(["## Why it matters", rule.rationale].join("\n\n"));

  // The limitation sits directly under the capability, never as a footnote. A
  // reader who stops after "what it flags" must still see what it cannot see.
  sections.push(["## Limitations", rule.limitations, "", `> ${SCOPE_STATEMENT}`].join("\n\n"));

  if (notes) {
    sections.push(notes);
  }

  sections.push(
    `[Try ${rule.id} in the live playground](${PLAYGROUND}) · [View its portal rule page](${PLAYGROUND}rules/${rule.id})`,
  );

  sections.push(
    [
      "---",
      "",
      `*Generated from \`soroban-lint rules --format json\` of ${releaseTag}, merged with`,
      `hand-written prose from \`rules-prose/${rule.id}.md\`. A rule that does not ship`,
      "cannot have a page here. See [Rules](/rules/) for the full catalog.",
    ].join("\n"),
  );

  const frontmatter = [
    "---",
    `title: "${rule.id} ${rule.name}"`,
    `description: "${rule.description.replace(/"/g, "'")}"`,
    `editPath: "site/rules-prose/${rule.id}.md"`,
    "---",
    "",
  ].join("\n");

  return `${frontmatter}${sections.join("\n\n").replace(/\n{3,}/g, "\n\n")}\n`;
}

function renderIndex() {
  const rows = [
    "| ID | Rule | Severity | Confidence | Stability | Description |",
    "|---|---|---|---|---|---|",
  ];
  for (const rule of rules) {
    const stab = rule.stability === "stable" ? "**stable**" : "experimental";
    rows.push(
      `| [\`${rule.id}\`](/rules/${rule.id}) | \`${rule.name}\` | ${rule.default_severity} | ${rule.default_confidence} | ${stab} | ${rule.description} |`,
    );
  }

  return `---
title: "Rules"
description: "The rule catalog, generated from the released binary's own metadata."
editPath: "site/rules-prose/README.md"
---

<!-- Generated by scripts/gen-rules.mjs. The table below comes from \`soroban-lint rules --format json\` of ${releaseTag}. -->

# Rules

soroban-lint ships ${rules.length} rules. The table is generated from \`soroban-lint rules --format json\` of the pinned release, so it cannot describe a rule the binary does not have, and cannot fall behind a severity change.

${rows.join("\n")}

**Severity** is how loudly a finding is reported. **Confidence** is how much syntactic evidence backs it, and it is independent of severity: [SL008](/rules/SL008) is \`high\` confidence for the \`unsafe\` half of its check and \`low\` for the \`no_std\` half.

## Stable and experimental

${rules.filter((r) => r.stability === "stable").map((r) => `[${r.id}](/rules/${r.id})`).join(", ")} are **stable** and run by default. ${rules.filter((r) => r.stability !== "stable").map((r) => `[${r.id}](/rules/${r.id})`).join(", ")} are **experimental** and are off unless you pass \`--experimental\` or set \`experimental = true\` in [\`soroban-lint.toml\`](/cli#configuration).

This is not a maturity decoration. A rule is promoted to stable only when a real-code corpus shows no unexplained false positives, and the current corpus is small and toy-heavy. No rule is promoted on the strength of passing its own fixtures. See [the promotion decision](/benchmarks/#promotion-decision) for the specific reasons the experimental rules are still waiting.

Turning experimental rules on is worth doing once, on a real repository, before you depend on them in a blocking CI check.

## SL000 is not a security rule

\`SL000\` is reserved for meta diagnostics — parse failures, invalid configuration, unknown rule ids in the config, and unknown ids in a suppression. It is deliberately not documented as a security rule and has no page here, because it reports a problem with the run rather than a pattern in the contract.

## Rule ids are permanent

An id is never reused and never renumbered. A retired id stays reserved. If you suppress \`SL006\` in a comment or a config file and the rule is later retired under a different id, that reference still means what it meant.

Adding a rule means taking the next free number, and it is a deliberate act: a true-positive fixture, a safe negative, at least one tricky negative, and a documented false-negative fixture proving the stated limitation is real. See [Writing a rule](/writing-rules/).

## How to read a rule page

Every rule page is built from two sources that are kept apart on purpose:

- **The binary's own metadata** — description, rationale, limitations, default severity, default confidence, stability — read from \`soroban-lint rules --format json\`. The release describes itself.
- **Hand-written prose** — examples and guidance, in \`site/rules-prose/<ID>.md\`, merged in verbatim at build time.

Neither source can overwrite the other, and the generator fails if prose documents a rule the binary does not have, or if a required field is missing or malformed.

The **Limitations** section is placed immediately after the capability claim on every page rather than at the foot. If you read only the first half of a rule page, you still see what the rule cannot see.

---

${SCOPE_STATEMENT}
`;
}

mkdirSync(OUT_DIR, { recursive: true });

const outputs = new Map([
  ["index.md", renderIndex()],
  ...rules.map((rule) => [`${rule.id}.md`, renderRulePage(rule)]),
]);

const stale = [];
for (const [name, content] of outputs) {
  const file = join(OUT_DIR, name);
  const previous = existsSync(file) ? readFileSync(file, "utf8") : null;
  if (previous === content) continue;
  if (CHECK_ONLY) stale.push(name);
  else writeFileSync(file, content);
}

// A page left behind by a retired rule would still be published.
const expected = new Set(outputs.keys());
for (const entry of readdirSync(OUT_DIR)) {
  if (!entry.endsWith(".md")) continue;
  if (!expected.has(entry)) {
    throw new Error(
      `rules/${entry} has no matching rule in ${releaseTag}. Rule pages are generated; delete the stale page.`,
    );
  }
}

if (CHECK_ONLY) {
  if (stale.length > 0) {
    throw new Error(
      [
        `generated rule pages are out of date: ${stale.join(", ")}`,
        "",
        "Run `npm run rules` and commit the result.",
      ].join("\n"),
    );
  }
  console.log(`gen-rules: ${rules.length} rule pages checked, all current`);
} else {
  const stable = rules.filter((r) => r.stability === "stable").length;
  console.log(
    [
      `gen-rules: ${rules.length} rule pages from ${source}`,
      `  ${stable} stable, ${rules.length - stable} experimental`,
      `  ${prose.size} with hand-written prose merged`,
      `  catalog sha256 ${createHash("sha256").update(emitted.stdout).digest("hex").slice(0, 12)}`,
    ].join("\n"),
  );
}
