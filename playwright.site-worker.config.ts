import { defineConfig } from "@playwright/test";
import site from "./playwright.site.config";

export default defineConfig(site, {
  testIgnore: [],
  use: { baseURL: "http://127.0.0.1:4322" },
  webServer: {
    command:
      "pnpm site:build && pnpm exec wrangler dev --config src/site/wrangler.jsonc --ip 127.0.0.1 --port 4322",
    url: "http://127.0.0.1:4322",
    timeout: 120_000,
    reuseExistingServer: false,
  },
});
