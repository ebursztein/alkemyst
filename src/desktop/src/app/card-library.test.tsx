// @vitest-environment jsdom
import { QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createServices, ServiceContext } from "./services.js";
import { CardLibrary } from "../ui/card-library.js";
const { invoke } = vi.hoisted(() => ({ invoke: vi.fn<(command: string) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

it("explains the first download and recovers from offline failure without inventing metadata", async () => {
  let loaded = false;
  let fail = true;
  const status = () => ({
    snapshot_id: loaded ? "fixture" : null,
    source: null,
    cards: loaded ? 3 : 0,
    printings: loaded ? 3 : 0,
    arena_ids: 1,
    arena_aliases: 1,
    arena_alias_revision: 1,
  });
  const fetcher: typeof fetch = async (input, init) => {
    const request = new Request(input, init);
    if (request.method === "POST") {
      if (fail)
        return Response.json(
          { code: "busy", message: "Metadata download is offline." },
          { status: 503 },
        );
      loaded = true;
    }
    return Response.json(status());
  };
  invoke.mockResolvedValue({ state: "ready", last_error: null, checks: 1 });
  const services = createServices(
    { base_url: "http://127.0.0.1:12345", token: "test-only-token" },
    fetcher,
  );
  render(
    <ServiceContext.Provider value={services}>
      <QueryClientProvider client={services.queryClient}>
        <CardLibrary />
      </QueryClientProvider>
    </ServiceContext.Provider>,
  );
  const download = await screen.findByRole("button", { name: "Download card metadata" });
  await screen.findByText(/Tracking can collect games before this download/);
  fireEvent.click(download);
  await screen.findByText("Metadata download is offline.");
  expect(loaded).toBe(false);
  expect(screen.getByRole("button", { name: "Download card metadata" })).toBeTruthy();
  fail = false;
  fireEvent.click(download);
  await screen.findByRole("button", { name: "Update card metadata" });
  expect(screen.queryByText("Metadata download is offline.")).toBeNull();
  services.queryClient.clear();
});

it("keeps private native errors out of the UI and supports discovery retry and picker cancellation", async () => {
  let healthy = false;
  invoke.mockImplementation(async (command) => {
    if (command === "native_card_cache_status")
      return healthy
        ? { state: "missing", last_error: null }
        : { state: "invalid", last_error: "private/path" };
    if (command === "retry_card_cache") {
      healthy = true;
      return null;
    }
    if (command === "choose_card_cache") return false;
    throw new Error(command);
  });
  const fetcher: typeof fetch = async () =>
    Response.json({
      snapshot_id: "fixture",
      source: null,
      cards: 3,
      printings: 3,
      arena_ids: 1,
      arena_aliases: 0,
      arena_alias_revision: 0,
    });
  const services = createServices(
    { base_url: "http://127.0.0.1:12345", token: "test-only-token" },
    fetcher,
  );
  render(
    <ServiceContext.Provider value={services}>
      <QueryClientProvider client={services.queryClient}>
        <CardLibrary />
      </QueryClientProvider>
    </ServiceContext.Provider>,
  );
  await screen.findByRole("alert");
  expect(screen.queryByText(/private\/path/)).toBeNull();
  healthy = true;
  fireEvent.click(screen.getByRole("button", { name: "Try again" }));
  await screen.findByText("No installed Arena card cache found.");
  fireEvent.click(screen.getByRole("button", { name: "Retry card discovery" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("retry_card_cache"));
  fireEvent.click(screen.getByRole("button", { name: "Choose Arena card cache…" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("choose_card_cache"));
  expect(screen.queryByText(/Checking the chosen/)).toBeNull();
  services.queryClient.clear();
});
