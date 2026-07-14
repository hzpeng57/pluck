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
      expect(await image.evaluate((node) => (node as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
    }
  });
}

test("social image is published at the metadata URL", async ({ request }) => {
  const response = await request.get("http://127.0.0.1:4173/pluck/og/pluck-cover.png");
  expect(response.ok()).toBeTruthy();
  expect(response.headers()["content-type"]).toContain("image/png");
});
