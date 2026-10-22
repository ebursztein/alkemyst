import { afterAll, beforeAll, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { startLocalService, type LocalService } from "../../../scripts/local-service.mjs";
import { createAlkemystClient, type components } from "../src/index.js";
let service: LocalService;
let client: ReturnType<typeof createAlkemystClient>;
beforeAll(async () => {
  service = await startLocalService({ analysis: true });
  client = createAlkemystClient(service.baseUrl, service.token);
}, 240_000);
afterAll(async () => {
  await service?.stop();
}, 15_000);
async function terminal(id: string) {
  const deadline = Date.now() + 20_000;
  while (Date.now() < deadline) {
    const job = await client.GET("/v1/simulations/{id}", { params: { path: { id } } });
    if (!job.data) throw new Error("job missing");
    if (!["queued", "running", "cancelling"].includes(job.data.status)) return job.data;
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  throw new Error("job did not finish");
}
it("generated jobs preserve full-width seeds, frozen inputs, results and cancellation", async () => {
  const fixture = JSON.parse(
    readFileSync("src/core/tests/fixtures/boros-equipment.json", "utf8"),
  ) as {
    cards: { quantity: number; card: { oracle_id: string } }[];
  };
  const version = await client.POST("/v1/decks", {
    body: {
      idempotency_key: "sdk-simulation-deck",
      input: {
        name: "Sanitized SDK benchmark",
        format: "historic",
        cards: fixture.cards.map(({ quantity, card }) => ({
          reference: `oracle:${card.oracle_id}`,
          quantity,
          zone: "main" as const,
        })),
      },
    },
  });
  if (!version.data) throw new Error("deck was not created");
  const parameters: components["schemas"]["SimulationParameters"] = {
    version_id: version.data.id,
    runs: 100,
    turns: 5,
    on_the_play: true,
    seed: "ffffffffffffffff",
    scope: "precombat_goldfish",
    goal: { equipment: 2, attached: 1, attack_ready: 1 },
    mulligan: {
      minimum_keep: 4,
      force_mulligans: 0,
      min_lands: 2,
      max_lands: 5,
      required_references: [],
    },
  };
  const created = await client.POST("/v1/simulations", {
    body: { idempotency_key: "sdk-job", input: parameters },
  });
  if (!created.data) throw new Error(`job failed: ${JSON.stringify(created.error)}`);
  const id = created.data.id;
  const job = await terminal(id);
  expect(job.status).toBe("completed");
  expect(job.report?.seed).toBe("ffffffffffffffff");
  expect(job.report?.turns[0]?.joint.samples).toBe(100);
  const repeated = await client.POST("/v1/simulations", {
    body: { idempotency_key: "sdk-job", input: parameters },
  });
  expect(repeated.data?.id).toBe(id);
  const inputs = await client.GET("/v1/simulations/{id}/inputs", { params: { path: { id } } });
  expect(inputs.data?.snapshot.version_id).toBe(version.data.id);
  const list = await client.GET("/v1/simulations", { params: { query: { limit: 2 } } });
  expect(list.data?.items[0]?.id).toBe(id);
  const long = await client.POST("/v1/simulations", {
    body: { idempotency_key: "sdk-cancel", input: { ...parameters, runs: 1_000_000 } },
  });
  if (!long.data) throw new Error("long job missing");
  await client.POST("/v1/simulations/{id}/cancel", { params: { path: { id: long.data.id } } });
  const stopped = await terminal(long.data.id);
  expect(stopped.status).toBe("cancelled");
  if (stopped.report) expect(stopped.report.completed_runs).toBe(stopped.completed_runs);
});
it("generated exact draw and joint probability contracts preserve their scope", async () => {
  const draw = await client.POST("/v1/probabilities/draw", {
    body: { population: 60, successes: 4, draws: 7, min: 1, max: 4 },
  });
  expect(draw.data?.probability).toBeCloseTo(0.3994996257442573, 12);
  const joint = await client.POST("/v1/probabilities/joint", {
    body: {
      draws: 2,
      categories: [
        { label: "Equipment", population: 2, min: 1, max: 1 },
        { label: "Creatures", population: 2, min: 1, max: 1 },
        { label: "Other", population: 2, min: 0, max: 2 },
      ],
    },
  });
  expect(joint.data?.probability).toBeCloseTo(4 / 15, 12);
});
