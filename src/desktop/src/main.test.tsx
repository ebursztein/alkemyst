// @vitest-environment jsdom
import { expect, it, vi } from "vitest";

it("mounts only when the root exists", async () => {
  const render = vi.fn();
  const createRoot = vi.fn(() => ({ render }));
  vi.doMock("react-dom/client", () => ({ createRoot }));
  document.body.innerHTML = '<div id="root"></div>';
  await import("./main.js");
  expect(createRoot).toHaveBeenCalledOnce();
  expect(render).toHaveBeenCalledOnce();
  vi.resetModules();
  document.body.innerHTML = "";
  await import("./main.js");
  expect(createRoot).toHaveBeenCalledOnce();
  vi.doUnmock("react-dom/client");
});
