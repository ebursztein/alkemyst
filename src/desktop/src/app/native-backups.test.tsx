// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { BackupControls } from "../ui/backup-controls.js";

const { call } = vi.hoisted(() => ({ call: vi.fn<(command: string) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: call }));
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

function mount() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <BackupControls />
    </QueryClientProvider>,
  );
  return client;
}

it("keeps file-picker cancellation inert and saves backups only through native integration", async () => {
  let writes = 0;
  call.mockImplementation(async (command) => {
    if (command === "native_restore_status") return { state: "none" };
    if (command === "create_backup") return ++writes === 1 ? null : "/chosen/backup.sqlite3";
    if (command === "select_restore") return null;
    throw new Error(command);
  });
  const client = mount();
  const save = await screen.findByRole("button", { name: "Save backup…" });
  fireEvent.click(save);
  await waitFor(() => expect(writes).toBe(1));
  expect(screen.queryByText(/Backup saved/)).toBeNull();
  fireEvent.click(save);
  await screen.findByText("Backup saved:");
  expect(screen.getByText("/chosen/backup.sqlite3")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Restore backup…" }));
  await waitFor(() => expect(call).toHaveBeenCalledWith("select_restore"));
  expect(screen.queryByRole("alertdialog")).toBeNull();
  expect(call).not.toHaveBeenCalledWith("restore_and_restart");
  client.clear();
});

it("requires explicit restore confirmation, supports cancel and waits for restart", async () => {
  let finish: (() => void) | undefined;
  call.mockImplementation(async (command) => {
    if (command === "native_restore_status") return { state: "none" };
    if (command === "select_restore") return "saved.sqlite3";
    if (command === "cancel_restore") return undefined;
    if (command === "restore_and_restart")
      await new Promise<void>((resolve) => {
        finish = resolve;
      });
    return undefined;
  });
  const client = mount();
  fireEvent.click(await screen.findByRole("button", { name: "Restore backup…" }));
  await screen.findByRole("alertdialog", { name: "Restore from backup?" });
  expect(call).not.toHaveBeenCalledWith("restore_and_restart");
  expect(screen.getByText("saved.sqlite3")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Keep current library" }));
  await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());
  expect(call).toHaveBeenCalledWith("cancel_restore");
  fireEvent.click(screen.getByRole("button", { name: "Restore backup…" }));
  fireEvent.click(await screen.findByRole("button", { name: "Restart and restore" }));
  await waitFor(() => expect(finish).toBeDefined());
  expect(
    (screen.getByRole("button", { name: "Keep current library" }) as HTMLButtonElement).disabled,
  ).toBe(true);
  finish?.();
  await screen.findByText("Restarting Alkemyst to restore your library…");
  client.clear();
});

it("retains trigger focus and ignores repeated selection while the native picker is pending", async () => {
  let resolve: ((name: string) => void) | undefined;
  call.mockImplementation(async (command) => {
    if (command === "native_restore_status") return { state: "none" };
    if (command === "select_restore")
      return await new Promise<string>((done) => {
        resolve = done;
      });
    return undefined;
  });
  const client = mount();
  const choose = await screen.findByRole("button", { name: "Restore backup…" });
  choose.focus();
  fireEvent.click(choose);
  await waitFor(() => expect(resolve).toBeDefined());
  fireEvent.click(choose);
  expect(call.mock.calls.filter(([command]) => command === "select_restore")).toHaveLength(1);
  expect(document.activeElement).toBe(choose);
  resolve?.("selected.sqlite3");
  await screen.findByRole("alertdialog", { name: "Restore from backup?" });
  client.clear();
});

it("explains recovery results and redacts native failure details", async () => {
  call.mockImplementation(async (command) => {
    if (command === "native_restore_status")
      return { state: "restored", safety_backup: "/backups/safety.sqlite3" };
    throw new Error("SQL error with private source path");
  });
  const client = mount();
  await screen.findByText("Backup restored. Your previous library was kept in a safety backup.");
  expect(screen.getByText("/backups/safety.sqlite3")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Save backup…" }));
  await screen.findByRole("alert");
  expect(screen.queryByText(/private source/)).toBeNull();
  client.clear();
});

it("rejects malformed native status and offers a retry without enabling unavailable controls", async () => {
  call
    .mockResolvedValueOnce({ state: "restored", safety_backup: 42 })
    .mockResolvedValue({ state: "failed", safety_backup: null });
  const client = mount();
  await screen.findByRole("alert");
  expect(screen.queryByRole("button", { name: "Save backup…" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Try again" }));
  await screen.findByText("Restore could not be applied. Your current library was kept.");
  await screen.findByRole("button", { name: "Save backup…" });
  client.clear();
});
