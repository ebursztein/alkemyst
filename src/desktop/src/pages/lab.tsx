import { Link, useParams } from "@tanstack/react-router";
import { useServices } from "../app/services.js";
import { Empty, Failure, Loading } from "../ui/states.js";
import { ModelBoundaries } from "../lab/model-boundaries.js";
import { Experiments } from "../lab/experiments.js";
export function LabIndex() {
  const { api } = useServices();
  const decks = api.useQuery("get", "/v1/decks", { params: { query: { limit: 100 } } });
  return (
    <>
      <header className="page-heading">
        <div>
          <p className="eyebrow">TEST YOUR HYPOTHESIS</p>
          <h1>Lab</h1>
          <p className="subtitle">
            Joint probabilities, real casting costs, reproducible experiments.
          </p>
        </div>
      </header>
      {decks.isPending && <Loading />}
      {decks.isError && (
        <Failure
          error={decks.error}
          retry={() => {
            void decks.refetch();
          }}
        />
      )}
      {decks.data?.items.length === 0 && (
        <Empty title="Choose your first experiment">
          Import a deck in Decks to start an analysis.
        </Empty>
      )}
      <div className="panel">
        <h2>Choose a deck</h2>
        {decks.data?.items.map((d) => (
          <Link
            className="selection-row"
            key={d.id}
            to="/lab/$versionId"
            params={{ versionId: d.latest_version }}
          >
            <strong>{d.name}</strong>
            <span>
              {d.format} · {d.main_count} cards <span aria-hidden="true">→</span>
            </span>
          </Link>
        ))}
        {decks.data?.next_cursor && (
          <p className="muted">
            More decks are available in <Link to="/decks">Decks</Link>.
          </p>
        )}
      </div>
    </>
  );
}
export function LabPage() {
  const { versionId } = useParams({ from: "/lab/$versionId" });
  const { api } = useServices();
  const profile = api.useQuery("get", "/v1/deck-versions/{id}/analysis", {
    params: { path: { id: versionId } },
  });
  if (profile.isPending) return <Loading>Preparing the deck model…</Loading>;
  if (profile.isError)
    return (
      <Failure
        error={profile.error}
        retry={() => {
          void profile.refetch();
        }}
      />
    );
  return (
    <>
      <header className="page-heading">
        <div>
          <Link className="back-link" to="/lab">
            ← All experiments
          </Link>
          <h1>{profile.data.name} · Lab</h1>
          <p className="subtitle">Equipment, tutors and mana sequencing · precombat goldfish</p>
        </div>
        <Link className="button button-secondary" to="/decks/$versionId" params={{ versionId }}>
          View deck
        </Link>
      </header>
      <ModelBoundaries profile={profile.data} />
      <Experiments key={versionId} profile={profile.data} />
    </>
  );
}
