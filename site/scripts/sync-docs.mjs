#!/usr/bin/env node
/**
 * Copy the documents that already live in this repository into the site.
 *
 * The repository's markdown is the source of truth. `SPEC.md`,
 * `docs/ARCHITECTURE.md`, `docs/BENCHMARKS.md`, `docs/WRITING_RULES.md`,
 * `docs/ROADMAP.md`, and `CHANGELOG.md` are read by contributors in a text
 * editor and by `tests/docs_scope.rs` as files at known paths. Publishing them
 * as site pages therefore cannot mean keeping a second copy: the two would
 * drift, and the copy is the one that would rot.
 *
 * So this script copies, adds frontmatter, and rewrites the links that would
 * break — nothing else. Every word of prose is preserved, because the prose is
 * reviewed in the repository, not on the website.
 *
 * No symlinks. They resolve on Linux and break on Windows checkouts and in some
 * CI containers, and a documentation site that only builds on the maintainer's
 * machine is not a documentation site.
 *
 * The destinations are gitignored (see `.gitignore`). If they were committed,
 * a reviewer could approve a change to a generated copy and wonder why the
 * build overwrote it.
 *
 * Usage:
 *   node scripts/sync-docs.mjs
 *   node scripts/sync-docs.mjs --check   # fail if the copies are stale
 */

import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SITE = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const REPO = resolve(SITE, "..");
const CHECK_ONLY = process.argv.includes("--check");

/**
 * Each entry is a source document, where its pages land, and the frontmatter to
 * put on top. `editPath` is what the page's "Edit this page" link should point
 * at — the file in the repository, not the generated copy, because that is the
 * file a reader should change.
 */
const DOCUMENTS = [
  {
    source: "SPEC.md",
    out: "spec/index.md",
    title: "Specification",
    description:
      "The normative specification: analysis mechanism, core interfaces, CLI contract, and every rule's algorithm.",
    editPath: "SPEC.md",
  },
  {
    source: "docs/ARCHITECTURE.md",
    out: "architecture/index.md",
    title: "Architecture",
    description:
      "Data flow, why the analysis is syntactic, and the three limits that follow from that.",
    editPath: "docs/ARCHITECTURE.md",
  },
  {
    source: "docs/BENCHMARKS.md",
    out: "benchmarks/index.md",
    title: "Benchmarks",
    description:
      "The 103-finding corpus run against stellar/soroban-examples, with per-rule triage.",
    editPath: "docs/BENCHMARKS.md",
  },
  {
    source: "docs/WRITING_RULES.md",
    out: "writing-rules/index.md",
    title: "Writing a rule",
    description:
      "How to add a rule without breaking the guarantees the existing rules make.",
    editPath: "docs/WRITING_RULES.md",
  },
  {
    source: "docs/ROADMAP.md",
    out: "roadmap/index.md",
    title: "Roadmap",
    description:
      "Work that is deliberately unfiled and not scheduled. Preconditions, not dates.",
    editPath: "docs/ROADMAP.md",
  },
  {
    source: "CHANGELOG.md",
    out: "changelog/index.md",
    title: "Changelog",
    description: "Every released change to soroban-lint.",
    editPath: "CHANGELOG.md",
  },
];

/**
 * Cross-document links, rewritten after the files move.
 *
 * These are all *backtick* references rather than markdown links — the source
 * documents name their siblings as code spans, e.g. `` `docs/BENCHMARKS.md` ``.
 * Those do not break when rendered as markdown, but they read as paths and go
 * stale the moment the site republishes the document somewhere else, so a
 * reader following `docs/BENCHMARKS.md` from the published Architecture page has
 * no way to get there.
 *
 * Rewritten to a real markdown link against the published location. Repository
 * paths that are *not* published as pages — `tests/fixtures/`, `src/rules/*.rs`
 * — are left exactly as they are: pointing at a source file in a repository is
 * the correct behaviour, and inventing a site URL for a fixture that is not
 * published would be a dead link.
 */
const LINK_REWRITES = new Map([
  ["SPEC.md", "/spec/"],
  ["docs/ARCHITECTURE.md", "/architecture/"],
  ["docs/BENCHMARKS.md", "/benchmarks/"],
  ["docs/WRITING_RULES.md", "/writing-rules/"],
  ["docs/ROADMAP.md", "/roadmap/"],
  ["CHANGELOG.md", "/changelog/"],
]);

/**
 * Links that point into the repository on GitHub rather than to a site page.
 * `src/...` and `tests/...` stay as text; a published fixture directory would
 * be a lie, since the site does not carry one.
 */
function rewriteLinks(markdown) {
  let out = markdown;

  // Backtick path -> site link, only for documents that are published.
  for (const [path, sitePath] of LINK_REWRITES) {
    const escaped = path.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    // `` `docs/BENCHMARKS.md` `` and `` `docs/BENCHMARKS.md`: `` alike.
    out = out.replace(
      new RegExp("`" + escaped + "`", "g"),
      `[${path}](${sitePath})`,
    );
  }

  // Bare backticked paths that are not published stay as they are, but the
  // markdown-it renderer's own link checker must not see them as links. They
  // are code spans, so they are not links; nothing to do.

  return out;
}

/** Frontmatter for a synced page. */
function frontmatter({ title, description, editPath }) {
  const lines = [
    "---",
    `title: ${title}`,
  ];
  if (description) {
    lines.push(`description: ${description}`);
  }
  // `lastUpdated` reads the commit timestamp of the *source* file, so a page
  // rebuilt without its document changing reports the copy's date instead.
  lines.push(`editPath: ${editPath}`);
  lines.push("---", "");
  return lines.join("\n");
}

/** Escape a string for use in a YAML double-quoted scalar. */
function yamlString(value) {
  return `"${String(value).replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function render(doc) {
  const source = join(REPO, doc.source);
  if (!existsSync(source)) {
    throw new Error(
      `sync-docs: ${doc.source} is missing from the repository.\n` +
        "The site publishes it; if it was renamed or removed, update DOCUMENTS in this script.",
    );
  }
  const body = readFileSync(source, "utf8");

  const header = [
    "---",
    `title: ${yamlString(doc.title)}`,
    doc.description ? `description: ${yamlString(doc.description)}` : null,
    `editPath: ${yamlString(doc.editPath)}`,
    "---",
    "",
    `<!-- Generated by scripts/sync-docs.mjs from ${doc.source}. Edit that file, not this one. -->`,
    "",
  ]
    .filter((line) => line !== null)
    .join("\n");

  return `${header}\n${rewriteLinks(body).trimEnd()}\n`;
}

mkdirSync(join(SITE, "rules-prose"), { recursive: true });

const stale = [];
const written = [];

for (const doc of DOCUMENTS) {
  const target = join(SITE, doc.out);
  const next = render(doc);
  mkdirSync(dirname(target), { recursive: true });

  const previous = existsSync(target) ? readFileSync(target, "utf8") : null;
  if (previous !== next) {
    if (CHECK_ONLY) {
      stale.push(doc.source);
    } else {
      writeFileSync(target, next);
    }
  }
  written.push(`${doc.source} -> site/${doc.out}`);
}

// Report a generated page whose source document has gone, in the other
// direction. Leaving it would publish a document nobody can edit.
const generated = new Set(DOCUMENTS.map((d) => dirname(d.out).split("/")[0]));
for (const entry of readdirSync(SITE, { withFileTypes: true })) {
  if (!entry.isDirectory()) continue;
  if (!["spec", "architecture", "benchmarks", "writing-rules", "roadmap", "changelog"].includes(entry.name)) {
    continue;
  }
  if (!generated.has(entry.name) && existsSync(join(SITE, entry.name, "index.md"))) {
    throw new Error(
      `sync-docs: site/${entry.name}/index.md exists but no document in this repository is published there. ` +
        "Delete the directory or add it to DOCUMENTS.",
    );
  }
}

if (CHECK_ONLY) {
  if (stale.length > 0) {
    throw new Error(
      [
        `synced documents are out of date: ${stale.join(", ")}`,
        "",
        "Run `npm run sync` and commit the result.",
      ].join("\n"),
    );
  }
  console.log(`sync-docs: ${written.length} documents checked, all current`);
} else {
  console.log(`sync-docs: ${written.length} documents synced from the repository`);
  for (const line of written) console.log(`  ${line}`);
}