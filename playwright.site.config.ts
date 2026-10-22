import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "src/site/tests/browser",
  use: { baseURL: "http://127.0.0.1:4321", browserName: "chromium" },
  projects: [
    { name: "desktop", use: { viewport: { width: 1280, height: 900 }, colorScheme: "light" } },
    {
      name: "mobile-dark",
      use: { viewport: { width: 390, height: 844 }, colorScheme: "dark", reducedMotion: "reduce" },
    },
  ],
  webServer: {
    command: "pnpm --filter @alkemyst/site build && pnpm --filter @alkemyst/site preview",
    url: "http://127.0.0.1:4321",
    timeout: 120_000,
    reuseExistingServer: false,
  },
  reporter: [["list"], ["html", { outputFolder: "playwright-report/site", open: "never" }]],
});
