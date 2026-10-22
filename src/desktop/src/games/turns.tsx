import { useServices } from "../app/services.js";
import { Failure, Loading } from "../ui/states.js";
import { Pagination, usePages } from "../ui/pagination.js";
export function ObservedTurns({ id }: { id: string }) {
  const { api } = useServices();
  const pages = usePages();
  const turns = api.useQuery("get", "/v1/games/{id}/turns", {
    params: { path: { id }, query: { limit: 30, cursor: pages.cursor } },
  });
  return (
    <section className="panel">
      <h2>Observed turns</h2>
      <p className="muted">
        Counters are sampled from captured states; unavailable values remain blank.
      </p>
      {turns.isPending && <Loading />}
      {turns.isError && (
        <Failure
          error={turns.error}
          retry={() => {
            void turns.refetch();
          }}
        />
      )}
      {turns.data && (
        <>
          <div className="table-scroll">
            <table aria-label="Observed turns">
              <thead>
                <tr>
                  <th>Turn</th>
                  <th>Player</th>
                  <th>Local life</th>
                  <th>Land drops recorded</th>
                  <th>Actions</th>
                  <th>States</th>
                </tr>
              </thead>
              <tbody>
                {turns.data.items.map((t) => (
                  <tr key={t.number}>
                    <th>{t.number}</th>
                    <td>
                      {t.is_local_turn == null ? "Unknown" : t.is_local_turn ? "You" : "Opponent"}
                    </td>
                    <td>{t.local_life_total ?? "—"}</td>
                    <td>
                      {t.land_drops_observed ?? "—"}
                      <small>{t.land_counter_samples} samples</small>
                    </td>
                    <td>{t.actions}</td>
                    <td>{t.states}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="caption">
            {turns.data.unassigned_events} events without a verified turn assignment.
          </p>
          <Pagination pages={pages} next={turns.data.next_cursor} busy={turns.isFetching} />
        </>
      )}
    </section>
  );
}
