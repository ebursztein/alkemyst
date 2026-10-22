// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { NativeControls } from "../ui/native-controls.js";
const { call } = vi.hoisted(() => ({
  call: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: call }));
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});
async function mount() {
  const { render } = await import("@testing-library/react");
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  render(
    <QueryClientProvider client={client}>
      <NativeControls />
    </QueryClientProvider>,
  );
  return client;
}
it("requires explicit opt-in to launch at login and can pause tracking separately", async () => {
  let startup = false;
  let paused = false;
  call.mockImplementation(async (command, args) => {
    if (command === "native_settings") return { launch_at_login: startup, tracking_paused: paused };
    if (command === "set_launch_at_login") startup = (args as { enabled: boolean }).enabled;
    else if (command === "set_tracking_paused") paused = (args as { paused: boolean }).paused;
    else throw new Error("Unexpected native command");
    return undefined;
  });
  const client = await mount();
  const login = await screen.findByRole("checkbox", { name: "Launch Alkemyst at login" });
  expect((login as HTMLInputElement).checked).toBe(false);
  expect(call).toHaveBeenCalledTimes(1);
  fireEvent.click(login);
  await waitFor(() => expect((login as HTMLInputElement).checked).toBe(true));
  expect(call).toHaveBeenCalledWith("set_launch_at_login", { enabled: true });
  fireEvent.click(screen.getByRole("checkbox", { name: "Pause Arena tracking" }));
  await waitFor(() => expect(paused).toBe(true));
  await waitFor(() =>
    expect(
      (screen.getByRole("checkbox", { name: "Pause Arena tracking" }) as HTMLInputElement).checked,
    ).toBe(true),
  );
  fireEvent.click(login);
  await waitFor(() => expect(startup).toBe(false));
  client.clear();
});
it("shows read failures with retry and rejects invalid native settings", async () => {
  call
    .mockResolvedValueOnce({ launch_at_login: "yes", tracking_paused: false })
    .mockResolvedValue({ launch_at_login: false, tracking_paused: false });
  const client = await mount();
  await screen.findByRole("alert");
  expect(screen.queryByRole("checkbox")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Try again" }));
  await screen.findByRole("checkbox", { name: "Launch Alkemyst at login" });
  call.mockRejectedValueOnce(new Error("private path must never appear"));
  fireEvent.click(screen.getByRole("checkbox", { name: "Launch Alkemyst at login" }));
  await screen.findByText("Could not update desktop settings. Try again.");
  expect(screen.queryByText(/private path/)).toBeNull();
  client.clear();
});
