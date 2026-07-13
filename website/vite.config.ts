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
