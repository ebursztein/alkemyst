import { Link } from "@tanstack/react-router";
import { useServices } from "../app/services.js";
import { Empty, Failure, Loading } from "../ui/states.js";
import { Pagination, usePages } from "../ui/pagination.js";
import { ImportDeck } from "../decks/import-deck.js";
export function DecksPage() {
  const { api } = useServices();
  const pages = usePages();
  const query = api.useQuery("get", "/v1/decks", {
    params: { query: { limit: 24, cursor: pages.cursor } },
  });
  return (
    <>
      <header className="page-heading">
        <div>
          <p className="eyebrow">YOUR COLLECTION</p>
          <h1>Decks</h1>
          <p className="subtitle">Build a better brew. Keep every version.</p>
        </div>
        <ImportDeck />
      </header>
      {query.isPending && <Loading />}
      {query.isError && (
        <Failure
          error={query.error}
          retry={() => {
            void query.refetch();
          }}
        />
      )}
      {query.data?.items.length === 0 && (
        <Empty title="Your next brew starts here">
          Import an Arena deck or let tracking capture a submitted deck.
        </Empty>
      )}
      <div className="deck-grid">
        {query.data?.items.map((deck) => (
          <Link
            className="deck-tile"
            key={deck.id}
            to="/decks/$versionId"
            params={{ versionId: deck.latest_version }}
          >
            <div className="deck-art" aria-hidden="true">
              <span>✦</span>
              <span className="deck-art-count">{deck.main_count}</span>
            </div>
            <div className="deck-tile-body">
              <span className="badge">{deck.format}</span>
              <h2>{deck.name}</h2>
              <p>
                {deck.main_count} cards <span>·</span> {deck.version_count}{" "}
                {deck.version_count === 1 ? "version" : "versions"}
              </p>
              <span className="deck-link">
                Open deck <span aria-hidden="true">↗</span>
              </span>
            </div>
          </Link>
        ))}
      </div>
      {query.data && (
        <Pagination pages={pages} next={query.data.next_cursor} busy={query.isFetching} />
      )}
    </>
  );
}
