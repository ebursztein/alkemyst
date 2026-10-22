// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { createMemoryHistory } from "@tanstack/react-router";
import { afterAll, afterEach, beforeAll, expect, it, vi } from "vitest";
import { startDesktopFixture } from "../../../../scripts/desktop-fixture.mjs";
import { createServices, type Services } from "./services.js";
import { DesktopApp } from "./desktop-app.js";
let fixture: Awaited<ReturnType<typeof startDesktopFixture>>;
let services: Services;
beforeAll(async () => {
  vi.spyOn(window, "scrollTo").mockImplementation(() => {});
  fixture = await startDesktopFixture();
}, 240_000);
afterAll(async () => {
  await fixture?.stop();
}, 15_000);
afterEach(() => {
  cleanup();
  services?.queryClient.clear();
});
function open(path: string) {
  services = createServices({ base_url: fixture.baseUrl, token: fixture.token });
  render(
    <DesktopApp services={services} history={createMemoryHistory({ initialEntries: [path] })} />,
  );
}
it("browses real local decks, their curve and model boundaries", async () => {
  open("/decks");
  await screen.findByRole("heading", { name: "Decks" });
  fireEvent.click(await screen.findByRole("link", { name: /Boros Forge/ }));
  await screen.findByRole("heading", { name: "Boros Forge" });
  expect(await screen.findByRole("figure", { name: "Printed mana curve" })).toBeTruthy();
  expect(screen.getByText("23 lands")).toBeTruthy();
  fireEvent.click(screen.getByRole("link", { name: "Analyze deck" }));
  await screen.findByRole("heading", { name: "Boros Forge · Lab" });
  expect(await screen.findByText(/No opponent interaction/)).toBeTruthy();
});
it("runs a persisted simulation through the generated SDK and displays actual denominators", async () => {
  open(`/lab/${fixture.versionId}`);
  const runs = await screen.findByLabelText("Trials");
  fireEvent.change(runs, { target: { value: "100" } });
  fireEvent.click(screen.getByRole("button", { name: "Run simulation" }));
  await screen.findByText("Completed · 100 / 100 trials", {}, { timeout: 15_000 });
  const table = screen.getByRole("table", { name: "Simulation probabilities by turn" });
  expect(within(table).getAllByRole("row").length).toBe(9);
  expect(screen.getByText(/Sampling intervals/)).toBeTruthy();
}, 20_000);

it("cancels a long native job and keeps the completed-trial denominator", async () => {
  open(`/lab/${fixture.versionId}`);
  const runs = await screen.findByLabelText("Trials");
  fireEvent.change(runs, { target: { value: "1000000" } });
  fireEvent.click(screen.getByRole("button", { name: "Run simulation" }));
  fireEvent.click(await screen.findByRole("button", { name: "Cancel simulation" }));
  await screen.findByText(/^Cancelled · [\d,]+ \/ 1,000,000 trials$/, {}, { timeout: 15_000 });
  const jobs = await services.client.GET("/v1/simulations", {
    params: { query: { version_id: fixture.versionId } },
  });
  const completed = jobs.data?.items.find((j) => j.status === "completed");
  if (!completed) throw new Error("completed benchmark missing");
  fireEvent.change(screen.getByLabelText("Saved experiments (latest 20)"), {
    target: { value: completed.id },
  });
  await screen.findByText("Completed · 100 / 100 trials");
}, 20_000);
