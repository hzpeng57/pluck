import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  outputDir: "./test-results",
  fullyParallel: true,
  use: {
    baseURL: "http://127.0.0.1:4173/pluck/",
    trace: "on-first-retry",
  },
  webServer: {
    command: "pnpm website:preview --host 127.0.0.1 --port 4173",
    url: "http://127.0.0.1:4173/pluck/",
    reuseExistingServer: true,
  },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    { name: "mobile", use: { ...devices["iPhone 13"], browserName: "chromium" } },
  ],
});
