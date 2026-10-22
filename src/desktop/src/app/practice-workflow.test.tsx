// @vitest-environment jsdom
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterAll, beforeAll, expect, it } from "vitest";
import { startDesktopFixture } from "../../../../scripts/desktop-fixture.mjs";
import { renderWorkspace } from "../../tests/support/workspace.js";
let fixture: Awaited<ReturnType<typeof startDesktopFixture>>;
beforeAll(async () => {
  fixture = await startDesktopFixture();
}, 240_000);
afterAll(async () => {
  await fixture?.stop();
}, 15_000);
it("starts a session, bottoms an ordered physical card, draws and saves its finish", async () => {
  renderWorkspace(fixture, "/practice");
  await screen.findByRole("option", { name: "Boros Forge" });
  fireEvent.change(screen.getByLabelText("Deck"), { target: { value: fixture.versionId } });
  fireEvent.click(screen.getByRole("button", { name: "Draw opening hand" }));
  await screen.findByRole("heading", { name: "Practice table" });
  fireEvent.click(screen.getByRole("button", { name: "Mulligan" }));
  await screen.findByText("Choose 1 to bottom · 0 selected");
  const choices = screen.getAllByRole("checkbox");
  await waitFor(() => expect((choices[0] as HTMLInputElement).disabled).toBe(false));
  fireEvent.click(choices[0]!);
  fireEvent.click(screen.getByRole("button", { name: "Keep hand" }));
  await screen.findByRole("heading", { name: "Hand · 6" });
  const hand = screen.getByRole("heading", { name: "Hand · 6" }).closest("section")!;
  await waitFor(() =>
    expect(within(hand).getAllByRole("button", { name: "Move" })[0]?.hasAttribute("disabled")).toBe(
      false,
    ),
  );
  fireEvent.click(within(hand).getAllByRole("button", { name: "Move" })[0]!);
  await screen.findByRole("heading", { name: "Hand · 5" });
  fireEvent.click(screen.getByText("graveyard · 1"));
  const grave = screen.getByText("graveyard · 1").closest("details")!;
  await waitFor(() =>
    expect(within(grave).getByRole("button", { name: "Move" }).hasAttribute("disabled")).toBe(
      false,
    ),
  );
  fireEvent.click(within(grave).getByRole("button", { name: "Move" }));
  await screen.findByRole("heading", { name: "Hand · 6" });
  fireEvent.click(screen.getByRole("button", { name: "Next turn" }));
  await screen.findByRole("heading", { name: "Hand · 7" });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Draw a card" }).hasAttribute("disabled")).toBe(
      false,
    ),
  );
  fireEvent.click(screen.getByRole("button", { name: "Draw a card" }));
  await screen.findByRole("heading", { name: "Hand · 8" });
  fireEvent.click(screen.getByRole("button", { name: "Finish practice" }));
  await screen.findByText("Practice session finished.");
  fireEvent.click(screen.getByRole("link", { name: "← Saved practice" }));
  await screen.findByText(/finished · Turn 2/);
});
