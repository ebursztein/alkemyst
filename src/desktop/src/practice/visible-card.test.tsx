// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { components } from "@alkemyst/sdk";
import { VisibleCard } from "./visible-card.js";
afterEach(cleanup);
const card: components["schemas"]["PracticeCard"] = {
  id: 0,
  reference: "oracle:modal",
  name: "Ember // Meadow",
  mana_cost: null,
  type_line: "Sorcery // Land",
  oracle_text: null,
  image_uri: null,
  faces: [
    {
      name: "Ember",
      mana_cost: "{R}",
      type_line: "Sorcery",
      oracle_text: "Deal damage.",
      image_uri: "https://cards.scryfall.io/normal/front.jpg",
    },
    {
      name: "Meadow",
      mana_cost: null,
      type_line: "Land",
      oracle_text: "Add mana.",
      image_uri: null,
    },
  ],
};
it("selects spell and land faces and exposes explicit manual movement", () => {
  const action = vi.fn();
  render(<VisibleCard card={card} zone="hand" busy={false} action={action} />);
  expect(screen.getByRole("img").getAttribute("src")).toContain("front.jpg");
  fireEvent.click(screen.getByRole("button", { name: "Resolve to graveyard" }));
  expect(action).toHaveBeenLastCalledWith({
    kind: "play",
    card: 0,
    face: 0,
    destination: "graveyard",
  });
  fireEvent.change(screen.getByLabelText("Face for Ember // Meadow, copy 1"), {
    target: { value: "1" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Play land" }));
  expect(action).toHaveBeenLastCalledWith({ kind: "play_land", card: 0, face: 1 });
  fireEvent.change(screen.getByLabelText("Move Ember // Meadow, copy 1, to"), {
    target: { value: "exile" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Move" }));
  expect(action).toHaveBeenLastCalledWith({ kind: "move", card: 0, destination: "exile" });
});
it("toggles tap controls and supports unresolved manual cards without invented rules", () => {
  const action = vi.fn();
  const unknown = { ...card, name: null, faces: [], type_line: null, mana_cost: null };
  const { rerender } = render(
    <VisibleCard card={unknown} zone="hand" busy={false} action={action} />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Play to battlefield" }));
  expect(action).toHaveBeenLastCalledWith({
    kind: "play",
    card: 0,
    face: 0,
    destination: "battlefield",
  });
  rerender(<VisibleCard card={card} zone="battlefield" busy={false} action={action} />);
  fireEvent.click(screen.getByRole("button", { name: "Tap Ember // Meadow" }));
  expect(action).toHaveBeenLastCalledWith({ kind: "tap", card: 0, tapped: true });
  rerender(<VisibleCard card={card} zone="battlefield" busy={false} tapped action={action} />);
  fireEvent.click(screen.getByRole("button", { name: "Untap Ember // Meadow" }));
  expect(action).toHaveBeenLastCalledWith({ kind: "tap", card: 0, tapped: false });
});
