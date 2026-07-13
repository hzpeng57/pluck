# Pluck Website Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and publish a bilingual, SEO-first Pluck product website at `https://hzpeng57.github.io/pluck/`, with real product media and download-first conversion.

**Architecture:** Add an HTML-first Vite multi-page site under `website/`, with English at `/pluck/` and Simplified Chinese at `/pluck/zh-CN/`. The site shares the repository toolchain but has isolated source, build, tests, and GitHub Pages deployment; all primary content and navigation work without JavaScript.

**Tech Stack:** Vite 6, TypeScript 5.6, semantic HTML, authored CSS, Lucide icons, Playwright, Sharp, Lighthouse CI, GitHub Pages Actions.

**Spec:** `docs/superpowers/specs/2026-07-13-pluck-website-design.md`

## Global Constraints

- Keep all website source under `website/`; do not refactor `src/` or `src-tauri/` while implementing the site.
- Preserve existing `pnpm dev`, `pnpm build`, `pnpm tauri dev`, and release workflow behavior.
- Use `/pluck/` as the production base path and English as the root language.
- Render complete English and Simplified Chinese content in initial HTML; language switching must use normal links and work without JavaScript.
- The primary CTA is the latest Apple Silicon release; GitHub is secondary.
- State that Pluck is inspired by Git workflows in JetBrains IDEs and is not affiliated with JetBrains.
- Use Developer Energy visual direction: dark neutral surfaces, mint primary accent, restrained graph colors, real app imagery, no decorative orbs, no large purple-blue gradient background.
- Use only real, sanitized Pluck screenshots; never expose private repository names, usernames, branches, commits, or source code.
- Do not add analytics, accounts, a backend, blog, docs system, testimonials, download metrics, Intel downloads, or Windows downloads.
- Respect `prefers-reduced-motion`, WCAG AA contrast, keyboard navigation, 200% zoom, and stable media dimensions.
- Desktop Lighthouse thresholds: Performance >= 0.95, Accessibility >= 0.95, Best Practices >= 0.95, SEO = 1.00.
- Mobile Lighthouse thresholds: Performance >= 0.90, Accessibility >= 0.95, Best Practices >= 0.95, SEO = 1.00.
- Run the full root `pnpm build` once during final verification, not after every small website edit.
- Commit messages are Chinese and follow the repository's conventional style; never push without explicit user confirmation.
- Execute implementation on an isolated feature branch/worktree so the existing untracked `docs/superpowers/plans/2026-07-08-side-by-side-diff-file-actions.md` remains untouched.

---

## File Structure

```text
package.json                                      Website scripts and dependencies
pnpm-lock.yaml                                   Resolved website tooling
.gitignore                                       Website build/test artifacts
.github/workflows/pages.yml                      GitHub Pages deployment
README.md                                        English website link
README.zh-CN.md                                  Chinese website link
website/
  index.html                                     Complete English page
  zh-CN/index.html                               Complete Chinese page
  404.html                                       Static bilingual recovery page
  vite.config.ts                                 Base path, MPA inputs, version injection
  tsconfig.json                                  Website TypeScript boundary
  playwright.config.ts                           Desktop/mobile browser test matrix
  lighthouserc.desktop.cjs                       Desktop performance gates
  lighthouserc.mobile.cjs                        Mobile performance gates
  src/
    main.ts                                      Icons, mobile menu, reveal enhancement
    styles.css                                   Developer Energy design system/layout
  public/
    favicon.png                                  Reused Pluck icon
    favicon.ico                                  Reused Pluck icon
    robots.txt                                   Crawler policy and sitemap URL
    sitemap.xml                                  Bilingual canonical URL inventory
    images/
      workspace-{1280,1920}.{avif,webp,png}      Main app screenshot variants
      diff-{1280,1920}.{avif,webp,png}           Side-by-side diff variants
      rebase-{1280,1920}.{avif,webp,png}         Interactive rebase variants
    og/pluck-cover.png                           1200x630 social image
  scripts/
    create-demo-repo.sh                          Reproducible sanitized screenshot repo
    optimize-images.mjs                          Image variants and social image
  tests/
    smoke.spec.ts                                Language routes and no-JS baseline
    seo.spec.ts                                  Metadata, JSON-LD, sitemap, canonical
    interaction.spec.ts                          Menu, focus, reduced motion
    media.spec.ts                                Image loading and source formats
    visual.spec.ts                               Responsive layout and screenshot capture
```

### Shared Interfaces

`website/vite.config.ts` owns these build constants:

```ts
export const SITE_BASE = "/pluck/";
export const SITE_ORIGIN = "https://hzpeng57.github.io";
export const SITE_URL = `${SITE_ORIGIN}${SITE_BASE}`;
```

The build replaces every literal `__PLUCK_VERSION__` in HTML with the version from root `package.json`. Both pages use these stable DOM hooks:

```html
<button data-menu-toggle aria-controls="site-navigation" aria-expanded="false"></button>
<nav id="site-navigation" data-site-navigation></nav>
<main id="main-content"></main>
<section id="features"></section>
<section id="workflow"></section>
<section id="download"></section>
<div data-reveal></div>
```

Tests use those hooks rather than styling classes.

---

### Task 1: Create the isolated website build and browser-test foundation

**Files:**
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`
- Modify: `.gitignore`
- Create: `website/vite.config.ts`
- Create: `website/tsconfig.json`
- Create: `website/playwright.config.ts`
- Create: `website/tests/smoke.spec.ts`
- Create: `website/index.html`
- Create: `website/zh-CN/index.html`
- Create: `website/404.html`
- Create: `website/src/main.ts`
- Create: `website/src/styles.css`

**Interfaces:**
- Consumes: root package version and existing Vite installation.
- Produces: `SITE_BASE`, `SITE_ORIGIN`, `SITE_URL`, `__PLUCK_VERSION__` HTML replacement, `pnpm website:*` commands, and browser-test base URL `http://127.0.0.1:4173/pluck/`.

- [ ] **Step 1: Create an isolated implementation worktree**

Use `superpowers:using-git-worktrees` before changing code. Create branch `codex/pluck-website` from the commit containing this plan, without tracking an upstream. Verify the user's untracked plan file remains only in the original worktree.

Expected: `git branch --show-current` prints `codex/pluck-website`; `git status --short` in the new worktree is clean.

- [ ] **Step 2: Add website tooling and exact scripts**

Run:

```bash
pnpm add lucide
pnpm add -D @playwright/test @lhci/cli @types/node sharp
pnpm exec playwright install chromium
```

Add these root scripts without changing existing scripts:

```json
{
  "website:dev": "vite --config website/vite.config.ts",
  "website:build": "tsc -p website/tsconfig.json --noEmit && vite build --config website/vite.config.ts",
  "website:preview": "vite preview --config website/vite.config.ts",
  "website:test": "pnpm website:build && playwright test -c website/playwright.config.ts",
  "website:lighthouse:desktop": "lhci autorun --config=website/lighthouserc.desktop.cjs",
  "website:lighthouse:mobile": "lhci autorun --config=website/lighthouserc.mobile.cjs"
}
```

Append these generated paths to `.gitignore`:

```gitignore
website/dist/
website/test-results/
website/.lighthouse/
```

- [ ] **Step 3: Write the failing route smoke tests**

Create `website/playwright.config.ts`:

```ts
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  outputDir: "./test-results",
  fullyParallel: true,
  use: {
    baseURL: "http://127.0.0.1:4173/pluck/",
    trace: "on-first-retry",
  },
  webServer: {
    command: "pnpm website:preview -- --host 127.0.0.1 --port 4173",
    url: "http://127.0.0.1:4173/pluck/",
    reuseExistingServer: true,
  },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    { name: "mobile", use: { ...devices["iPhone 13"] } },
  ],
});
```

Create `website/tests/smoke.spec.ts`:

```ts
import { expect, test } from "@playwright/test";

const pages = [
  { path: "./", lang: "en", heading: "Pluck" },
  { path: "zh-CN/", lang: "zh-CN", heading: "Pluck" },
] as const;

for (const entry of pages) {
  test(`${entry.lang} page is static and readable`, async ({ browser, page }) => {
    await page.goto(entry.path);
    await expect(page.locator("html")).toHaveAttribute("lang", entry.lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(entry.heading);

    const noJs = await browser.newContext({ javaScriptEnabled: false });
    const noJsPage = await noJs.newPage();
    await noJsPage.goto(new URL(entry.path, "http://127.0.0.1:4173/pluck/").toString());
    await expect(noJsPage.getByRole("main")).toBeVisible();
    await expect(noJsPage.getByRole("link", { name: /download|下载/i }).first()).toBeVisible();
    await noJs.close();
  });
}
```

- [ ] **Step 4: Run the smoke test and verify it fails**

Run:

```bash
pnpm website:test --project=desktop website/tests/smoke.spec.ts
```

Expected: FAIL because `website/vite.config.ts` or the language HTML pages do not exist yet.

- [ ] **Step 5: Add the Vite MPA boundary and minimal semantic pages**

Create `website/tsconfig.json`:

```json
{
  "extends": "../tsconfig.json",
  "compilerOptions": {
    "composite": false,
    "types": ["vite/client", "node"]
  },
  "include": ["./src/**/*.ts", "./tests/**/*.ts", "./vite.config.ts", "./playwright.config.ts"]
}
```

Create `website/vite.config.ts` with this implementation:

```ts
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { defineConfig, type Plugin } from "vite";

export const SITE_BASE = "/pluck/";
export const SITE_ORIGIN = "https://hzpeng57.github.io";
export const SITE_URL = `${SITE_ORIGIN}${SITE_BASE}`;

const websiteRoot = fileURLToPath(new URL(".", import.meta.url));
const packagePath = fileURLToPath(new URL("../package.json", import.meta.url));
const packageVersion = JSON.parse(readFileSync(packagePath, "utf8")).version as string;

function injectVersion(): Plugin {
  return {
    name: "pluck-website-version",
    transformIndexHtml(html) {
      return html.replaceAll("__PLUCK_VERSION__", packageVersion);
    },
  };
}

export default defineConfig({
  root: websiteRoot,
  base: SITE_BASE,
  plugins: [injectVersion()],
  build: {
    outDir: resolve(websiteRoot, "dist"),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        en: resolve(websiteRoot, "index.html"),
        zhCN: resolve(websiteRoot, "zh-CN/index.html"),
        notFound: resolve(websiteRoot, "404.html"),
      },
    },
  },
});
```

Each initial HTML page must include one H1, a visible download link, a `<main id="main-content">`, `/src/styles.css`, and `/src/main.ts`. Use `<html lang="en">` for `index.html` and `<html lang="zh-CN">` for `zh-CN/index.html`. Create a minimal bilingual `404.html` with links to `/pluck/` and `/pluck/zh-CN/`.

Create empty, valid `website/src/main.ts` and this stable baseline in `website/src/styles.css`:

```css
:root { color-scheme: dark; }
html { scroll-behavior: smooth; }
body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif; }
```

- [ ] **Step 6: Verify build isolation and passing smoke tests**

Run:

```bash
pnpm website:test --project=desktop website/tests/smoke.spec.ts
```

Expected: 2 passed. Then run `git status --short` and verify no files under `src/` or `src-tauri/` changed.

- [ ] **Step 7: Commit the website foundation**

```bash
git add package.json pnpm-lock.yaml .gitignore website
git commit -m "chore(website): 搭建官网构建与测试入口"
```

---

### Task 2: Add complete bilingual content and SEO metadata

**Files:**
- Modify: `website/index.html`
- Modify: `website/zh-CN/index.html`
- Modify: `website/404.html`
- Create: `website/public/robots.txt`
- Create: `website/public/sitemap.xml`
- Create: `website/tests/seo.spec.ts`
- Copy: `src-tauri/app/icons/32x32.png` to `website/public/favicon.png`
- Copy: `src-tauri/app/icons/icon.ico` to `website/public/favicon.ico`

**Interfaces:**
- Consumes: `SITE_URL`, `__PLUCK_VERSION__`, stable section IDs and language paths from Task 1.
- Produces: indexable English/Chinese content, canonical and alternate links, `SoftwareApplication` JSON-LD, sitemap, robots policy, and static favicon URLs.

- [ ] **Step 1: Write failing SEO and content tests**

Create `website/tests/seo.spec.ts`:

```ts
import { expect, test } from "@playwright/test";

const cases = [
  {
    path: "./",
    lang: "en",
    title: "Pluck - A focused Git client for macOS",
    canonical: "https://hzpeng57.github.io/pluck/",
    alternate: "https://hzpeng57.github.io/pluck/zh-CN/",
  },
  {
    path: "zh-CN/",
    lang: "zh-CN",
    title: "Pluck - 专注于 macOS 的 Git 客户端",
    canonical: "https://hzpeng57.github.io/pluck/zh-CN/",
    alternate: "https://hzpeng57.github.io/pluck/",
  },
] as const;

for (const entry of cases) {
  test(`${entry.lang} metadata and content`, async ({ page }) => {
    await page.goto(entry.path);
    await expect(page).toHaveTitle(entry.title);
    await expect(page.locator('meta[name="description"]')).toHaveAttribute("content", /Pluck|Git/);
    await expect(page.locator('link[rel="canonical"]')).toHaveAttribute("href", entry.canonical);
    await expect(page.locator('link[rel="alternate"][hreflang]')).toHaveCount(3);
    await expect(page.locator(`link[rel="alternate"][href="${entry.alternate}"]`)).toHaveCount(1);
    await expect(page.locator('meta[property="og:image"]')).toHaveAttribute(
      "content",
      "https://hzpeng57.github.io/pluck/og/pluck-cover.png",
    );
    await expect(page.locator("h1")).toHaveCount(1);
    await expect(page.locator("#features, #workflow, #download")).toHaveCount(3);

    const jsonLd = JSON.parse(await page.locator('script[type="application/ld+json"]').textContent() ?? "{}");
    expect(jsonLd["@type"]).toBe("SoftwareApplication");
    expect(jsonLd.applicationCategory).toBe("DeveloperApplication");
    expect(jsonLd.operatingSystem).toBe("macOS");
    expect(jsonLd.softwareVersion).toMatch(/^\d+\.\d+\.\d+$/);
    expect(jsonLd.downloadUrl).toBe("https://github.com/hzpeng57/pluck/releases/latest");
  });
}

test("crawler files are published", async ({ request }) => {
  const robots = await request.get("http://127.0.0.1:4173/pluck/robots.txt");
  expect(await robots.text()).toContain("Sitemap: https://hzpeng57.github.io/pluck/sitemap.xml");
  const sitemap = await request.get("http://127.0.0.1:4173/pluck/sitemap.xml");
  expect(await sitemap.text()).toContain("https://hzpeng57.github.io/pluck/zh-CN/");
});
```

- [ ] **Step 2: Run the SEO tests and verify they fail**

Run:

```bash
pnpm website:test --project=desktop website/tests/seo.spec.ts
```

Expected: FAIL on missing canonical, alternate, JSON-LD, sections, and crawler files.

- [ ] **Step 3: Write the complete English page**

Use semantic `header`, `nav`, `main`, six full-width sections, and `footer`. The exact visible section titles and CTAs are:

```text
Hero: Pluck / Git with developer muscle memory.
Hero description: A focused macOS Git client, inspired by the Git workflows in JetBrains IDEs.
Primary CTA: Download for Apple Silicon
Secondary CTA: View on GitHub

Features: Familiar workflows. Full context.
Workflow 1: Visual history
Workflow 2: Review-ready diff
Workflow 3: Interactive rebase
Workflow 4: Safe branch actions

Principles: Familiar by design / Git underneath / Focused on the work
Download: Built for Apple Silicon
Open source: Built in the open
```

The download section must state `Personal-use alpha`, `macOS`, `Apple Silicon`, DMG installation steps, and the exact command. Do not claim a minimum macOS version because the application configuration does not currently declare one.

```bash
xattr -dr com.apple.quarantine /Applications/Pluck.app
```

Use `https://github.com/hzpeng57/pluck/releases/latest` for all download/latest-release links and `https://github.com/hzpeng57/pluck` for repository links. Add the disclaimer:

```text
Pluck is an independent project and is not affiliated with, endorsed by, or sponsored by JetBrains.
```

- [ ] **Step 4: Write the complete Simplified Chinese page with parity**

Mirror the same DOM structure and IDs. The exact visible section titles and CTAs are:

```text
Hero: Pluck / 让熟悉的 Git 操作成为肌肉记忆。
Hero description: 一款专注于 macOS 的 Git 客户端，操作方式参考 JetBrains 系列 IDE 的 Git 工作流。
Primary CTA: 下载 Apple Silicon 版本
Secondary CTA: 在 GitHub 查看

Features: 熟悉的工作流，完整的上下文。
Workflow 1: 可视化历史
Workflow 2: 为审阅而生的 Diff
Workflow 3: 交互式 Rebase
Workflow 4: 安全的分支操作

Principles: 熟悉的设计 / Git 作为底层 / 专注于工作本身
Download: 为 Apple Silicon 构建
Open source: 开放开发
```

Include the same alpha, platform, installation, command, repository, release, and non-affiliation facts in natural Chinese.

- [ ] **Step 5: Add complete per-language metadata**

Each page must include:

```html
<link rel="canonical" href="LANGUAGE_CANONICAL_URL">
<link rel="alternate" hreflang="en" href="https://hzpeng57.github.io/pluck/">
<link rel="alternate" hreflang="zh-CN" href="https://hzpeng57.github.io/pluck/zh-CN/">
<link rel="alternate" hreflang="x-default" href="https://hzpeng57.github.io/pluck/">
<meta property="og:type" content="website">
<meta property="og:site_name" content="Pluck">
<meta property="og:image" content="https://hzpeng57.github.io/pluck/og/pluck-cover.png">
<meta name="twitter:card" content="summary_large_image">
```

Add language-specific title, description, Open Graph title/description/locale, Twitter title/description, and `SoftwareApplication` JSON-LD. Use `__PLUCK_VERSION__` for `softwareVersion`, `true` for `isAccessibleForFree`, and the repository URL in `sameAs`. Do not add ratings, reviews, pricing, or organization claims.

- [ ] **Step 6: Add crawler files, favicon, and useful 404 content**

Create `website/public/robots.txt`:

```text
User-agent: *
Allow: /

Sitemap: https://hzpeng57.github.io/pluck/sitemap.xml
```

Create `website/public/sitemap.xml` with this exact content:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">
  <url>
    <loc>https://hzpeng57.github.io/pluck/</loc>
    <xhtml:link rel="alternate" hreflang="en" href="https://hzpeng57.github.io/pluck/" />
    <xhtml:link rel="alternate" hreflang="zh-CN" href="https://hzpeng57.github.io/pluck/zh-CN/" />
    <xhtml:link rel="alternate" hreflang="x-default" href="https://hzpeng57.github.io/pluck/" />
  </url>
  <url>
    <loc>https://hzpeng57.github.io/pluck/zh-CN/</loc>
    <xhtml:link rel="alternate" hreflang="en" href="https://hzpeng57.github.io/pluck/" />
    <xhtml:link rel="alternate" hreflang="zh-CN" href="https://hzpeng57.github.io/pluck/zh-CN/" />
    <xhtml:link rel="alternate" hreflang="x-default" href="https://hzpeng57.github.io/pluck/" />
  </url>
</urlset>
```

Copy existing icon assets with `cp`, not by recreating them:

```bash
cp src-tauri/app/icons/32x32.png website/public/favicon.png
cp src-tauri/app/icons/icon.ico website/public/favicon.ico
```

Expand `404.html` with English and Chinese headings and links to both language roots and GitHub. Keep the document `noindex`.

- [ ] **Step 7: Run SEO, smoke, and build-output checks**

Run:

```bash
pnpm website:test --project=desktop website/tests/smoke.spec.ts website/tests/seo.spec.ts
```

Expected: 4 tests pass. Inspect `website/dist/index.html` and `website/dist/zh-CN/index.html`; neither contains `__PLUCK_VERSION__`, and both contain complete body copy.

- [ ] **Step 8: Commit bilingual content and SEO**

```bash
git add website
git commit -m "feat(website): 添加中英文内容与 SEO 元数据"
```

---

### Task 3: Implement Developer Energy visual design and accessible interactions

**Files:**
- Modify: `website/src/styles.css`
- Modify: `website/src/main.ts`
- Modify: `website/index.html`
- Modify: `website/zh-CN/index.html`
- Create: `website/tests/interaction.spec.ts`

**Interfaces:**
- Consumes: Task 2 semantic HTML, stable DOM hooks, and section IDs.
- Produces: `data-menu-toggle` behavior, Lucide icon hydration, `data-reveal` progressive enhancement, keyboard-safe navigation, and Developer Energy responsive styling.

- [ ] **Step 1: Write failing interaction and accessibility tests**

Create `website/tests/interaction.spec.ts`:

```ts
import { expect, test } from "@playwright/test";

test("mobile navigation manages state and focus", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("./");
  const toggle = page.locator("[data-menu-toggle]");
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("[data-site-navigation]")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  await expect(toggle).toBeFocused();
});

test("keyboard users can reach primary actions", async ({ page }) => {
  await page.goto("./");
  await page.keyboard.press("Tab");
  await expect(page.locator(":focus-visible")).toBeVisible();
  await expect(page.getByRole("link", { name: "Download for Apple Silicon" })).toHaveAttribute(
    "href",
    "https://github.com/hzpeng57/pluck/releases/latest",
  );
});

test("reduced motion disables reveal transitions", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("./");
  const duration = await page.locator("[data-reveal]").first().evaluate(
    (element) => getComputedStyle(element).transitionDuration,
  );
  expect(duration).toBe("0s");
});
```

- [ ] **Step 2: Run interaction tests and verify they fail**

Run:

```bash
pnpm website:test --project=mobile website/tests/interaction.spec.ts
```

Expected: FAIL because menu behavior, focus restoration, icons, and reduced-motion rules are absent.

- [ ] **Step 3: Implement the progressive enhancement controller**

Create `website/src/main.ts` with these exact responsibilities:

```ts
import {
  ArrowRight,
  Check,
  Download,
  ExternalLink,
  Github,
  Languages,
  Menu,
  X,
  createIcons,
} from "lucide";

document.documentElement.classList.add("js");

createIcons({
  icons: { ArrowRight, Check, Download, ExternalLink, Github, Languages, Menu, X },
  attrs: { "aria-hidden": "true", "stroke-width": "1.8" },
});

const menuButton = document.querySelector<HTMLButtonElement>("[data-menu-toggle]");
const navigation = document.querySelector<HTMLElement>("[data-site-navigation]");

function setMenu(open: boolean): void {
  if (!menuButton || !navigation) return;
  menuButton.setAttribute("aria-expanded", String(open));
  navigation.toggleAttribute("data-open", open);
  document.body.toggleAttribute("data-menu-open", open);
}

menuButton?.addEventListener("click", () => {
  setMenu(menuButton.getAttribute("aria-expanded") !== "true");
});

navigation?.addEventListener("click", (event) => {
  if ((event.target as Element).closest("a")) setMenu(false);
});

window.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && menuButton?.getAttribute("aria-expanded") === "true") {
    setMenu(false);
    menuButton.focus();
  }
});

const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
if (!reduceMotion && "IntersectionObserver" in window) {
  const observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      entry.target.setAttribute("data-visible", "");
      observer.unobserve(entry.target);
    }
  }, { rootMargin: "0px 0px -8%", threshold: 0.12 });
  document.querySelectorAll("[data-reveal]").forEach((element) => observer.observe(element));
} else {
  document.querySelectorAll("[data-reveal]").forEach((element) => element.setAttribute("data-visible", ""));
}
```

- [ ] **Step 4: Implement the Developer Energy token and layout system**

Define these stable tokens at the top of `website/src/styles.css`:

```css
:root {
  color-scheme: dark;
  --bg: #0d0e10;
  --panel: #141615;
  --raised: #1a1d1b;
  --hover: #222622;
  --border: #303730;
  --border-soft: #242924;
  --fg: #f1f0ea;
  --fg-2: #bbb8ae;
  --fg-3: #80847a;
  --accent: #78d7bd;
  --accent-2: #a7ead8;
  --blue: #8caee8;
  --pink: #e596bc;
  --yellow: #dfbd68;
  --green: #86d49c;
  --danger: #ee7f7a;
  --max-content: 1180px;
  --max-copy: 720px;
  --radius: 8px;
  --mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}
```

Implement, in this order:

1. Reset, selection, focus-visible, skip link, typography, link and button states.
2. Compact sticky navigation with desktop links and a 44x44 mobile icon button.
3. Hero grid with brand first, download/GitHub actions, real-media slot, and a visible hint of the feature band at 812px height.
4. Full-width workflow bands alternating copy and media; no section-level floating cards.
5. Four repeated workflow cards at radius 8px or less.
6. Git graph decorative line built with CSS borders and small status nodes, never overlapping text.
7. Installation command block that wraps or scrolls without overflowing.
8. Footer and disclaimer.
9. Breakpoints at 960px and 640px, plus safe-area padding.
10. `@media (prefers-reduced-motion: reduce)` setting animation and transition durations to `0s`.

Use `font-size` values with explicit breakpoints or bounded `clamp()` values; do not use viewport-width-only font scaling and keep `letter-spacing: 0`.

- [ ] **Step 5: Add complete navigation and icon markup to both pages**

Both pages must use the same structure:

```html
<a class="skip-link" href="#main-content">Skip to content</a>
<header class="site-header">
  <a class="brand" href="/pluck/" aria-label="Pluck home">...</a>
  <button class="menu-toggle" type="button" data-menu-toggle aria-controls="site-navigation"
          aria-expanded="false" aria-label="Open navigation">
    <i data-lucide="menu" data-icon-open></i>
    <i data-lucide="x" data-icon-close></i>
  </button>
  <nav id="site-navigation" data-site-navigation aria-label="Primary">...</nav>
</header>
```

Translate accessible labels on the Chinese page. Add `data-reveal` only to content wrappers; content remains visible by default, and `.js` opts into pre-reveal styling.

- [ ] **Step 6: Verify interaction, no-JS, and responsive layout behavior**

Run:

```bash
pnpm website:test website/tests/smoke.spec.ts website/tests/interaction.spec.ts
```

Expected: all tests pass in desktop and mobile projects. Manually tab through both pages at 200% zoom and verify no control is clipped.

- [ ] **Step 7: Commit visual design and interactions**

```bash
git add website
git commit -m "feat(website): 完成 Developer Energy 视觉与交互"
```

---

### Task 4: Capture sanitized product media and integrate responsive images

**Files:**
- Create: `website/scripts/create-demo-repo.sh`
- Create: `website/scripts/optimize-images.mjs`
- Create: `website/tests/media.spec.ts`
- Create: `website/public/images/*`
- Create: `website/public/og/pluck-cover.png`
- Modify: `website/index.html`
- Modify: `website/zh-CN/index.html`

**Interfaces:**
- Consumes: real Pluck application, `sharp`, Task 3 media slots, and `/tmp/pluck-website-captures/*.png` inputs.
- Produces: deterministic sanitized demo repository, AVIF/WebP/PNG image variants, 1200x630 OG image, and responsive `<picture>` markup.

- [ ] **Step 1: Write failing media tests**

Create `website/tests/media.spec.ts`:

```ts
import { expect, test } from "@playwright/test";

for (const path of ["./", "zh-CN/"]) {
  test(`${path} product media loads`, async ({ page }) => {
    await page.goto(path);
    const pictures = page.locator("picture[data-product-media]");
    await expect(pictures).toHaveCount(3);
    await expect(pictures.locator('source[type="image/avif"]')).toHaveCount(3);
    await expect(pictures.locator('source[type="image/webp"]')).toHaveCount(3);
    for (const image of await pictures.locator("img").all()) {
      await expect(image).toHaveAttribute("width", /\d+/);
      await expect(image).toHaveAttribute("height", /\d+/);
      expect(await image.evaluate((node) => node.naturalWidth)).toBeGreaterThan(0);
    }
  });
}

test("social image is published at the metadata URL", async ({ request }) => {
  const response = await request.get("http://127.0.0.1:4173/pluck/og/pluck-cover.png");
  expect(response.ok()).toBeTruthy();
  expect(response.headers()["content-type"]).toContain("image/png");
});
```

- [ ] **Step 2: Run media tests and verify they fail**

Run:

```bash
pnpm website:test --project=desktop website/tests/media.spec.ts
```

Expected: FAIL because responsive product assets do not exist.

- [ ] **Step 3: Create the reproducible demo Git repository script**

Create `website/scripts/create-demo-repo.sh` as this executable script. It deletes and recreates only `/tmp/pluck-website-demo`, configures a repository-local demo identity, and creates invented source content:

```bash
#!/usr/bin/env bash
set -euo pipefail

repo="/tmp/pluck-website-demo"
rm -rf "$repo"
mkdir -p "$repo/src/components"

git -C "$repo" init -b main
git -C "$repo" config user.name "Pluck Demo"
git -C "$repo" config user.email "demo@pluck.local"

printf '%s\n' '# Pluck Website Demo' 'Sanitized repository used for product screenshots.' > "$repo/README.md"
printf '%s\n' '<template>' '  <main>Repository workspace</main>' '</template>' > "$repo/src/App.vue"
printf '%s\n' '<template>' '  <pre class="diff">Unified diff</pre>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add README.md src/App.vue src/components/DiffViewer.vue
git -C "$repo" commit -m "Initial workspace"

printf '%s\n' '<template>' '  <aside>Repository switcher</aside>' '</template>' > "$repo/src/components/RepoSwitcher.vue"
git -C "$repo" add src/components/RepoSwitcher.vue
git -C "$repo" commit -m "Add repository switcher"

git -C "$repo" switch -c feature/diff
printf '%s\n' '<template>' '  <div class="split-diff">Side-by-side diff</div>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add src/components/DiffViewer.vue
git -C "$repo" commit -m "Add side-by-side diff"
printf '%s\n' '<script setup lang="ts">' 'const ignoreWhitespace = true;' '</script>' '<template>' '  <div class="split-diff">Side-by-side diff</div>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add src/components/DiffViewer.vue
git -C "$repo" commit -m "Ignore whitespace changes"

git -C "$repo" switch main
git -C "$repo" switch -c feature/rebase
printf '%s\n' '<template>' '  <dialog open>Interactive rebase</dialog>' '</template>' > "$repo/src/components/RebaseTodoDialog.vue"
git -C "$repo" add src/components/RebaseTodoDialog.vue
git -C "$repo" commit -m "Add interactive rebase editor"

git -C "$repo" switch main
printf '%s\n' '<template>' '  <pre class="diff">Unified diff with pending review</pre>' '</template>' > "$repo/src/components/DiffViewer.vue"

git -C "$repo" log --oneline --all --decorate --graph
git -C "$repo" status --short
```

Run `chmod +x website/scripts/create-demo-repo.sh` after creating it. The resulting history is:

```text
main:              Initial workspace
main:              Add repository switcher
feature/diff:      Add side-by-side diff
feature/diff:      Ignore whitespace changes
feature/rebase:    Add interactive rebase editor
main working tree: modified src/components/DiffViewer.vue
```

Use a minimal Vue-shaped file tree containing only invented demo content. The script ends with:

```bash
git -C /tmp/pluck-website-demo log --oneline --all --decorate --graph
git -C /tmp/pluck-website-demo status --short
```

Expected: graph contains all listed commit subjects and status contains only the intended modified demo file.

- [ ] **Step 4: Capture three real Pluck states**

Run the demo script. Launch the real app with an isolated temporary `HOME` while retaining the installed Rust toolchains; this prevents existing Pluck/WebKit local storage from appearing in screenshots:

```bash
bash website/scripts/create-demo-repo.sh
mkdir -p /tmp/pluck-website-home /tmp/pluck-website-captures
HOME=/tmp/pluck-website-home CARGO_HOME=/Users/hzp/.cargo RUSTUP_HOME=/Users/hzp/.rustup RUST_LOG=pluck=debug pnpm tauri dev
```

In Pluck, add `/tmp/pluck-website-demo`. Capture only the Pluck window with macOS `screencapture -W`, saving these source files outside the repository:

```text
/tmp/pluck-website-captures/workspace.png   main log with branches and commit detail
/tmp/pluck-website-captures/diff.png        side-by-side diff with file navigation
/tmp/pluck-website-captures/rebase.png      interactive rebase dialog over the demo history
```

Use these commands after arranging each real app state:

```bash
screencapture -W /tmp/pluck-website-captures/workspace.png
screencapture -W /tmp/pluck-website-captures/diff.png
screencapture -W /tmp/pluck-website-captures/rebase.png
```

Before continuing, inspect all three images. Reject and recapture any image containing another app, desktop notifications, personal repository data, personal account names, clipped controls, or unreadable content.

- [ ] **Step 5: Implement deterministic image optimization**

Create `website/scripts/optimize-images.mjs` with this implementation:

```js
import { mkdir, stat } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import sharp from "sharp";

const inputDir = resolve(process.argv[2] ?? "/tmp/pluck-website-captures");
const publicDir = fileURLToPath(new URL("../public/", import.meta.url));
const imageDir = resolve(publicDir, "images");
const ogDir = resolve(publicDir, "og");
const names = ["workspace", "diff", "rebase"];
const widths = [1280, 1920];

await mkdir(imageDir, { recursive: true });
await mkdir(ogDir, { recursive: true });

for (const name of names) {
  const input = resolve(inputDir, `${name}.png`);
  const metadata = await sharp(input).metadata();
  if (!metadata.width || metadata.width < 1920 || !metadata.height) {
    throw new Error(`${input} must be a valid screenshot at least 1920px wide`);
  }

  for (const width of widths) {
    const base = sharp(input).rotate().resize({ width, withoutEnlargement: true });
    await Promise.all([
      base.clone().avif({ quality: 62 }).toFile(resolve(imageDir, `${name}-${width}.avif`)),
      base.clone().webp({ quality: 82 }).toFile(resolve(imageDir, `${name}-${width}.webp`)),
      base.clone().png({ compressionLevel: 9 }).toFile(resolve(imageDir, `${name}-${width}.png`)),
    ]);
  }
}

const workspace = resolve(inputDir, "workspace.png");
const icon = await sharp(resolve(publicDir, "favicon.png")).resize(72, 72).png().toBuffer();
const overlay = Buffer.from(`
  <svg width="1200" height="630" xmlns="http://www.w3.org/2000/svg">
    <rect width="1200" height="630" fill="#0d0e10" fill-opacity="0.18"/>
    <rect width="540" height="630" fill="#0d0e10" fill-opacity="0.92"/>
    <text x="82" y="270" fill="#f1f0ea" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="72" font-weight="700">Pluck</text>
    <text x="82" y="330" fill="#78d7bd" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="30">Git with developer muscle memory.</text>
    <text x="82" y="382" fill="#bbb8ae" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="22">A focused Git client for macOS.</text>
  </svg>
`);

await sharp(workspace)
  .rotate()
  .resize(1200, 630, { fit: "cover", position: "centre" })
  .composite([
    { input: overlay, top: 0, left: 0 },
    { input: icon, top: 112, left: 82 },
  ])
  .png({ compressionLevel: 9 })
  .toFile(resolve(ogDir, "pluck-cover.png"));

for (const name of names) {
  for (const width of widths) {
    for (const format of ["avif", "webp", "png"]) {
      const output = resolve(imageDir, `${name}-${width}.${format}`);
      const metadata = await sharp(output).metadata();
      const size = await stat(output);
      console.log(`${output}: ${metadata.width}x${metadata.height}, ${size.size} bytes`);
    }
  }
}
console.log(`${resolve(ogDir, "pluck-cover.png")}: 1200x630`);
```

For each `workspace`, `diff`, and `rebase` source, the script:

1. Auto-orient and strip metadata.
2. Resize to widths 1280 and 1920 without enlargement.
3. Write AVIF at quality 62, WebP at quality 82, and PNG with compression level 9.
4. Preserve the source aspect ratio and print each output's width, height, and byte size.

Generate `website/public/og/pluck-cover.png` at exactly 1200x630 from the workspace capture. Composite a `#0d0e10` readability layer, the existing Pluck app icon, `Pluck`, and `Git with developer muscle memory.`; keep the real interface visibly identifiable on at least half the canvas.

Add root script:

```json
"website:images": "node website/scripts/optimize-images.mjs /tmp/pluck-website-captures"
```

Run `pnpm website:images`. Expected: 19 files are written: 18 responsive product variants plus one OG image.

- [ ] **Step 6: Integrate responsive `<picture>` elements**

Use exactly three `picture[data-product-media]` elements per language page. Each contains AVIF and WebP `srcset` entries for 1280w and 1920w, a PNG fallback, fixed source width/height, and language-specific alt text. The hero workspace image uses `fetchpriority="high"`; diff and rebase use `loading="lazy"` and `decoding="async"`.

Do not crop away the branch list, commit graph, diff gutters, or rebase controls on mobile. Use CSS `object-fit: contain` and a stable aspect ratio.

- [ ] **Step 7: Verify and visually inspect product media**

Run:

```bash
pnpm website:test website/tests/media.spec.ts
```

Expected: media tests pass in desktop and mobile projects. Use Playwright to capture 1440x1000 and 390x844 screenshots, then inspect both images with `view_image`; verify the product media is nonblank and text does not overlap.

- [ ] **Step 8: Commit sanitized product media**

```bash
git add package.json website
git commit -m "feat(website): 添加真实产品截图与响应式媒体"
```

---

### Task 5: Add Lighthouse gates, GitHub Pages deployment, and repository entry points

**Files:**
- Create: `website/lighthouserc.desktop.cjs`
- Create: `website/lighthouserc.mobile.cjs`
- Create: `.github/workflows/pages.yml`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Interfaces:**
- Consumes: `pnpm website:build`, `website/dist`, the production base path, and the exact Lighthouse thresholds in Global Constraints.
- Produces: reproducible quality gates, Pages artifact deployment, and discoverable website links from both READMEs.

- [ ] **Step 1: Create desktop and mobile Lighthouse configurations**

Create `website/lighthouserc.desktop.cjs`:

```js
module.exports = {
  ci: {
    collect: {
      startServerCommand: "pnpm website:preview -- --host 127.0.0.1 --port 4173",
      startServerReadyPattern: "Local:",
      url: [
        "http://127.0.0.1:4173/pluck/",
        "http://127.0.0.1:4173/pluck/zh-CN/",
      ],
      numberOfRuns: 1,
      settings: { preset: "desktop" },
    },
    assert: {
      assertions: {
        "categories:performance": ["error", { minScore: 0.95 }],
        "categories:accessibility": ["error", { minScore: 0.95 }],
        "categories:best-practices": ["error", { minScore: 0.95 }],
        "categories:seo": ["error", { minScore: 1 }],
      },
    },
    upload: {
      target: "filesystem",
      outputDir: "website/.lighthouse/desktop",
    },
  },
};
```

Create `website/lighthouserc.mobile.cjs`:

```js
module.exports = {
  ci: {
    collect: {
      startServerCommand: "pnpm website:preview -- --host 127.0.0.1 --port 4173",
      startServerReadyPattern: "Local:",
      url: [
        "http://127.0.0.1:4173/pluck/",
        "http://127.0.0.1:4173/pluck/zh-CN/",
      ],
      numberOfRuns: 1,
      settings: {
        formFactor: "mobile",
        screenEmulation: {
          mobile: true,
          width: 390,
          height: 844,
          deviceScaleFactor: 3,
          disabled: false,
        },
      },
    },
    assert: {
      assertions: {
        "categories:performance": ["error", { minScore: 0.9 }],
        "categories:accessibility": ["error", { minScore: 0.95 }],
        "categories:best-practices": ["error", { minScore: 0.95 }],
        "categories:seo": ["error", { minScore: 1 }],
      },
    },
    upload: {
      target: "filesystem",
      outputDir: "website/.lighthouse/mobile",
    },
  },
};
```

- [ ] **Step 2: Run Lighthouse and verify thresholds before adding deployment**

Run:

```bash
pnpm website:build
pnpm website:lighthouse:desktop
pnpm website:lighthouse:mobile
```

Expected: both language URLs meet every configured assertion. If an assertion fails, stop and correct the reported page, accessibility, metadata, or asset defect; do not lower thresholds.

- [ ] **Step 3: Add the GitHub Pages workflow**

Create `.github/workflows/pages.yml`:

```yaml
name: Pages

on:
  push:
    branches: [main]
    paths:
      - "website/**"
      - "package.json"
      - "pnpm-lock.yaml"
      - ".github/workflows/pages.yml"
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm website:build
      - uses: actions/configure-pages@v5
      - uses: actions/upload-pages-artifact@v3
        with:
          path: website/dist

  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

Do not modify `.github/workflows/release.yml`.

- [ ] **Step 4: Add bilingual repository entry points**

In `README.md`, add a centered `Website` link to `https://hzpeng57.github.io/pluck/` beside the language links. In `README.zh-CN.md`, add `官网` pointing to the same URL. Do not replace the existing Releases install instructions.

- [ ] **Step 5: Verify deployment artifacts and workflow scope**

Run:

```bash
pnpm website:build
find website/dist -maxdepth 3 -type f | sort
git diff --check
```

Expected output includes `index.html`, `zh-CN/index.html`, `404.html`, `robots.txt`, `sitemap.xml`, favicon files, OG image, product images, CSS, and JS. Confirm the workflow path is `website/dist` and the release workflow diff is empty.

- [ ] **Step 6: Commit quality gates and deployment configuration**

```bash
git add .github/workflows/pages.yml README.md README.zh-CN.md website/lighthouserc.desktop.cjs website/lighthouserc.mobile.cjs
git commit -m "chore(website): 添加 Pages 部署与质量门槛"
```

---

### Task 6: Add final responsive regression checks and complete verification

**Files:**
- Create: `website/tests/visual.spec.ts`
- Modify: `website/index.html`
- Modify: `website/zh-CN/index.html`
- Modify: `website/src/styles.css`
- Modify: `website/src/main.ts`

**Interfaces:**
- Consumes: complete website, Playwright desktop/mobile projects, Lighthouse configs, and root app build.
- Produces: explicit no-overflow and first-viewport coverage, local screenshot artifacts, and final verified implementation.

- [ ] **Step 1: Write final responsive regression tests**

Create `website/tests/visual.spec.ts`:

```ts
import { expect, test } from "@playwright/test";

const viewports = [
  { name: "mobile", width: 375, height: 812 },
  { name: "tablet", width: 768, height: 1024 },
  { name: "desktop", width: 1280, height: 900 },
  { name: "wide", width: 1440, height: 1000 },
] as const;

for (const locale of ["./", "zh-CN/"] as const) {
  for (const viewport of viewports) {
    test(`${locale} ${viewport.name} has stable layout`, async ({ page }, testInfo) => {
      await page.setViewportSize(viewport);
      await page.goto(locale);
      await page.waitForLoadState("networkidle");

      const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      expect(overflow).toBeLessThanOrEqual(1);
      await expect(page.locator("#features")).toBeInViewport({ ratio: 0.01 });
      await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
      await expect(page.getByRole("link", { name: /download|下载/i }).first()).toBeVisible();

      await page.screenshot({
        path: testInfo.outputPath(`${viewport.name}.png`),
        fullPage: true,
      });
    });
  }
}

test("all internal resources load without errors", async ({ page }) => {
  const failures: string[] = [];
  page.on("response", (response) => {
    if (response.url().includes("127.0.0.1") && response.status() >= 400) {
      failures.push(`${response.status()} ${response.url()}`);
    }
  });
  await page.goto("./");
  await page.waitForLoadState("networkidle");
  expect(failures).toEqual([]);
});
```

- [ ] **Step 2: Run the full website suite**

Run:

```bash
pnpm website:test
```

Expected: smoke, SEO, interaction, media, and visual tests pass in both Playwright projects. If a browser project duplicates viewport-specific cases, keep the assertions but remove only redundant screenshot generation, not coverage.

- [ ] **Step 3: Inspect desktop and mobile screenshots**

Open at least these two generated artifacts with `view_image`:

```text
English 1440x1000 full-page screenshot
Chinese 375x812 full-page screenshot
```

Verify: real app media is visible and nonblank; no text overlap; long Chinese copy wraps cleanly; navigation, buttons, command block, media, and footer stay within their containers; the next section is hinted in the first viewport.

- [ ] **Step 4: Run final build and SEO verification once**

Run:

```bash
pnpm build
pnpm website:build
pnpm website:lighthouse:desktop
pnpm website:lighthouse:mobile
git diff --check
```

Expected: root Vue/Tauri frontend build passes; website TypeScript and Vite build pass; all Lighthouse assertions pass; diff check reports no whitespace errors. Do not run Cargo checks because implementation does not change Rust.

- [ ] **Step 5: Review the final diff for scope and secrets**

Run:

```bash
git status --short
git diff --stat HEAD~5..HEAD
rg -n "/Users/|@taptap|workspace/|personal/|localhost" website README.md README.zh-CN.md .github/workflows/pages.yml
```

Expected: no private path, account, internal domain, or localhost reference appears in production HTML/assets; localhost may appear only in test/config files. Confirm no updater key, unrelated untracked file, `src/`, `src-tauri/`, or release workflow change is staged.

- [ ] **Step 6: Commit final regression coverage and any verified polish**

```bash
git add website/tests/visual.spec.ts website/index.html website/zh-CN/index.html website/src
git commit -m "test(website): 补充响应式与资源回归验证"
```

- [ ] **Step 7: Hand off without remote mutation**

Start `pnpm website:dev -- --host 127.0.0.1` on an available port and provide the full local `/pluck/` URL. Report commits, tests, Lighthouse scores, and any GitHub repository setting still needed. Ask separately before pushing the branch, enabling Pages, or changing the GitHub Homepage field.
