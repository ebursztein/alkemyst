import { Link, useParams } from "@tanstack/react-router";
import { useServices } from "../app/services.js";
import { Failure, Loading } from "../ui/states.js";
import { ManaCurve } from "../decks/curve.js";
import { DeckActions } from "../decks/actions.js";
import { DeckNotes } from "../decks/notes.js";
import { DeckVersions } from "../decks/versions.js";
import { CardList } from "../decks/card-list.js";
export function DeckDetailPage() {
  const { versionId } = useParams({ from: "/decks/$versionId" });
  const { api } = useServices();
  const version = api.useQuery("get", "/v1/deck-versions/{id}", {
    params: { path: { id: versionId } },
  });
  const profile = api.useQuery("get", "/v1/deck-versions/{id}/analysis", {
    params: { path: { id: versionId } },
  });
  if (version.isPending) return <Loading />;
  if (version.isError)
    return (
      <Failure
        error={version.error}
        retry={() => {
          void version.refetch();
        }}
      />
    );
  return (
    <>
      <header className="page-heading">
        <div>
          <Link className="back-link" to="/decks">
            ← All decks
          </Link>
          <h1>{version.data.name}</h1>
          <p className="subtitle">
            {version.data.format} · Version {version.data.ordinal} ·{" "}
            <span className="badge">{version.data.origin}</span>
          </p>
        </div>
        <Link className="button button-primary" to="/lab/$versionId" params={{ versionId }}>
          Analyze deck <span aria-hidden="true">↗</span>
        </Link>
      </header>
      {profile.isPending && <Loading>Analyzing the printed curve…</Loading>}
      {profile.isError && (
        <Failure
          error={profile.error}
          retry={() => {
            void profile.refetch();
          }}
        />
      )}
      {profile.data && (
        <>
          <div className="metrics">
            <div>
              <span>Main deck</span>
              <strong>{profile.data.counts.main}</strong>
            </div>
            <div>
              <span>Lands</span>
              <strong>{profile.data.counts.lands}</strong>
              <small>{profile.data.counts.lands} lands</small>
            </div>
            <div>
              <span>Equipment</span>
              <strong>{profile.data.counts.equipment}</strong>
            </div>
            <div>
              <span>Creatures</span>
              <strong>{profile.data.counts.creatures}</strong>
            </div>
          </div>
          <div className="two-columns">
            <ManaCurve bins={profile.data.curve} />
            <section className="panel">
              <h2>Mana sources</h2>
              <p className="muted">
                Potential sources. Conditional sources depend on the spell and board.
              </p>
              <table>
                <thead>
                  <tr>
                    <th>Color</th>
                    <th>Lands</th>
                    <th>Conditional</th>
                    <th>Other</th>
                  </tr>
                </thead>
                <tbody>
                  {profile.data.sources.map((s) => (
                    <tr key={s.color}>
                      <th>{s.color}</th>
                      <td>{s.unrestricted_lands}</td>
                      <td>{s.conditional_lands}</td>
                      <td>{s.nonland_sources}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </section>
          </div>
        </>
      )}
      <DeckActions key={versionId} version={version.data} />
      <CardList version={version.data} />
      <DeckVersions
        key={`versions:${versionId}`}
        deckId={version.data.deck_id}
        versionId={versionId}
      />
      <DeckNotes key={`notes:${versionId}`} deckId={version.data.deck_id} versionId={versionId} />
    </>
  );
}
