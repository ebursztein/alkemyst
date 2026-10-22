import { useServices } from "../app/services.js";
import { ArenaCacheControls } from "./arena-cache-controls.js";
import { Button } from "./button.js";
import { Failure, Loading } from "./states.js";

export function CardLibrary() {
  const { api, queryClient } = useServices();
  const catalog = api.useQuery("get", "/v1/catalog");
  const refresh = api.useMutation("post", "/v1/catalog/refresh", {
    onSuccess: () => {
      void queryClient.invalidateQueries();
    },
  });
  const empty = catalog.data?.cards === 0;
  return (
    <section className="panel">
      <h2>Card library</h2>
      {catalog.isPending && <Loading />}
      {catalog.isError && (
        <Failure
          error={catalog.error}
          retry={() => {
            void catalog.refetch();
          }}
        />
      )}
      {catalog.data && (
        <>
          {empty ? (
            <>
              <h3>Set up your card library</h3>
              <p>
                Tracking can collect games before this download. Card search, resolved deck names
                and analysis need the Scryfall metadata.
              </p>
              <p className="muted">
                The first download and import can take a few minutes and needs internet and free
                disk space. You can keep using Alkemyst while it runs.
              </p>
            </>
          ) : (
            <>
              <p>
                <strong>{catalog.data.cards.toLocaleString()}</strong> card identities ·{" "}
                {catalog.data.printings.toLocaleString()} printings
              </p>
              <p className="muted">
                Snapshot: {catalog.data.source?.updated_at ?? "Not recorded"}.
              </p>
            </>
          )}
          <p className="muted">
            {empty
              ? "After the first import, metadata remains usable offline."
              : "Your imported metadata remains usable offline."}{" "}
            Card artwork loads when online.
          </p>
          <p>
            {catalog.data.arena_aliases.toLocaleString()} installed Arena identities ·{" "}
            {catalog.data.arena_ids.toLocaleString()} direct Scryfall Arena IDs
          </p>
        </>
      )}
      <Button
        color={empty ? "primary" : "secondary"}
        isDisabled={!catalog.data}
        isLoading={refresh.isPending}
        onPress={() => refresh.mutate({})}
      >
        {empty ? "Download card metadata" : "Update card metadata"}
      </Button>
      {refresh.isPending && <Loading>Downloading and importing card metadata…</Loading>}
      {refresh.isError && <Failure error={refresh.error} />}
      <ArenaCacheControls hasIdentities={(catalog.data?.arena_aliases ?? 0) > 0} />
    </section>
  );
}
