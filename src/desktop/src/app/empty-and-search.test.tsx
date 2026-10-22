// @vitest-environment jsdom
import { fireEvent, screen } from "@testing-library/react";
import { afterAll, beforeAll, expect, it } from "vitest";
import type { components } from "@alkemyst/sdk";
import { startDesktopFixture } from "../../../../scripts/desktop-fixture.mjs";
import { renderWorkspace } from "../../tests/support/workspace.js";
let fixture: Awaited<ReturnType<typeof startDesktopFixture>>;
beforeAll(async () => {
  fixture = await startDesktopFixture();
}, 240_000);
afterAll(async () => {
  await fixture?.stop();
}, 15_000);
it.each([
  ["/decks", "Your next brew starts here"],
  ["/lab", "Choose your first experiment"],
  ["/practice", "A fresh table"],
])("explains the empty %s workspace", async (path, title) => {
  const fetcher: typeof fetch = async (input, init) => {
    const request = new Request(input, init);
    if (new URL(request.url).pathname === "/v1/decks")
      return Response.json({
        items: [],
        next_cursor: null,
      } satisfies components["schemas"]["DeckPage"]);
    return fetch(request);
  };
  renderWorkspace(fixture, path!, fetcher);
  expect(await screen.findByRole("heading", { name: title })).toBeTruthy();
});
it("searches local rules metadata, adds an existing card and cancels the draft", async () => {
  renderWorkspace(fixture, `/decks/${fixture.versionId}`);
  fireEvent.click(await screen.findByRole("button", { name: "Edit deck" }));
  fireEvent.change(screen.getByLabelText("Search local cards"), { target: { value: "Cloud" } });
  fireEvent.click(screen.getByRole("button", { name: "Search cards" }));
  fireEvent.click(await screen.findByRole("button", { name: "Add Cloud, Midgar Mercenary" }));
  expect(
    (screen.getByLabelText(/^Quantity for Cloud, Midgar Mercenary/) as HTMLInputElement).value,
  ).toBe("3");
  fireEvent.change(screen.getByLabelText("Search local cards"), {
    target: { value: "NothingMatchesThisName" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Search cards" }));
  await screen.findByText("No matching local cards.");
  fireEvent.click(screen.getByRole("button", { name: "Cancel edit" }));
  expect(screen.queryByRole("heading", { name: "Edit a new version" })).toBeNull();
  fireEvent.click(screen.getByRole("link", { name: "← All decks" }));
  fireEvent.click(await screen.findByRole("button", { name: /Import deck/ }));
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(screen.queryByLabelText("Arena deck text")).toBeNull();
});
it("shows unsupported cards honestly and never enables an unsupported simulation", async () => {
  const { services } = renderWorkspace(fixture, "/lab");
  const version = await services.client.POST("/v1/decks", {
    body: {
      idempotency_key: "unsupported-ui",
      input: {
        name: "Unknown brew",
        format: "test",
        cards: [{ reference: "oracle:unknown", quantity: 60, zone: "main" }],
      },
    },
  });
  if (!version.data) throw new Error("fixture missing");
  await services.queryClient.invalidateQueries();
  fireEvent.click(await screen.findByRole("link", { name: /Unknown brew/ }));
  await screen.findByText("Unsupported inputs");
  expect(
    (screen.getByRole("button", { name: "Run simulation" }) as HTMLButtonElement).disabled,
  ).toBe(true);
});
