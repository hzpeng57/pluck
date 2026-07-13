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
