import { afterAll, beforeAll, expect, it } from "vitest";
import { startLocalService, type LocalService } from "../../../scripts/local-service.mjs";
import { createAlkemystClient } from "../src/index.js";
let service: LocalService;
let client: ReturnType<typeof createAlkemystClient>;
beforeAll(async () => {
  service = await startLocalService();
  client = createAlkemystClient(service.baseUrl, service.token);
}, 240_000);
afterAll(async () => {
  await service?.stop();
}, 15_000);
it("lists typed version summaries with opaque cursors after version creation", async () => {
  const base = await client.POST("/v1/decks", {
    body: {
      idempotency_key: "version-list-base",
      input: {
        name: "Version list",
        format: "test",
        cards: [{ reference: "arena:42", quantity: 4, zone: "main" }],
      },
    },
  });
  if (!base.data) throw new Error("base missing");
  const next = await client.POST("/v1/deck-versions", {
    body: {
      idempotency_key: "version-list-next",
      input: {
        deck_id: base.data.deck_id,
        parent_version: base.data.id,
        name: "Version list two",
        format: "test",
        cards: [{ reference: "arena:42", quantity: 3, zone: "main" }],
      },
    },
  });
  if (!next.data) throw new Error("next missing");
  const page = await client.GET("/v1/decks/{id}/versions", {
    params: { path: { id: base.data.deck_id }, query: { limit: 1 } },
  });
  expect(page.data?.items[0]?.id).toBe(next.data.id);
  expect(page.data?.items[0]?.main_count).toBe(3);
  expect(page.data?.next_cursor).toBeTruthy();
  const older = await client.GET("/v1/decks/{id}/versions", {
    params: {
      path: { id: base.data.deck_id },
      query: { limit: 1, cursor: page.data?.next_cursor ?? undefined },
    },
  });
  expect(older.data?.items[0]?.id).toBe(base.data.id);
  expect(older.data?.next_cursor).toBeNull();
});
