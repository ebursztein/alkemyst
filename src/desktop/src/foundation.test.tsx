// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { Foundation } from "./foundation.js";
const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("./app/desktop-app.js", () => ({ DesktopApp: () => <p>Connected workspace</p> }));
afterEach(cleanup);
it("explains a failed native connection and retries without displaying diagnostics", async () => {
  invoke.mockRejectedValueOnce(new Error("private diagnostic"));
  render(<Foundation />);
  expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Alkemyst");
  await screen.findByText("Open Alkemyst desktop to connect to your local library.");
  invoke.mockResolvedValueOnce({ base_url: "http://127.0.0.1:1234", token: "a".repeat(64) });
  fireEvent.click(screen.getByRole("button", { name: "Try again" }));
  await screen.findByText("Connected workspace");
  expect(screen.queryByText("private diagnostic")).toBeNull();
});
it("does not connect a workspace after it unmounts", async () => {
  let complete!: (value: unknown) => void;
  invoke.mockReturnValueOnce(
    new Promise((resolve) => {
      complete = resolve;
    }),
  );
  const view = render(<Foundation />);
  view.unmount();
  await act(async () => {
    complete({ base_url: "http://127.0.0.1:1234", token: "a".repeat(64) });
  });
  expect(screen.queryByText("Connected workspace")).toBeNull();
});
