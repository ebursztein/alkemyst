// @vitest-environment jsdom
import { fireEvent, screen } from "@testing-library/react";
import { afterAll, beforeAll, expect, it, vi } from "vitest";
import { createAlkemystClient } from "@alkemyst/sdk";
import { startDesktopFixture } from "../../../../scripts/desktop-fixture.mjs";
import { renderWorkspace } from "../../tests/support/workspace.js";
let fixture: Awaited<ReturnType<typeof startDesktopFixture>>;
beforeAll(async () => {
  fixture = await startDesktopFixture();
}, 240_000);
afterAll(async () => {
  await fixture?.stop();
}, 15_000);
it("moves the same card between zones without duplicate row identities or losing quantities", async () => {
  const client = createAlkemystClient(fixture.baseUrl, fixture.token);
  const source = await client.GET("/v1/deck-versions/{id}", {
    params: { path: { id: fixture.versionId } },
  });
  const reference = source.data?.cards[0]?.reference;
  if (!reference) throw new Error("fixture card missing");
  const created = await client.POST("/v1/decks", {
    body: {
      idempotency_key: "two-zones",
      input: {
        name: "Two zones",
        format: "test",
        cards: [
          { reference, quantity: 3, zone: "main" },
          { reference, quantity: 1, zone: "sideboard" },
        ],
      },
    },
  });
  if (!created.data) throw new Error("fixture deck missing");
  renderWorkspace(fixture, `/decks/${created.data.id}`);
  fireEvent.click(await screen.findByRole("button", { name: "Edit deck" }));
  const zones = await screen.findAllByRole("combobox", { name: /^Zone for/ });
  const warning = vi.spyOn(console, "error").mockImplementation(() => {});
  try {
    fireEvent.change(zones[1]!, { target: { value: "main" } });
    expect(warning).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Version name"), { target: { value: "Merged zones" } });
    fireEvent.click(screen.getByRole("button", { name: "Save new version" }));
    await screen.findByRole("heading", { name: "Merged zones" });
    const history = await client.GET("/v1/decks/{id}/versions", {
      params: { path: { id: created.data.deck_id } },
    });
    expect(history.data?.items[0]?.main_count).toBe(4);
    expect(history.data?.items[0]?.sideboard_count).toBe(0);
  } finally {
    warning.mockRestore();
  }
});
