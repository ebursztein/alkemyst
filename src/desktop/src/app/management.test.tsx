// @vitest-environment jsdom
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterAll, beforeAll, expect, it, vi } from "vitest";
import { startDesktopFixture } from "../../../../scripts/desktop-fixture.mjs";
import { renderWorkspace } from "../../tests/support/workspace.js";
let fixture: Awaited<ReturnType<typeof startDesktopFixture>>;
beforeAll(async () => {
  fixture = await startDesktopFixture();
}, 240_000);
afterAll(async () => {
  await fixture?.stop();
}, 15_000);
it("saves notes, exports Arena text, edits a new version and compares history", async () => {
  renderWorkspace(fixture, `/decks/${fixture.versionId}`);
  await screen.findByRole("heading", { name: "Boros Forge" });
  fireEvent.change(screen.getByLabelText("New note"), {
    target: { value: "Try two extra tutors against the same seed." },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save note" }));
  await screen.findByText("Try two extra tutors against the same seed.");
  fireEvent.click(screen.getByRole("button", { name: "Export for Arena" }));
  const exported = (await screen.findByLabelText("Arena export text")) as HTMLTextAreaElement;
  expect(exported.value).toContain("Deck");
  const copy = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: copy } });
  fireEvent.click(screen.getByRole("button", { name: "Copy deck text" }));
  await screen.findByText("Deck text copied.");
  expect(copy).toHaveBeenCalledWith(exported.value);
  fireEvent.click(screen.getByRole("button", { name: "Close export" }));
  fireEvent.click(screen.getByRole("button", { name: "Edit deck" }));
  await screen.findByRole("heading", { name: "Edit a new version" });
  fireEvent.change(screen.getByLabelText("Version name"), {
    target: { value: "Boros Experiment" },
  });
  fireEvent.change(await screen.findByLabelText(/^Quantity for Kellan, the Fae-Blooded/), {
    target: { value: "3" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save new version" }));
  await screen.findByRole("heading", { name: "Boros Experiment" });
  fireEvent.click(await screen.findByRole("button", { name: "Compare version 1" }));
  const changes = await screen.findByRole("table", { name: "Card quantity changes" });
  expect(within(changes).getByText("+1")).toBeTruthy();
  expect(screen.getByRole("table", { name: "Mana curve changes" })).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Compare version 2" }));
  await screen.findByRole("heading", { name: "Same card quantities" });
});
it("opens, cancels and submits an Arena import without losing a failed form", async () => {
  const { services } = renderWorkspace(fixture, "/decks");
  fireEvent.click(await screen.findByRole("button", { name: /Import deck/ }));
  fireEvent.change(screen.getByLabelText("Deck name"), { target: { value: "Imported brew" } });
  fireEvent.change(screen.getByLabelText("Arena deck text"), {
    target: { value: "Deck\n1 Missing Card" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Import" }));
  await screen.findByRole("alert");
  expect((screen.getByLabelText("Deck name") as HTMLInputElement).value).toBe("Imported brew");
  const text = await services.client.GET("/v1/deck-versions/{id}/arena", {
    params: { path: { id: fixture.versionId } },
  });
  fireEvent.change(screen.getByLabelText("Arena deck text"), {
    target: { value: text.data?.text },
  });
  fireEvent.click(screen.getByRole("button", { name: "Import" }));
  await screen.findByRole("heading", { name: "Imported brew" });
  fireEvent.click(screen.getByRole("button", { name: "Duplicate deck" }));
  await screen.findByRole("heading", { name: "Imported brew (copy)" });
  await waitFor(() => expect(screen.getByText("Version 1 · Imported brew (copy)")).toBeTruthy());
});
