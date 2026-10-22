import { afterAll, beforeAll, expect, it } from "vitest";
import { startLocalService, type LocalService } from "../../../scripts/local-service.mjs";
import { createAlkemystClient } from "../src/index.js";
let service: LocalService;
beforeAll(async () => {
  service = await startLocalService();
}, 240_000);
afterAll(async () => {
  await service?.stop();
}, 15_000);
it("generated profile operation preserves missing data and agrees with MCP", async () => {
  const client = createAlkemystClient(service.baseUrl, service.token);
  const created = await client.POST("/v1/decks", {
    body: {
      idempotency_key: "sdk-profile",
      input: {
        name: "Local profile test",
        format: "standard",
        cards: [{ reference: "oracle:unresolved", quantity: 4, zone: "main" }],
      },
    },
  });
  const id = created.data?.id;
  if (!id) throw new Error("fixture deck was not created");
  const profile = await client.GET("/v1/deck-versions/{id}/analysis", {
    params: { path: { id } },
  });
  expect(profile.data?.counts.main).toBe(4);
  expect(profile.data?.counts.unresolved_main).toBe(4);
  expect(profile.data?.can_simulate).toBe(false);
  expect(profile.data?.cards[0]?.issues[0]?.code).toBe("missing_card");
  const rpc = await fetch(`${service.baseUrl}/mcp`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${service.token}`,
      "Content-Type": "application/json",
      Accept: "application/json, text/event-stream",
      "MCP-Protocol-Version": "2025-03-26",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "tools/call",
      params: { name: "analysisProfileGet", arguments: { id } },
    }),
  });
  expect(await rpc.json()).toMatchObject({ result: { structuredContent: { data: profile.data } } });
  const missing = await client.GET("/v1/deck-versions/{id}/analysis", {
    params: { path: { id: "absent" } },
  });
  expect(missing.error?.code).toBe("not_found");
});
