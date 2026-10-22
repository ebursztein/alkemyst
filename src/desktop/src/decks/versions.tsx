import { useState } from "react";
import { Link } from "@tanstack/react-router";
import { useServices } from "../app/services.js";
import { Button } from "../ui/button.js";
import { Empty, Failure, Loading } from "../ui/states.js";
import { Pagination, usePages } from "../ui/pagination.js";
export function DeckVersions({ deckId, versionId }: { deckId: string; versionId: string }) {
  const { api } = useServices();
  const pages = usePages();
  const [base, setBase] = useState<string>();
  const versions = api.useQuery("get", "/v1/decks/{id}/versions", {
    params: { path: { id: deckId }, query: { limit: 20, cursor: pages.cursor } },
  });
  const comparison = api.useQuery(
    "get",
    "/v1/deck-versions/{id}/compare/{candidate_id}",
    { params: { path: { id: base ?? "", candidate_id: versionId } } },
    { enabled: !!base },
  );
  return (
    <section className="panel">
      <h2>Version history</h2>
      <p className="muted">
        Compare a saved version against the version shown above. Printing-only changes cancel out.
      </p>
      {versions.isPending && <Loading />}
      {versions.isError && (
        <Failure
          error={versions.error}
          retry={() => {
            void versions.refetch();
          }}
        />
      )}
      {versions.data?.items.map((v) => (
        <div className="version-row" key={v.id}>
          <div>
            <Link to="/decks/$versionId" params={{ versionId: v.id }}>
              Version {v.ordinal} · {v.name}
            </Link>
            <small>
              {v.main_count} main · {v.sideboard_count} sideboard · {v.origin}
              {v.id === versionId ? " · Viewing" : ""}
            </small>
          </div>
          <Button color="quiet" onPress={() => setBase(v.id)}>
            Compare version {v.ordinal}
          </Button>
        </div>
      ))}
      {versions.data && (
        <Pagination pages={pages} next={versions.data.next_cursor} busy={versions.isFetching} />
      )}
      {base && comparison.isPending && <Loading>Comparing versions…</Loading>}
      {comparison.isError && (
        <Failure
          error={comparison.error}
          retry={() => {
            void comparison.refetch();
          }}
        />
      )}
      {comparison.data && (
        <div className="version-comparison">
          <h3>
            Version {comparison.data.base.ordinal} → Version {comparison.data.candidate.ordinal}
          </h3>
          <p className="muted">
            Quantity changes and printed curve deltas; no performance inference.
          </p>
          {comparison.data.cards.length === 0 ? (
            <Empty title="Same card quantities">
              No quantity changes were found by card identity and zone.
            </Empty>
          ) : (
            <table aria-label="Card quantity changes">
              <thead>
                <tr>
                  <th>Card</th>
                  <th>Zone</th>
                  <th>Before</th>
                  <th>After</th>
                  <th>Change</th>
                </tr>
              </thead>
              <tbody>
                {comparison.data.cards.map((c) => (
                  <tr key={`${c.zone}:${c.identity}`}>
                    <th>{c.name ?? c.identity}</th>
                    <td>{c.zone}</td>
                    <td>{c.base_quantity}</td>
                    <td>{c.candidate_quantity}</td>
                    <td>
                      {c.delta > 0 ? "+" : ""}
                      {c.delta}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          <table aria-label="Mana curve changes">
            <thead>
              <tr>
                <th>Printed mana value</th>
                <th>Before</th>
                <th>After</th>
                <th>Change</th>
              </tr>
            </thead>
            <tbody>
              {comparison.data.curve.map((c) => (
                <tr key={c.mana_value ?? "unknown"}>
                  <th>{c.mana_value ?? "Unknown"}</th>
                  <td>{c.base_quantity}</td>
                  <td>{c.candidate_quantity}</td>
                  <td>
                    {c.delta > 0 ? "+" : ""}
                    {c.delta}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
