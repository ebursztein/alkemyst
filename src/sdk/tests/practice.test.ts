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
it("persists manual London practice with generated actions and conflict errors", async () => {
  const deck = await client.POST("/v1/decks", {
    body: {
      idempotency_key: "sdk-practice-deck",
      input: {
        name: "Practice fixture",
        format: "test",
        cards: [{ reference: "arena:42", quantity: 60, zone: "main" }],
      },
    },
  });
  if (!deck.data) throw new Error("deck missing");
  const created = await client.POST("/v1/practice", {
    body: {
      idempotency_key: "sdk-practice",
      input: { version_id: deck.data.id, on_the_play: true, seed: "ffffffffffffffff" },
    },
  });
  if (!created.data) throw new Error("practice missing");
  const id = created.data.id;
  expect(created.data.hand).toHaveLength(7);
  expect(new Set(created.data.hand.map((c) => c.id)).size).toBe(7);
  expect(created.data.seed).toBe("ffffffffffffffff");
  const mulligan = await client.POST("/v1/practice/{id}/actions", {
    params: { path: { id } },
    body: {
      idempotency_key: "sdk-mulligan",
      expected_revision: 0,
      action: { kind: "mulligan" },
    },
  });
  if (!mulligan.data?.hand[0]) throw new Error("mulligan missing");
  const keep = {
    idempotency_key: "sdk-keep",
    expected_revision: 1,
    action: { kind: "keep" as const, bottom: [mulligan.data.hand[0].id] },
  };
  const kept = await client.POST("/v1/practice/{id}/actions", {
    params: { path: { id } },
    body: keep,
  });
  expect(kept.data?.hand).toHaveLength(6);
  expect(kept.data?.library_count).toBe(54);
  expect(kept.data?.phase).toBe("main");
  expect(kept.data?.rules_mode).toBe("manual_sandbox");
  expect(kept.data?.warnings.length).toBeGreaterThan(0);
  const repeated = await client.POST("/v1/practice/{id}/actions", {
    params: { path: { id } },
    body: keep,
  });
  expect(repeated.data).toEqual(kept.data);
  const stale = await client.POST("/v1/practice/{id}/actions", {
    params: { path: { id } },
    body: {
      idempotency_key: "sdk-stale",
      expected_revision: 0,
      action: { kind: "draw" },
    },
  });
  expect(stale.response.status).toBe(409);
  expect(stale.error?.code).toBe("conflict");
  const next = await client.POST("/v1/practice/{id}/actions", {
    params: { path: { id } },
    body: {
      idempotency_key: "sdk-next",
      expected_revision: 2,
      action: { kind: "next_turn" },
    },
  });
  expect(next.data?.turn).toBe(2);
  expect(next.data?.hand).toHaveLength(7);
  const current = await client.GET("/v1/practice/{id}", { params: { path: { id } } });
  expect(current.data).toEqual(next.data);
  const inputs = await client.GET("/v1/practice/{id}/inputs", { params: { path: { id } } });
  expect(inputs.data?.counts.main).toBe(60);
  const page = await client.GET("/v1/practice", { params: { query: { limit: 2 } } });
  expect(page.data?.items[0]?.revision).toBe(3);
});
