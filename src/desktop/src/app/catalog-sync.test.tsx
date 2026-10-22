// @vitest-environment jsdom
import { QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { createServices, ServiceContext, useServices } from "./services.js";
import { useCatalogStatus } from "./catalog-sync.js";
afterEach(cleanup);

function Probe() {
  useCatalogStatus();
  const { api } = useServices();
  const lookup = api.useQuery("post", "/v1/cards/resolve-arena", { body: { ids: [100] } });
  const card = lookup.data?.[0]?.card;
  const name = card && typeof card === "object" && "name" in card ? card.name : undefined;
  return <p>{typeof name === "string" ? name : "Unresolved"}</p>;
}

it("refreshes warm application reads when installed identities or the metadata snapshot change", async () => {
  let revision = 0;
  let snapshot = "first";
  let resolved = false;
  let name = "Forgeblade";
  let lookups = 0;
  const fetcher: typeof fetch = async (input, init) => {
    const request = new Request(input, init);
    if (new URL(request.url).pathname === "/v1/catalog")
      return Response.json({
        snapshot_id: snapshot,
        source: {
          url: "https://data.scryfall.io/fixture.jsonl.gz",
          updated_at: "2026-10-21T00:00:00Z",
        },
        cards: 3,
        printings: 3,
        arena_ids: 1,
        arena_aliases: 1,
        arena_alias_revision: revision,
      });
    lookups++;
    return Response.json([
      {
        arena_id: 100,
        card: resolved ? { name } : null,
        resolved_by: resolved ? "arena_asset_printing" : null,
      },
    ]);
  };
  const services = createServices(
    { base_url: "http://127.0.0.1:12345", token: "test-only-token" },
    fetcher,
  );
  services.queryClient.setQueryData(["native-card-cache-status"], { state: "ready" });
  render(
    <ServiceContext.Provider value={services}>
      <QueryClientProvider client={services.queryClient}>
        <Probe />
      </QueryClientProvider>
    </ServiceContext.Provider>,
  );
  await waitFor(() => expect(lookups).toBeGreaterThan(0));
  await screen.findByText("Unresolved");
  resolved = true;
  revision = 1;
  const queryKey = services.api.queryOptions("get", "/v1/catalog").queryKey;
  await services.queryClient.refetchQueries({ queryKey, exact: true });
  await screen.findByText("Forgeblade");
  const same = lookups;
  await services.queryClient.refetchQueries({ queryKey, exact: true });
  expect(lookups).toBe(same);
  snapshot = "second";
  name = "Find the Blade";
  await services.queryClient.refetchQueries({ queryKey, exact: true });
  await screen.findByText("Find the Blade");
  expect(services.queryClient.getQueryState(["native-card-cache-status"])?.isInvalidated).toBe(
    false,
  );
  services.queryClient.clear();
});
