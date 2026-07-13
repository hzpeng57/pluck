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
      await page.emulateMedia({ reducedMotion: "reduce" });
      await page.goto(locale);
      await page.waitForLoadState("networkidle");

      const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      expect(overflow).toBeLessThanOrEqual(1);
      await expect(page.locator("#features")).toBeInViewport({ ratio: 0.01 });
      await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
      await expect(page.getByRole("link", { name: /download|下载/i }).first()).toBeVisible();
      await expect.poll(() => page.locator("[data-reveal]").evaluateAll(
        (elements) => elements.every((element) => getComputedStyle(element).opacity === "1"),
      )).toBe(true);

      if (testInfo.project.name === "desktop") {
        await page.screenshot({
          path: testInfo.outputPath(`${viewport.name}.png`),
          fullPage: true,
        });
      }
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
