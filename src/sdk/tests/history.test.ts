import { afterAll, beforeAll, expect, it } from "vitest";
import { startLocalService, type LocalService } from "../../../scripts/local-service.mjs";
import { createAlkemystClient } from "../src/index.js";
let service: LocalService;
let client: ReturnType<typeof createAlkemystClient>;
beforeAll(async () => {
  service = await startLocalService({ history: true });
  client = createAlkemystClient(service.baseUrl, service.token);
}, 240_000);
afterAll(async () => {
  await service?.stop();
}, 15_000);
it("generated history operations preserve gaps, paginate, and agree with MCP", async () => {
  const games = await client.GET("/v1/games");
  expect(games.data?.items).toHaveLength(1);
  const game = games.data?.items[0];
  if (!game) throw new Error("sanitized fixture must have one game");
  expect(game.capture_complete).toBe(false);
  expect(game.missing_data.length).toBeGreaterThan(0);
  const detail = await client.GET("/v1/games/{id}", { params: { path: { id: game.id } } });
  expect(detail.data).toEqual(game);
  const timeline = await client.GET("/v1/games/{id}/events", {
    params: { path: { id: game.id }, query: { limit: 2 } },
  });
  expect(timeline.data?.items).toHaveLength(2);
  const cursor = timeline.data?.next_cursor;
  if (!cursor) throw new Error("fixture timeline must paginate");
  const next = await client.GET("/v1/games/{id}/events", {
    params: { path: { id: game.id }, query: { limit: 2, cursor } },
  });
  expect(next.data?.through_ordinal).toBe(timeline.data?.through_ordinal);
  expect(next.data?.items[0]?.ordinal).toBeGreaterThan(timeline.data?.items[1]?.ordinal ?? 0);
  const turns = await client.GET("/v1/games/{id}/turns", {
    params: { path: { id: game.id }, query: { limit: 2 } },
  });
  expect(turns.data?.items.length).toBeGreaterThan(0);
  const matches = await client.GET("/v1/matches");
  expect(matches.data?.items).toHaveLength(1);
  const match = await client.GET("/v1/matches/{id}", { params: { path: { id: game.match_id } } });
  expect(match.data?.game_count).toBe(1);
  const stats = await client.GET("/v1/statistics");
  expect(stats.data?.games).toBe(1);
  expect(stats.data?.known_results).toBe(1);
  expect(stats.data?.partial_games).toBe(1);
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
      params: { name: "statisticsGet", arguments: {} },
    }),
  });
  expect(await rpc.json()).toMatchObject({ result: { structuredContent: { data: stats.data } } });
  const capture = await client.GET("/v1/capture");
  expect(capture.data?.games).toBe(1);
  const missing = await client.GET("/v1/games/{id}", { params: { path: { id: "absent" } } });
  expect(missing.error?.code).toBe("not_found");
});
