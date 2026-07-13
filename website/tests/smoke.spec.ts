import { expect, test } from "@playwright/test";

const pages = [
  { path: "./", lang: "en", heading: "Pluck" },
  { path: "zh-CN/", lang: "zh-CN", heading: "Pluck" },
] as const;

for (const entry of pages) {
  test(`${entry.lang} page is static and readable`, async ({ browser, page }, testInfo) => {
    await page.goto(entry.path);
    await expect(page.locator("html")).toHaveAttribute("lang", entry.lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(entry.heading);

    const { viewport, deviceScaleFactor, isMobile, hasTouch, userAgent } = testInfo.project.use;
    const noJs = await browser.newContext({
      javaScriptEnabled: false,
      viewport,
      deviceScaleFactor,
      isMobile,
      hasTouch,
      userAgent,
    });
    const noJsPage = await noJs.newPage();
    const noJsNavigationRequest = noJsPage.waitForRequest(
      (request) => request.isNavigationRequest() && request.frame() === noJsPage.mainFrame(),
    );
    await noJsPage.goto(new URL(entry.path, "http://127.0.0.1:4173/pluck/").toString());
    const noJsUserAgent = await (await noJsNavigationRequest).headerValue("user-agent");
    const noJsEnvironment = await noJsPage.evaluate(() => ({
      deviceScaleFactor: window.devicePixelRatio,
      hasTouch: navigator.maxTouchPoints > 0,
    }));
    expect(noJsPage.viewportSize()).toEqual(page.viewportSize());
    expect(noJsUserAgent).toBe(await page.evaluate(() => navigator.userAgent));
    expect(noJsEnvironment.deviceScaleFactor).toBe(deviceScaleFactor ?? 1);
    expect(noJsEnvironment.hasTouch).toBe(hasTouch ?? false);
    await expect(noJsPage.locator("html")).not.toHaveClass(/\bjs\b/);
    await expect(noJsPage.getByRole("main")).toBeVisible();
    await expect(noJsPage.getByRole("link", { name: /download|下载/i }).first()).toBeVisible();
    await noJs.close();
  });
}
