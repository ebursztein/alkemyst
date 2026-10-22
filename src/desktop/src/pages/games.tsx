import { Link } from "@tanstack/react-router";
import { useState } from "react";
import type { components } from "@alkemyst/sdk";
import { useServices } from "../app/services.js";
import { Empty, Failure, Loading } from "../ui/states.js";
import { Pagination, usePages } from "../ui/pagination.js";
export const outcomeLabel = (
  outcome: components["schemas"]["ObservedOutcome"] | null | undefined,
) =>
  outcome === "won" ? "Win" : outcome === "lost" ? "Loss" : outcome === "draw" ? "Draw" : "Unknown";
export function GamesPage() {
  const { api } = useServices();
  const pages = usePages();
  const [mode, setMode] = useState<components["schemas"]["MatchMode"] | "all">("all");
  const filter = mode === "all" ? undefined : mode;
  const games = api.useQuery("get", "/v1/games", {
    params: { query: { limit: 30, cursor: pages.cursor, mode: filter } },
  });
  const stats = api.useQuery("get", "/v1/statistics", { params: { query: { mode: filter } } });
  return (
    <>
      <header className="page-heading">
        <div>
          <p className="eyebrow">OBSERVED, NOT INFERRED</p>
          <h1>Games</h1>
          <p className="subtitle">Your captured Arena history, with gaps kept visible.</p>
        </div>
        <label className="field">
          <span>Match mode</span>
          <select
            value={mode}
            onChange={(e) => {
              setMode(e.target.value as typeof mode);
              pages.reset();
            }}
          >
            <option value="all">All modes</option>
            <option value="bo1">Best of one</option>
            <option value="bo3">Best of three</option>
            <option value="unknown">Unknown mode</option>
          </select>
        </label>
      </header>
      {stats.isError && (
        <Failure
          error={stats.error}
          retry={() => {
            void stats.refetch();
          }}
        />
      )}
      {stats.data && (
        <div className="metrics">
          <div>
            <span>Games / matches</span>
            <strong>
              {stats.data.games} / {stats.data.matches}
            </strong>
          </div>
          <div>
            <span>Decisive win rate</span>
            <strong>
              {stats.data.decisive_win_rate == null
                ? "—"
                : `${(stats.data.decisive_win_rate * 100).toFixed(1)}%`}
            </strong>
            <small>{stats.data.decisive_games} wins + losses</small>
          </div>
          <div>
            <span>Unknown outcomes</span>
            <strong>{stats.data.unknown_results}</strong>
          </div>
          <div>
            <span>Partial captures</span>
            <strong>{stats.data.partial_games}</strong>
          </div>
        </div>
      )}
      {games.isPending && <Loading>Reading local history…</Loading>}
      {games.isError && (
        <Failure
          error={games.error}
          retry={() => {
            void games.refetch();
          }}
        />
      )}
      {games.data?.items.length === 0 && (
        <Empty title="No captured games yet">
          Arena tracking will add games here. Unknown outcomes never count as losses.
        </Empty>
      )}
      {!!games.data?.items.length && (
        <div className="panel table-scroll">
          <table aria-label="Captured games">
            <thead>
              <tr>
                <th>Deck</th>
                <th>Outcome</th>
                <th>Mode</th>
                <th>Play / draw</th>
                <th>Capture</th>
                <th>Turns observed</th>
              </tr>
            </thead>
            <tbody>
              {games.data.items.map((game) => (
                <tr key={game.id}>
                  <td>
                    <Link to="/games/$gameId" params={{ gameId: game.id }}>
                      {game.deck_name ?? "Unresolved deck"}
                    </Link>
                    <small>
                      Game {game.number} · {game.local_time ?? "Time unavailable"}
                    </small>
                  </td>
                  <td>
                    <span className={`badge ${game.outcome === "won" ? "badge-success" : ""}`}>
                      {outcomeLabel(game.outcome)}
                    </span>
                  </td>
                  <td>{game.mode}</td>
                  <td>{game.on_play == null ? "Unknown" : game.on_play ? "Play" : "Draw"}</td>
                  <td>{game.capture_complete ? "No detected gaps" : "Partial"}</td>
                  <td>{game.turns_observed}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {games.data && (
        <Pagination pages={pages} next={games.data.next_cursor} busy={games.isFetching} />
      )}
    </>
  );
}
