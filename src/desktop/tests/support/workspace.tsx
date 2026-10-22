import { cleanup, render } from "@testing-library/react";
import { createMemoryHistory } from "@tanstack/react-router";
import { afterEach, beforeAll, vi } from "vitest";
import { DesktopApp } from "../../src/app/desktop-app.js";
import { createServices, type Services } from "../../src/app/services.js";
import type { LocalService } from "../../../../scripts/local-service.mjs";
// This fixture emulates only native integration; application data uses real HTTP.
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) => {
    if (command === "native_restore_status") return { state: "none" };
    if (command === "native_settings")
      return {
        launch_at_login: false,
        tracking_paused: false,
        tracking: { state: "waiting", last_error: null },
      };
    throw new Error(`Unexpected native command: ${command}`);
  }),
}));
let clients: Services[] = [];
beforeAll(() => {
  vi.spyOn(window, "scrollTo").mockImplementation(() => {});
});
afterEach(() => {
  cleanup();
  for (const client of clients) client.queryClient.clear();
  clients = [];
});
export function renderWorkspace(
  service: LocalService,
  path: string,
  fetcher: typeof fetch = fetch,
) {
  const services = createServices({ base_url: service.baseUrl, token: service.token }, fetcher);
  clients.push(services);
  return {
    ...render(
      <DesktopApp services={services} history={createMemoryHistory({ initialEntries: [path] })} />,
    ),
    services,
  };
}
