import { useState } from "react";
import { useServices } from "../app/services.js";
import { useRequestKey } from "../app/request-key.js";
import { Button } from "../ui/button.js";
import { Failure, Loading } from "../ui/states.js";
import { Pagination, usePages } from "../ui/pagination.js";
export function DeckNotes({ deckId, versionId }: { deckId: string; versionId: string }) {
  const { api, queryClient } = useServices();
  const pages = usePages();
  const [body, setBody] = useState("");
  const key = useRequestKey();
  const notes = api.useQuery("get", "/v1/notes", {
    params: { query: { deck_id: deckId, limit: 20, cursor: pages.cursor } },
  });
  const create = api.useMutation("post", "/v1/notes", {
    onSuccess: () => {
      key.clear();
      setBody("");
      pages.reset();
      void queryClient.invalidateQueries();
    },
  });
  return (
    <section className="panel">
      <h2>Lab notes</h2>
      <p className="muted">
        Notes across this deck's versions. New notes attach to the version you are viewing.
      </p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const input = { deck_id: deckId, version_id: versionId, body };
          create.mutate({ body: { input, idempotency_key: key.forBody(input) } });
        }}
      >
        <label className="field">
          <span>New note</span>
          <textarea
            required
            maxLength={10_000}
            rows={3}
            value={body}
            onChange={(e) => setBody(e.target.value)}
            placeholder="What are you testing in this brew?"
          />
        </label>
        <Button type="submit" color="secondary" isLoading={create.isPending}>
          Save note
        </Button>
      </form>
      {create.isError && <Failure error={create.error} />}
      {notes.isPending && <Loading />}
      {notes.isError && (
        <Failure
          error={notes.error}
          retry={() => {
            void notes.refetch();
          }}
        />
      )}
      {notes.data?.items.map((note) => (
        <article className="deck-note" key={note.id}>
          <span className="badge">
            {note.origin} ·{" "}
            {note.version_id === versionId
              ? "This version"
              : note.version_id
                ? "Another version"
                : "Deck note"}
          </span>
          <p>{note.body}</p>
        </article>
      ))}
      {notes.data && (
        <Pagination pages={pages} next={notes.data.next_cursor} busy={notes.isFetching} />
      )}
    </section>
  );
}
