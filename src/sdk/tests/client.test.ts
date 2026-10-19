import { describe, expect, it, vi } from "vitest";
import { createAlkemystClient } from "../src/index.js";

describe("generated API client", () => {
  it("sends authentication and decodes the documented response", async () => {
    const fetcher = vi
      .fn<typeof fetch>()
      .mockResolvedValue(Response.json({ name: "Alkemyst", version: "0.1.0" }));
    const client = createAlkemystClient("http://127.0.0.1:47831", "test-only", fetcher);
    const result = await client.GET("/v1/health");
    const request = fetcher.mock.calls[0]?.[0] as Request;
    expect(request.url).toBe("http://127.0.0.1:47831/v1/health");
    expect(request.headers.get("Authorization")).toBe("Bearer test-only");
    expect(result.data).toEqual({ name: "Alkemyst", version: "0.1.0" });
  });

  it("uses the platform fetch by default", async () => {
    const fetcher = vi
      .fn<typeof fetch>()
      .mockResolvedValue(Response.json({ name: "Alkemyst", version: "0.1.0" }));
    vi.stubGlobal("fetch", fetcher);
    try {
      await createAlkemystClient("http://127.0.0.1:47831", "test-only").GET("/v1/health");
      expect(fetcher).toHaveBeenCalledOnce();
    } finally {
      vi.unstubAllGlobals();
    }
  });
});
