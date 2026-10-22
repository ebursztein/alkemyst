import { afterAll, beforeAll, expect, it } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import { startLocalService, type LocalService } from "../../../scripts/local-service.mjs";
import { createAlkemystClient, createAlkemystQueryClient } from "../src/index.js";

let service: LocalService;
let client: ReturnType<typeof createAlkemystClient>;

beforeAll(async () => {
  service = await startLocalService();
  client = createAlkemystClient(service.baseUrl, service.token);
}, 240_000);
afterAll(async () => {
  await service?.stop();
}, 15_000);

it("uses the generated SDK against actual Rust storage and HTTP", async () => {
  const search = await client.POST("/v1/cards/search", { body: { text: "forge" } });
  expect(search.data?.items[0]?.name).toBe("Forgeblade");
  const command = {
    idempotency_key: "sdk-create",
    input: { name: "Boros", format: "standard", cards: [] },
  };
  const first = await client.POST("/v1/decks", { body: command });
  const retry = await client.POST("/v1/decks", { body: command });
  expect(first.data?.id).toBeTruthy();
  expect(retry.data).toEqual(first.data);
  const cache = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const query = createAlkemystQueryClient(service.baseUrl, service.token);
  expect((await cache.fetchQuery(query.queryOptions("get", "/v1/decks"))).items[0]?.name).toBe(
    "Boros",
  );
  cache.clear();
});

it("decodes typed authentication and missing-object errors", async () => {
  const unauthorized = await createAlkemystClient(service.baseUrl, "wrong").GET("/v1/decks");
  expect(unauthorized.response.status).toBe(401);
  expect(unauthorized.error?.code).toBe("unauthorized");
  const missing = await client.GET("/v1/deck-versions/{id}", {
    params: { path: { id: "absent" } },
  });
  expect(missing.error?.code).toBe("not_found");
});
