import { test as base, expect } from "@playwright/test";
import { startDesktopFixture } from "../../../../scripts/desktop-fixture.mjs";
export const test = base.extend<{ backend: Awaited<ReturnType<typeof startDesktopFixture>> }>({
  backend: async ({ baseURL }, use) => {
    if (baseURL !== "http://127.0.0.1:1420")
      throw new Error("Desktop tests require the local development origin");
    const fixture = await startDesktopFixture({ history: true });
    try {
      await use(fixture);
    } finally {
      await fixture.stop();
    }
  },
  page: async ({ page, backend }, use) => {
    await page.addInitScript(
      (connection) => {
        Object.defineProperty(window, "__TAURI_INTERNALS__", {
          value: {
            invoke: async (command: string) => {
              if (command !== "connection_bootstrap")
                throw new Error("Unexpected native command in browser fixture");
              return connection;
            },
          },
        });
      },
      { base_url: backend.baseUrl, token: backend.token },
    );
    await use(page);
  },
});
export { expect };
