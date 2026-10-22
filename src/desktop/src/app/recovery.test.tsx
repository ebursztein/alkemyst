// @vitest-environment jsdom
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { afterAll, beforeAll, expect, it } from "vitest";
import { startDesktopFixture } from "../../../../scripts/desktop-fixture.mjs";
import { createAlkemystClient } from "@alkemyst/sdk";
import { renderWorkspace } from "../../tests/support/workspace.js";
let fixture: Awaited<ReturnType<typeof startDesktopFixture>>;
beforeAll(async () => {
  fixture = await startDesktopFixture();
}, 240_000);
afterAll(async () => {
  await fixture?.stop();
}, 15_000);
const cases = [
  ["/decks", "/v1/decks"],
  ["/lab", "/v1/decks"],
  ["/lab/ID", "/v1/deck-versions/ID/analysis"],
  ["/decks/ID", "/v1/deck-versions/ID"],
  ["/decks/ID", "/v1/cards/resolve-references"],
  ["/decks/ID", "/v1/notes"],
  ["/games", "/v1/games"],
  ["/games", "/v1/statistics"],
  ["/practice", "/v1/practice"],
  ["/practice", "/v1/decks"],
  ["/settings", "/v1/catalog"],
  ["/settings", "/v1/capture"],
] as const;
it.each(cases)("recovers %s when %s becomes available", async (path, endpoint) => {
  let blocked = true;
  const target = endpoint.replace("ID", fixture.versionId);
  const fetcher: typeof fetch = async (input, init) => {
    const request = new Request(input, init);
    if (blocked && new URL(request.url).pathname === target)
      return Response.json(
        { code: "busy", message: "Local data is temporarily unavailable." },
        { status: 503 },
      );
    return fetch(request);
  };
  renderWorkspace(fixture, path.replace("ID", fixture.versionId), fetcher);
  await screen.findByRole("alert");
  expect(screen.queryAllByText("Unresolved card")).toHaveLength(0);
  blocked = false;
  fireEvent.click(screen.getAllByRole("button", { name: "Try again" })[0]!);
  await waitFor(() => expect(screen.queryAllByRole("alert")).toHaveLength(0));
});
it("retries a committed import after a dropped response without creating a duplicate deck", async () => {
  const client = createAlkemystClient(fixture.baseUrl, fixture.token);
  const exported = await client.GET("/v1/deck-versions/{id}/arena", {
    params: { path: { id: fixture.versionId } },
  });
  let dropped = false;
  const keys: string[] = [];
  const fetcher: typeof fetch = async (input, init) => {
    const request = new Request(input, init);
    const importing = new URL(request.url).pathname === "/v1/decks/import-arena";
    if (importing)
      keys.push(((await request.clone().json()) as { idempotency_key: string }).idempotency_key);
    const response = await fetch(request);
    if (importing && !dropped && response.ok) {
      dropped = true;
      return Response.json(
        { code: "busy", message: "Response interrupted. Retry the same import." },
        { status: 503 },
      );
    }
    return response;
  };
  renderWorkspace(fixture, "/decks", fetcher);
  fireEvent.click(await screen.findByRole("button", { name: /Import deck/ }));
  fireEvent.change(screen.getByLabelText("Deck name"), { target: { value: "Retry-safe brew" } });
  fireEvent.change(screen.getByLabelText("Arena deck text"), {
    target: { value: exported.data?.text },
  });
  fireEvent.click(screen.getByRole("button", { name: "Import" }));
  await screen.findByText("Response interrupted. Retry the same import.");
  fireEvent.click(screen.getByRole("button", { name: "Import" }));
  await screen.findByRole("heading", { name: "Retry-safe brew" });
  expect(keys).toHaveLength(2);
  expect(keys[0]).toBe(keys[1]);
  const decks = await client.GET("/v1/decks");
  expect(decks.data?.items.filter((d) => d.name === "Retry-safe brew")).toHaveLength(1);
});
