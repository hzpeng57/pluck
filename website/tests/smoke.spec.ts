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
