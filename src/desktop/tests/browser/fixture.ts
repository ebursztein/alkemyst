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
        const calls: string[] = [];
        Reflect.set(window, "__ALKEMYST_NATIVE_CALLS__", calls);
        Object.defineProperty(window, "__TAURI_INTERNALS__", {
          value: {
            invoke: async (command: string) => {
              calls.push(command);
              // Browser tests simulate only OS dialogs/settings; all deck and history
              // operations still go through the authenticated Rust API and generated SDK.
              if (command === "native_restore_status") return { state: "none" };
              if (command === "native_settings")
                return {
                  launch_at_login: false,
                  tracking_paused: false,
                  tracking: { state: "waiting", last_error: null },
                };
              if (command === "create_backup") return "/test/alkemyst-backup.sqlite3";
              if (command === "select_restore") return "alkemyst-backup.sqlite3";
              if (command === "cancel_restore" || command === "restore_and_restart") return null;
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
