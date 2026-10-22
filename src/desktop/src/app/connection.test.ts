import { expect, it, vi } from "vitest";
import { bootstrapConnection, validateConnection } from "./connection.js";
const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
it("accepts only credentialed loopback bootstrap results", () => {
  const token = "a".repeat(64);
  expect(validateConnection({ base_url: "http://127.0.0.1:1234", token })).toEqual({
    base_url: "http://127.0.0.1:1234",
    token,
  });
  for (const result of [
    null,
    {},
    { base_url: "https://example.com", token },
    { base_url: "http://127.0.0.1:1234/path", token },
    { base_url: "http://user:pass@127.0.0.1:1234", token },
    { base_url: "http://127.0.0.1:1234", token: "short" },
  ]) {
    expect(() => validateConnection(result)).toThrow("Invalid local connection");
  }
});
it("gets credentials only from the native bootstrap command and hides native errors", async () => {
  invoke.mockResolvedValueOnce({ base_url: "http://127.0.0.1:1234", token: "a".repeat(64) });
  await expect(bootstrapConnection()).resolves.toHaveProperty("base_url");
  expect(invoke).toHaveBeenCalledWith("connection_bootstrap");
  invoke.mockRejectedValueOnce(new Error("private diagnostic"));
  await expect(bootstrapConnection()).rejects.toThrow(
    "Open Alkemyst desktop to connect to your local library.",
  );
});
