import { Link, useParams } from "@tanstack/react-router";
import { useServices } from "../app/services.js";
import { Failure, Loading } from "../ui/states.js";
import { usePages, Pagination } from "../ui/pagination.js";
import { ObservedTurns } from "../games/turns.js";
import { outcomeLabel } from "./games.js";
function Timeline({ id }: { id: string }) {
  const { api } = useServices();
  const pages = usePages();
  const events = api.useQuery("get", "/v1/games/{id}/events", {
    params: { path: { id }, query: { limit: 50, cursor: pages.cursor } },
  });
  return (
    <section className="panel">
      <h2>Event timeline</h2>
      <p className="muted">Captured order; missing actions are not reconstructed.</p>
      {events.isPending && <Loading />}
      {events.isError && (
        <Failure
          error={events.error}
          retry={() => {
            void events.refetch();
          }}
        />
      )}
      <ol className="timeline">
        {events.data?.items.map((event) => (
          <li key={event.id}>
            <span className="timeline-turn">
              {event.turn_number == null ? "Unassigned" : `Turn ${event.turn_number}`}
            </span>
            <strong>{event.kind.replaceAll("_", " ")}</strong>
            <small>{[event.phase, event.step, event.local_time].filter(Boolean).join(" · ")}</small>
          </li>
        ))}
      </ol>
      {events.data && (
        <Pagination pages={pages} next={events.data.next_cursor} busy={events.isFetching} />
      )}
    </section>
  );
}
export function GameDetailPage() {
  const { gameId } = useParams({ from: "/games/$gameId" });
  const { api } = useServices();
  const game = api.useQuery("get", "/v1/games/{id}", { params: { path: { id: gameId } } });
  if (game.isPending) return <Loading />;
  if (game.isError)
    return (
      <Failure
        error={game.error}
        retry={() => {
          void game.refetch();
        }}
      />
    );
  const g = game.data;
  return (
    <>
      <header className="page-heading">
        <div>
          <Link className="back-link" to="/games">
            ← Game history
          </Link>
          <h1>{g.deck_name ?? "Captured game"}</h1>
          <p className="subtitle">
            Game {g.number} · {outcomeLabel(g.outcome)} · {g.mode}
          </p>
        </div>
        {g.deck_version && (
          <Link
            className="button button-secondary"
            to="/decks/$versionId"
            params={{ versionId: g.deck_version }}
          >
            Captured deck version
          </Link>
        )}
      </header>
      {g.missing_data.length > 0 && (
        <section className="notice">
          <h2>Partial capture</h2>
          <p>
            Available history remains useful. These gaps were detected:{" "}
            {g.missing_data.map((s) => s.replaceAll("_", " ")).join(", ")}.
          </p>
        </section>
      )}
      <div className="metrics">
        <div>
          <span>Events</span>
          <strong>{g.events}</strong>
        </div>
        <div>
          <span>Turns observed</span>
          <strong>{g.turns_observed}</strong>
        </div>
        <div>
          <span>Mulligans recorded</span>
          <strong>{g.mulligans ?? "—"}</strong>
        </div>
        <div>
          <span>Land counter total</span>
          <strong>{g.land_drops_observed ?? "—"}</strong>
          <small>{g.land_turn_samples} sampled local turns</small>
        </div>
      </div>
      <ObservedTurns key={`turns:${gameId}`} id={gameId} />
      <Timeline key={gameId} id={gameId} />
    </>
  );
}
