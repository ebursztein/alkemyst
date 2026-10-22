// @vitest-environment jsdom
import { fireEvent, screen, within } from "@testing-library/react";
import { afterAll, beforeAll, expect, it, vi } from "vitest";
import { startDesktopFixture } from "../../../../scripts/desktop-fixture.mjs";
import { renderWorkspace } from "../../tests/support/workspace.js";
let fixture: Awaited<ReturnType<typeof startDesktopFixture>>;
beforeAll(async () => {
  fixture = await startDesktopFixture({ history: true });
}, 240_000);
afterAll(async () => {
  await fixture?.stop();
}, 15_000);
it("shows observed game history, gaps and turn/event timelines without inventing outcomes", async () => {
  renderWorkspace(fixture, "/games");
  const games = await screen.findByRole("table", { name: "Captured games" });
  const first = within(games).getAllByRole("row")[1]!;
  fireEvent.click(within(first).getByRole("link"));
  await screen.findByRole("heading", { name: "Event timeline" });
  await screen.findByRole("table", { name: "Observed turns" });
  expect(await screen.findByRole("list")).toBeTruthy();
  fireEvent.click(screen.getByRole("link", { name: "← Game history" }));
  await screen.findByRole("table", { name: "Captured games" });
  fireEvent.change(screen.getByLabelText("Match mode"), { target: { value: "bo3" } });
  await screen.findByRole("heading", { name: "No captured games yet" });
});
it("keeps MCP credentials hidden until asked and recovers from clipboard failure", async () => {
  renderWorkspace(fixture, "/settings");
  await screen.findByText(/24 card identities/);
  expect(screen.queryByText(new RegExp(fixture.token))).toBeNull();
  const clipboard = vi.fn().mockRejectedValueOnce(new Error("denied")).mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: clipboard },
  });
  fireEvent.click(screen.getByRole("button", { name: "Reveal credential" }));
  fireEvent.click(screen.getByRole("button", { name: "Copy connection config" }));
  await screen.findByText("Clipboard unavailable. Select and copy the displayed configuration.");
  fireEvent.click(screen.getByRole("button", { name: "Copy connection config" }));
  await screen.findByText("Connection config copied.");
  expect(clipboard).toHaveBeenLastCalledWith(expect.stringContaining(fixture.token));
  fireEvent.click(screen.getByRole("button", { name: "Hide credential" }));
  expect(screen.queryByText(new RegExp(fixture.token))).toBeNull();
});
