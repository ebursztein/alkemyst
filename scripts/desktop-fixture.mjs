import { readFile } from "node:fs/promises";
import { startLocalService } from "./local-service.mjs";
import { createAlkemystClient } from "../src/sdk/src/index.ts";
/** Isolated native backend with public card metadata; never opens the user's database. */
export async function startDesktopFixture({ history = false } = {}) {
  const service = await startLocalService({ analysis: true, history });
  try {
    const fixture = JSON.parse(
      await readFile("src/core/tests/fixtures/boros-equipment.json", "utf8"),
    );
    const client = createAlkemystClient(service.baseUrl, service.token);
    const result = await client.POST("/v1/decks", {
      body: {
        idempotency_key: "desktop-fixture",
        input: {
          name: "Boros Forge",
          format: "historic",
          cards: fixture.cards.map(({ quantity, card }) => ({
            quantity,
            reference: `oracle:${card.oracle_id}`,
            zone: "main",
          })),
        },
      },
    });
    if (!result.data) throw new Error("Desktop fixture creation failed");
    return { ...service, versionId: result.data.id, deckId: result.data.deck_id };
  } catch (error) {
    await service.stop();
    throw error;
  }
}
