import { expect, it, vi } from "vitest";
import { watchSettingsNavigation } from "./native-navigation.js";
const { listen } = vi.hoisted(() => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
it("opens Settings on native menu events and removes the listener on unmount", async () => {
  const stop = vi.fn();
  const navigate = vi.fn();
  let callback = () => {};
  listen.mockImplementationOnce(async (_event: string, fn: () => void) => {
    callback = fn;
    return stop;
  });
  const close = watchSettingsNavigation(navigate);
  await Promise.resolve();
  expect(listen).toHaveBeenLastCalledWith("navigate-settings", expect.any(Function));
  callback();
  expect(navigate).toHaveBeenCalledOnce();
  close();
  expect(stop).toHaveBeenCalledOnce();
});
it("removes a listener that resolves after unmount without navigating a closed page", async () => {
  const stop = vi.fn();
  const navigate = vi.fn();
  let callback = () => {};
  let ready!: (stop: () => void) => void;
  listen.mockImplementationOnce((_event: string, fn: () => void) => {
    callback = fn;
    return new Promise((resolve) => {
      ready = resolve;
    });
  });
  const close = watchSettingsNavigation(navigate);
  close();
  ready(stop);
  await Promise.resolve();
  expect(stop).toHaveBeenCalledOnce();
  callback();
  expect(navigate).not.toHaveBeenCalled();
});
it("tolerates a browser preview without native events", async () => {
  listen.mockRejectedValueOnce(new Error("native bridge unavailable"));
  const close = watchSettingsNavigation(vi.fn());
  await Promise.resolve();
  await Promise.resolve();
  close();
});
