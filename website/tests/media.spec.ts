import { expect, test } from "@playwright/test";

const media = [
  {
    name: "workspace",
    enAlt: "Pluck repository workspace showing branches, commit history, and commit details",
    zhAlt: "Pluck 仓库工作区，展示分支、提交历史和提交详情",
    loading: "eager",
  },
  {
    name: "diff",
    enAlt: "Pluck side-by-side diff showing a repository switcher component added in a commit",
    zhAlt: "Pluck 并排 Diff 视图，展示提交中新增的仓库切换组件",
    loading: "lazy",
  },
  {
    name: "rebase",
    enAlt: "Pluck interactive rebase dialog with pick and fixup actions for three commits",
    zhAlt: "Pluck 交互式 Rebase 对话框，展示三个提交的 pick 与 fixup 操作",
    loading: "lazy",
  },
] as const;

for (const locale of [
  { path: "./", altKey: "enAlt" },
  { path: "zh-CN/", altKey: "zhAlt" },
] as const) {
  test(`${locale.path} publishes the exact responsive media contract`, async ({ page }) => {
    await page.goto(locale.path);
    await expect(page.locator("picture[data-product-media]")).toHaveCount(3);

    for (const item of media) {
      const picture = page.locator(`picture[data-product-media="${item.name}"]`);
      await expect(picture).toHaveCount(1);
      await expect(picture.locator('source[type="image/avif"]')).toHaveAttribute(
        "srcset",
        `/pluck/images/${item.name}-1280.avif 1280w, /pluck/images/${item.name}-1920.avif 1920w`,
      );
      await expect(picture.locator('source[type="image/webp"]')).toHaveAttribute(
        "srcset",
        `/pluck/images/${item.name}-1280.webp 1280w, /pluck/images/${item.name}-1920.webp 1920w`,
      );

      const image = picture.locator("img");
      await expect(image).toHaveAttribute("src", `/pluck/images/${item.name}-1920.png`);
      await expect(image).toHaveAttribute("width", "1920");
      await expect(image).toHaveAttribute("height", "1133");
      await expect(image).toHaveAttribute("alt", item[locale.altKey]);
      await expect(image).toHaveAttribute("loading", item.loading);

      if (item.name === "workspace") {
        await expect(image).toHaveAttribute("fetchpriority", "high");
      } else {
        await expect(image).toHaveAttribute("decoding", "async");
      }

      await image.scrollIntoViewIfNeeded();
      await expect.poll(() => image.evaluate((node) => {
        const element = node as HTMLImageElement;
        return element.complete && element.naturalWidth > 0;
      })).toBe(true);
    }
  });
}

test("all responsive variants and the social image are published", async ({ request }) => {
  for (const item of media) {
    for (const width of [1280, 1920]) {
      for (const [format, contentType] of [
        ["avif", "image/avif"],
        ["webp", "image/webp"],
        ["png", "image/png"],
      ] as const) {
        const response = await request.get(
          `http://127.0.0.1:4173/pluck/images/${item.name}-${width}.${format}`,
        );
        expect(response.status(), `${item.name}-${width}.${format}`).toBe(200);
        expect(response.headers()["content-type"], `${item.name}-${width}.${format}`).toContain(contentType);
      }
    }
  }

  const social = await request.get("http://127.0.0.1:4173/pluck/og/pluck-cover.png");
  expect(social.status()).toBe(200);
  expect(social.headers()["content-type"]).toContain("image/png");
});
