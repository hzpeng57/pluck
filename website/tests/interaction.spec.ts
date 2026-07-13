import { expect, test } from "@playwright/test";

test("mobile navigation manages state and focus", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("./");
  const toggle = page.locator("[data-menu-toggle]");
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  await expect(toggle).toHaveAttribute("aria-label", "Open navigation");
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  await expect(toggle).toHaveAttribute("aria-label", "Close navigation");
  await expect(page.locator("[data-site-navigation]")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  await expect(toggle).toHaveAttribute("aria-label", "Open navigation");
  await expect(toggle).toBeFocused();
});

test("mobile navigation uses localized state labels", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("zh-CN/");
  const toggle = page.locator("[data-menu-toggle]");
  await expect(toggle).toHaveAttribute("aria-label", "打开导航");
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-label", "关闭导航");
  await page.keyboard.press("Escape");
  await expect(toggle).toHaveAttribute("aria-label", "打开导航");
});

test("mobile navigation unlocks the page above its breakpoint", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("./");
  const toggle = page.locator("[data-menu-toggle]");
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("body")).toHaveAttribute("data-menu-open", "");

  await page.setViewportSize({ width: 1200, height: 812 });
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  await expect(page.locator("body")).not.toHaveAttribute("data-menu-open", "");
});

test("keyboard users can reach primary actions", async ({ page }) => {
  await page.goto("./");
  await page.keyboard.press("Tab");
  await expect(page.locator(":focus-visible")).toBeVisible();
  await expect(page.getByRole("link", { name: "Download for Apple Silicon" }).first()).toHaveAttribute(
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

test("mobile hero leaves the next section visible", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("./");
  const featuresTop = await page.locator("#features").evaluate(
    (element) => element.getBoundingClientRect().top,
  );
  expect(featuresTop).toBeLessThan(812);
});

test("standard motion reveals every content wrapper after scrolling", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await page.goto("./");
  const reveals = page.locator("[data-reveal]");
  await expect(reveals).toHaveCount(8);

  for (let index = 0; index < await reveals.count(); index += 1) {
    const reveal = reveals.nth(index);
    await reveal.evaluate((element) => element.scrollIntoView({ block: "center" }));
    await expect(reveal).toHaveAttribute("data-visible", "");
    await expect.poll(() => reveal.evaluate((element) => getComputedStyle(element).opacity)).toBe("1");
  }
});
