import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";

const licenseHref = "https://github.com/hzpeng57/pluck/blob/main/LICENSE";

test("repository publishes the standard MIT License", async () => {
  const licensePath = fileURLToPath(new URL("../../LICENSE", import.meta.url));
  const license = await readFile(licensePath, "utf8");

  expect(license).toContain("MIT License");
  expect(license).toContain("Copyright (c) 2026 hzpeng57");
  expect(license).toContain("Permission is hereby granted, free of charge");
  expect(license).toContain('THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND');
});

for (const entry of [
  {
    path: "./",
    openSourceLabel: "MIT License",
    footerLabel: "License",
    disclosure: "not yet signed with an Apple Developer ID",
    roadmap: "Developer ID signing and notarization are planned",
    noLongerNeeded: "xattr step will no longer be needed",
  },
  {
    path: "zh-CN/",
    openSourceLabel: "MIT \u8bb8\u53ef\u8bc1",
    footerLabel: "\u8bb8\u53ef\u8bc1",
    disclosure: "\u5c1a\u672a\u4f7f\u7528 Apple Developer ID \u7b7e\u540d",
    roadmap: "\u7b7e\u540d\u4e0e\u516c\u8bc1\u5728\u89c4\u5212\u4e2d",
    noLongerNeeded: "\u5c06\u4e0d\u518d\u9700\u8981 xattr \u6b65\u9aa4",
  },
] as const) {
  test(`${entry.path} discloses its license and unsigned download`, async ({ page }) => {
    await page.goto(entry.path);

    await expect(page.locator("#open-source").getByRole("link", { name: entry.openSourceLabel })).toHaveAttribute(
      "href",
      licenseHref,
    );
    await expect(page.locator(".site-footer").getByRole("link", { name: entry.footerLabel })).toHaveAttribute(
      "href",
      licenseHref,
    );

    const download = page.locator("#download");
    await expect(download).toContainText(entry.disclosure);
    await expect(download).toContainText(entry.roadmap);
    await expect(download).toContainText(entry.noLongerNeeded);
    await expect(download).toContainText("xattr -dr com.apple.quarantine /Applications/Pluck.app");
  });
}
