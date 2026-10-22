import { Link, useNavigate } from "@tanstack/react-router";
import { useServices } from "../app/services.js";
import { useRequestKey } from "../app/request-key.js";
import { Button } from "../ui/button.js";
import { Empty, Failure, Loading } from "../ui/states.js";
import { Pagination, usePages } from "../ui/pagination.js";
export function PracticePage() {
  const { api, queryClient } = useServices();
  const pages = usePages();
  const navigate = useNavigate();
  const key = useRequestKey();
  const sessions = api.useQuery("get", "/v1/practice", {
    params: { query: { limit: 20, cursor: pages.cursor } },
  });
  const decks = api.useQuery("get", "/v1/decks", { params: { query: { limit: 100 } } });
  const create = api.useMutation("post", "/v1/practice", {
    onSuccess: (session) => {
      key.clear();
      void queryClient.invalidateQueries();
      void navigate({ to: "/practice/$sessionId", params: { sessionId: session.id } });
    },
  });
  return (
    <>
      <header className="page-heading">
        <div>
          <p className="eyebrow">LEARN YOUR OPENING LINES</p>
          <h1>Practice</h1>
          <p className="subtitle">A local manual sandbox. No opponent, no Arena smoothing.</p>
        </div>
      </header>
      <form
        className="panel"
        onSubmit={(event) => {
          event.preventDefault();
          const form = new window.FormData(event.currentTarget);
          const input = {
            version_id: String(form.get("version")),
            on_the_play: form.get("play") === "on",
          };
          create.mutate({ body: { idempotency_key: key.forBody(input), input } });
        }}
      >
        <h2>Start a practice hand</h2>
        {decks.isPending && <Loading />}
        {decks.isError && (
          <Failure
            error={decks.error}
            retry={() => {
              void decks.refetch();
            }}
          />
        )}
        <label className="field">
          <span>Deck</span>
          <select name="version" required defaultValue="">
            <option value="" disabled>
              Choose a deck
            </option>
            {decks.data?.items.map((d) => (
              <option key={d.id} value={d.latest_version}>
                {d.name}
              </option>
            ))}
          </select>
        </label>
        <label className="checkbox">
          <input name="play" type="checkbox" defaultChecked />
          On the play
        </label>
        <Button type="submit" isLoading={create.isPending} isDisabled={!decks.data?.items.length}>
          Draw opening hand
        </Button>
        {create.isError && <Failure error={create.error} />}
      </form>
      {sessions.isPending && <Loading>Loading saved sessions…</Loading>}
      {sessions.isError && (
        <Failure
          error={sessions.error}
          retry={() => {
            void sessions.refetch();
          }}
        />
      )}
      {sessions.data?.items.length === 0 && (
        <Empty title="A fresh table">
          Start a hand to practice keeping, mulligans and basic turns.
        </Empty>
      )}
      {!!sessions.data?.items.length && (
        <section className="panel">
          <h2>Saved sessions</h2>
          {sessions.data.items.map((s) => (
            <Link
              className="selection-row"
              key={s.id}
              to="/practice/$sessionId"
              params={{ sessionId: s.id }}
            >
              <strong>{s.name}</strong>
              <span>
                {s.phase} · Turn {s.turn} · {s.mulligans} mulligans
              </span>
            </Link>
          ))}
        </section>
      )}
      {sessions.data && (
        <Pagination pages={pages} next={sessions.data.next_cursor} busy={sessions.isFetching} />
      )}
    </>
  );
}
