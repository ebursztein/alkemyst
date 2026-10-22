import { useState } from "react";
import { Link, useParams } from "@tanstack/react-router";
import type { components } from "@alkemyst/sdk";
import { useServices } from "../app/services.js";
import { useRequestKey } from "../app/request-key.js";
import { Button } from "../ui/button.js";
import { Failure, Loading } from "../ui/states.js";
import { PracticeArtwork, VisibleCard } from "../practice/visible-card.js";
export function PracticeTablePage() {
  const { sessionId } = useParams({ from: "/practice/$sessionId" });
  const { api, queryClient } = useServices();
  const key = useRequestKey();
  const session = api.useQuery("get", "/v1/practice/{id}", { params: { path: { id: sessionId } } });
  const [draft, setDraft] = useState<{ revision: number; bottom: number[] }>({
    revision: -1,
    bottom: [],
  });
  const mutation = api.useMutation("post", "/v1/practice/{id}/actions", {
    onSuccess: async () => {
      key.clear();
      await queryClient.invalidateQueries();
    },
    onError: () => {
      void session.refetch();
    },
  });
  if (session.isPending) return <Loading>Dealing the saved hand…</Loading>;
  if (session.isError)
    return (
      <Failure
        error={session.error}
        retry={() => {
          void session.refetch();
        }}
      />
    );
  const s = session.data;
  const busy = mutation.isPending || session.isFetching;
  const bottom = draft.revision === s.revision ? draft.bottom : [];
  function act(action: components["schemas"]["PracticeAction"]) {
    const body = { expected_revision: s.revision, action };
    mutation.mutate({
      params: { path: { id: sessionId } },
      body: { ...body, idempotency_key: key.forBody(body) },
    });
  }
  return (
    <>
      <header className="page-heading">
        <div>
          <Link className="back-link" to="/practice">
            ← Saved practice
          </Link>
          <h1>Practice table</h1>
          <p className="subtitle">
            {s.phase === "opening" ? "Opening hand" : `Turn ${s.turn}`} ·{" "}
            {s.on_the_play ? "On the play" : "On the draw"} · {s.mulligans} mulligans
          </p>
        </div>
        <span className="badge">{s.library_count} cards in library</span>
      </header>
      <section className="notice">
        <strong>Manual sandbox</strong>
        <p>
          Pay mana and resolve abilities yourself. Combat and the legend rule are not automated.
        </p>
        <details>
          <summary>Rules and replay details</summary>
          <ul>
            {s.warnings.map((w) => (
              <li key={w}>{w}</li>
            ))}
          </ul>
          <p>
            Seed <code>{s.seed}</code> · {s.practice_version}
          </p>
        </details>
      </section>
      {mutation.isError && (
        <Failure
          error={mutation.error}
          retry={() => {
            void session.refetch();
          }}
        />
      )}
      {s.phase === "opening" ? (
        <>
          <div className="actions">
            <Button
              color="secondary"
              isDisabled={busy || s.mulligans >= s.hand.length}
              onPress={() => act({ kind: "mulligan" })}
            >
              Mulligan
            </Button>
            <Button
              isDisabled={busy || bottom.length !== s.bottom_count}
              onPress={() => act({ kind: "keep", bottom })}
            >
              Keep hand
            </Button>
            <span>
              Choose {s.bottom_count} to bottom · {bottom.length} selected
            </span>
          </div>
          <div className="practice-hand">
            {s.hand.map((card) => (
              <label
                className={`practice-card bottom-choice ${bottom.includes(card.id) ? "selected" : ""}`}
                key={card.id}
              >
                <input
                  type="checkbox"
                  checked={bottom.includes(card.id)}
                  disabled={busy || s.bottom_count === 0}
                  aria-label={`Bottom ${card.name ?? "unresolved card"}, copy ${card.id + 1}`}
                  onChange={() =>
                    setDraft({
                      revision: s.revision,
                      bottom: bottom.includes(card.id)
                        ? bottom.filter((id) => id !== card.id)
                        : [...bottom, card.id],
                    })
                  }
                />
                <PracticeArtwork card={card} />
                <strong>{card.name ?? "Unresolved card"}</strong>
                <small>
                  {bottom.includes(card.id)
                    ? `Bottom position ${bottom.indexOf(card.id) + 1}`
                    : `Copy ${card.id + 1}`}
                </small>
              </label>
            ))}
          </div>
          <p className="caption">
            Choose bottom cards in order. They will follow the rest of the library.
          </p>
        </>
      ) : (
        <>
          {s.phase === "main" && (
            <div className="actions">
              <Button isDisabled={busy} onPress={() => act({ kind: "next_turn" })}>
                Next turn
              </Button>
              <Button color="secondary" isDisabled={busy} onPress={() => act({ kind: "draw" })}>
                Draw a card
              </Button>
              <span>{s.land_played ? "Land drop used" : "Land drop available"}</span>
            </div>
          )}
          {s.phase === "finished" && (
            <p className="notice" role="status">
              {s.decked
                ? "Practice finished after an empty-library draw."
                : "Practice session finished."}
            </p>
          )}
          <section className="table-zone">
            <h2>Hand · {s.hand.length}</h2>
            <div className="practice-hand">
              {s.hand.map((card) => (
                <VisibleCard
                  key={card.id}
                  card={card}
                  zone="hand"
                  busy={busy || s.phase === "finished"}
                  action={act}
                />
              ))}
            </div>
          </section>
          <section className="table-zone">
            <h2>Battlefield · {s.battlefield.length}</h2>
            <div className="practice-hand">
              {s.battlefield.map((p) => (
                <VisibleCard
                  key={p.card.id}
                  card={p.card}
                  face={p.face}
                  zone="battlefield"
                  tapped={p.tapped}
                  busy={busy || s.phase === "finished"}
                  action={act}
                />
              ))}
            </div>
          </section>
          {(["graveyard", "exile"] as const).map((zone) => (
            <details className="panel" key={zone}>
              <summary>
                {zone} · {s[zone].length}
              </summary>
              <div className="practice-hand">
                {s[zone].map((card) => (
                  <VisibleCard
                    key={card.id}
                    card={card}
                    zone={zone}
                    busy={busy || s.phase === "finished"}
                    action={act}
                  />
                ))}
              </div>
            </details>
          ))}
        </>
      )}
      {s.phase !== "finished" && (
        <div className="actions">
          <Button color="quiet" isDisabled={busy} onPress={() => act({ kind: "finish" })}>
            Finish practice
          </Button>
        </div>
      )}
    </>
  );
}
