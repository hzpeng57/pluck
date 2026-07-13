import { expect, test } from "@playwright/test";

const cases = [
  {
    path: "./",
    lang: "en",
    title: "Pluck - A focused Git client for macOS",
    canonical: "https://hzpeng57.github.io/pluck/",
    alternate: "https://hzpeng57.github.io/pluck/zh-CN/",
    alternateLang: "zh-CN",
  },
  {
    path: "zh-CN/",
    lang: "zh-CN",
    title: "Pluck - 专注于 macOS 的 Git 客户端",
    canonical: "https://hzpeng57.github.io/pluck/zh-CN/",
    alternate: "https://hzpeng57.github.io/pluck/",
    alternateLang: "en",
  },
] as const;

for (const entry of cases) {
  test(`${entry.lang} metadata and content`, async ({ page }) => {
    await page.goto(entry.path);
    await expect(page).toHaveTitle(entry.title);
    await expect(page.locator('meta[name="description"]')).toHaveAttribute("content", /Pluck|Git/);
    await expect(page.locator('link[rel="canonical"]')).toHaveAttribute("href", entry.canonical);
    await expect(page.locator('link[rel="alternate"][hreflang]')).toHaveCount(3);
    await expect(
      page.locator(
        `link[rel="alternate"][hreflang="${entry.alternateLang}"][href="${entry.alternate}"]`,
      ),
    ).toHaveCount(1);
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
