/**
 * VitePress configuration for the soroban-lint documentation site.
 *
 * The content lives in this repository and is published from its Pages site.
 *
 * Four things here are deliberate rather than defaults:
 *
 * 1. `base` is `/soroban-lint-core/`, matching this repository's Pages project
 *    path. Every internal link and asset URL is resolved through `base`, so the
 *    site works from a subdirectory and not only from a domain root.
 * 2. Search is `local`, so there is no search index hosted anywhere and no
 *    keystroke leaves the browser.
 * 3. `cleanUrls` is on and trailing slashes are kept, so `/rules/SL001` and
 *    `/rules/SL001/` both resolve. GitHub Pages serves the directory form.
 * 4. `editLink` points at this repository, which is what makes every page's
 *    "Edit this page" button land on the file that actually generates it.
 */

import { defineConfig } from "vitepress";

/**
 * Telegram's mark.
 *
 * VitePress's `VPSocialLink` renders a string `icon` as a CSS-masked
 * `<span class="vpi-social-<name>">`, and for an object it `v-html`s
 * `icon.svg` directly. An object is the escape hatch used here, so the SVG is
 * inlined and `fill="currentColor"` makes the mark inherit the link colour and
 * the focus ring exactly like the built-in social icons.
 *
 * Note for whoever picks this up next: `simple-icons`, which VitePress bundles,
 * *does* ship a `telegram` glyph, so `icon: "telegram"` would also work and be
 * inlined into `vp-icons.css` at build time. The object form is kept because it
 * was asked for explicitly, and because it means the icon does not depend on the
 * bundled icon set at all.
 */
const telegramIcon = {
  name: "telegram",
  svg: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true"><path fill="currentColor" d="M21.94 4.3 19.2 19.1c-.2 1-.83 1.24-1.68.77l-4.63-3.41-2.23 2.15c-.25.25-.46.46-.94.46l.33-4.8 8.74-7.9c.38-.34-.08-.53-.59-.19L6.66 12.9 2 11.4c-1.01-.32-1.03-1.01.21-1.5l19.1-7.36c.85-.31 1.6.2 1.63 1.76Z"/></svg>`,
};

const DISCORD = "https://discord.gg/xZRZT6TpB";
const TELEGRAM = "https://t.me/+MrTh9uraIS5jMjhk";
const CORE = "https://github.com/Stellar-Soroban-Lint/soroban-lint-core";
const ORG = "https://github.com/Stellar-Soroban-Lint";
const PORTAL = "https://github.com/Stellar-Soroban-Lint/soroban-lint-portal";
const PLAYGROUND = "https://stellar-soroban-lint.github.io/soroban-lint-portal/";
const ACTION = "https://github.com/Stellar-Soroban-Lint/soroban-lint-action";
const DEMO_PR = `${PORTAL}/pull/1`;
const ISSUES = `${CORE}/issues`;

export default defineConfig({
  title: "soroban-lint",
  description:
    "Syntactic, per-file static analysis for Soroban smart contracts. Not an audit, and it does not claim to be one.",

  base: "/soroban-lint-core/",
  lang: "en-US",
  cleanUrls: true,
  trailingSlash: true,
  srcExclude: ["rules-prose/**"],
  lastUpdated: true,

  sitemap: {
    hostname: "https://stellar-soroban-lint.github.io/soroban-lint-core/",
  },

  // Local search: the index is built into the site at build time. No Algolia
  // credentials, no hosted index, no query leaves the browser.
  search: {
    provider: "local",
    options: {
      // The scope statement is on every page, so matching any of these returns
      // most of the site. Searching for them should land on the rules index,
      // which is the one page that summarises what the tool does and does not do.
      _split: ["-", "_", "/"],
    },
  },

  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" }],
    ["link", { rel: "alternate icon", href: "/favicon.svg" }],
    ["meta", { name: "theme-color", content: "#7C3AED" }],
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:site_name", content: "soroban-lint" }],
    ["meta", { property: "og:title", content: "soroban-lint" }],
    [
      "meta",
      {
        property: "og:description",
        content:
          "Syntactic, per-file static analysis for Soroban smart contracts.",
      },
    ],
    [
      "meta",
      { property: "og:image", content: "https://stellar-soroban-lint.github.io/soroban-lint-core/og.png" },
    ],
    ["meta", { name: "twitter:card", content: "summary_large_image" }],
    [
      "meta",
      {
        name: "twitter:image",
        content: "https://stellar-soroban-lint.github.io/soroban-lint-core/og.png",
      },
    ],
  ],

  markdown: {
    // Every relative link in a synced document would break once the file moves
    // out of its original directory, so this is the last line of defence rather
    // than the fix. `sync-docs.mjs` rewrites the ones it knows about; anything
    // it misses fails the build here rather than shipping a dead link.
    //
    // Deliberately NOT set to `ignoreDeadLinks`. Step 7 requires zero dead links
    // with the check left switched on.
  },

  themeConfig: {
    logo: "/logo.svg",
    siteTitle: "soroban-lint",

    nav: [
      { text: "Home", link: "/" },
      { text: "Getting Started", link: "/getting-started" },
      { text: "Rules", link: "/rules/" },
      { text: "CLI", link: "/cli" },
      { text: "GitHub Action", link: "/github-action" },
      { text: "Playground", link: "/playground" },
      { text: "Live playground", link: PLAYGROUND },
      { text: "Architecture", link: "/architecture" },
      { text: "Benchmarks", link: "/benchmarks" },
      { text: "Contributing", link: "/contributing" },
      { text: "Community", link: "/community" },
      {
        text: "Repos",
        items: [
          { text: "soroban-lint-core", link: CORE },
          { text: "soroban-lint-action", link: ACTION },
          { text: "soroban-lint-portal", link: PORTAL },
          { text: "Demo pull request", link: DEMO_PR },
          { text: "Issue board", link: ISSUES },
        ],
      },
      {
        text: "Social",
        items: [
          { text: "Discord", link: DISCORD },
          { text: "Telegram", link: TELEGRAM },
        ],
      },
    ],

    socialLinks: [
      { icon: "discord", link: DISCORD, ariaLabel: "Discord" },
      { icon: telegramIcon, link: TELEGRAM, ariaLabel: "Telegram" },
      { icon: "github", link: ORG, ariaLabel: "Stellar-Soroban-Lint on GitHub" },
    ],

    sidebar: {
      // The docs that exist in this repository and are published as pages in
      // their own right. They are synced by `sync-docs.mjs`, so there is no
      // hand-written copy to drift.
      "/spec/": [
        {
          text: "Specification",
          items: [
            { text: "SPEC.md", link: "/spec/" },
          ],
        },
      ],
      "/writing-rules/": [
        {
          text: "Writing rules",
          items: [{ text: "How to add a rule", link: "/writing-rules/" }],
        },
      ],
      "/changelog/": [
        {
          text: "Changelog",
          items: [{ text: "Releases", link: "/changelog/" }],
        },
      ],
      "/roadmap/": [
        {
          text: "Roadmap",
          items: [{ text: "Not scheduled", link: "/roadmap/" }],
        },
      ],
      "/rules/": [
        {
          text: "Rules",
          items: [
            { text: "Overview", link: "/rules/" },
            { text: "SL001 missing-require-auth", link: "/rules/SL001" },
            { text: "SL002 panic-hazards", link: "/rules/SL002" },
            { text: "SL003 unchecked-arithmetic", link: "/rules/SL003" },
            { text: "SL004 unbounded-storage-growth", link: "/rules/SL004" },
            { text: "SL005 missing-ttl-extension", link: "/rules/SL005" },
            { text: "SL006 questionable-storage-type", link: "/rules/SL006" },
            { text: "SL007 unprotected-initializer", link: "/rules/SL007" },
            { text: "SL008 unsafe-and-no-std", link: "/rules/SL008" },
          ],
        },
      ],
    },

    // Every page offers "Edit this page". For a synced document this points at
    // the original file in this repository, which is the one a reader should
    // edit; for a generated rule page it points at the prose file, since the
    // page itself is rebuilt on every build.
    editLink: {
      pattern: "https://github.com/Stellar-Soroban-Lint/soroban-lint-core/edit/main/:path",
      text: "Edit this page on GitHub",
    },

    outline: { level: [2, 3], label: "On this page" },

    docFooter: { prev: "Previous", next: "Next" },
    darkModeSwitchLabel: "Appearance",
    lightModeSwitchTitle: "Switch to light theme",
    darkModeSwitchTitle: "Switch to dark theme",
    sidebarMenuLabel: "Menu",
    returnToTopLabel: "Return to top",

    footer: {
      message:
        `MIT OR Apache-2.0. <a href="${DISCORD}">Discord</a> · <a href="${TELEGRAM}">Telegram</a> · <a href="${ORG}">GitHub</a>.`,
      copyright:
        "A clean report is not evidence a contract is secure.",
    },
  },
});
