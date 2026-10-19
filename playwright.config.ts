import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "src/desktop/tests/browser",
  use: { baseURL: "http://127.0.0.1:1420", browserName: "chromium" },
  webServer: {
    command: "pnpm dev",
    url: "http://127.0.0.1:1420",
    reuseExistingServer: !process.env.CI,
  },
  reporter: [["list"], ["html", { open: "never" }]],
});
