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

it("compares immutable versions through the generated contract", async () => {
  const base = await client.POST("/v1/decks", {
    body: {
      idempotency_key: "sdk-comparison-base",
      input: {
        name: "Comparison base",
        format: "standard",
        cards: [{ reference: "arena:42", quantity: 4, zone: "main" }],
      },
    },
  });
  if (!base.data) throw new Error("base version missing");
  const candidate = await client.POST("/v1/deck-versions", {
    body: {
      idempotency_key: "sdk-comparison-candidate",
      input: {
        deck_id: base.data.deck_id,
        parent_version: base.data.id,
        name: "Comparison candidate",
        format: "standard",
        cards: [
          { reference: "scryfall:print-equipment-old", quantity: 3, zone: "main" },
          { reference: "oracle:oracle-tutor", quantity: 1, zone: "main" },
        ],
      },
    },
  });
  if (!candidate.data) throw new Error("candidate version missing");
  const result = await client.GET("/v1/deck-versions/{id}/compare/{candidate_id}", {
    params: { path: { id: base.data.id, candidate_id: candidate.data.id } },
  });
  expect(result.data?.base.main).toBe(4);
  expect(result.data?.candidate.main).toBe(4);
  expect(result.data?.cards.find((c) => c.identity === "oracle:oracle-equipment")?.delta).toBe(-1);
  expect(result.data?.cards.find((c) => c.identity === "oracle:oracle-tutor")?.delta).toBe(1);
  expect(result.data?.curve.find((c) => c.mana_value === 2)?.delta).toBe(-1);
});
