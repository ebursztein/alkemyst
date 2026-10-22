import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import type { components } from "@alkemyst/sdk";
import { useServices } from "../app/services.js";
import { useRequestKey } from "../app/request-key.js";
import { Field } from "../ui/field.js";
import { Button } from "../ui/button.js";
import { Failure, Loading } from "../ui/states.js";
type Version = components["schemas"]["DeckVersion"];
export function DeckEditor({ version, close }: { version: Version; close: () => void }) {
  const { api, queryClient } = useServices();
  const [name, setName] = useState(version.name);
  const [cards, setCards] = useState(version.cards);
  const [search, setSearch] = useState("");
  const [query, setQuery] = useState("");
  const navigate = useNavigate();
  const key = useRequestKey();
  const resolved = api.useQuery("post", "/v1/cards/resolve-references", {
    body: { references: version.cards.map((c) => c.reference) },
  });
  const matches = api.useQuery(
    "post",
    "/v1/cards/search",
    { body: { text: query, limit: 10, arena_only: false } },
    { enabled: query.length >= 2 },
  );
  const [addedNames, setAddedNames] = useState<Record<string, string>>({});
  const names = new Map(
    resolved.data?.map((c) => [
      c.reference,
      c.card && typeof c.card === "object" && "name" in c.card ? String(c.card.name) : c.reference,
    ]),
  );
  const save = api.useMutation("post", "/v1/deck-versions", {
    onSuccess: (v) => {
      key.clear();
      void queryClient.invalidateQueries();
      close();
      void navigate({ to: "/decks/$versionId", params: { versionId: v.id } });
    },
  });
  return (
    <section className="panel" aria-label="Edit deck version">
      <h2>Edit a new version</h2>
      <p className="muted">
        This saves a new version. The original deck and its captured games stay intact. Set quantity
        to 0 to remove a card.
      </p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const input = {
            deck_id: version.deck_id,
            parent_version: version.id,
            name,
            format: version.format,
            cards: cards.filter((c) => c.quantity > 0),
          };
          save.mutate({ body: { idempotency_key: key.forBody(input), input } });
        }}
      >
        <Field
          label="Version name"
          required
          maxLength={200}
          value={name}
          onChange={(e) => setName(e.target.value)}
          autoFocus
        />
        {resolved.isPending && <Loading>Resolving card names…</Loading>}
        <div className="editor-list">
          {/* Draft rows only append; an editable zone cannot serve as their identity. */}
          {resolved.data &&
            cards.map((c, i) => (
              <div className="editor-row" key={`${i}:${c.reference}`}>
                <label>
                  <span>{addedNames[c.reference] ?? names.get(c.reference) ?? c.reference}</span>
                  <input
                    aria-label={`Quantity for ${addedNames[c.reference] ?? names.get(c.reference) ?? c.reference}, line ${i + 1}`}
                    type="number"
                    min={0}
                    max={250}
                    required
                    value={c.quantity}
                    onChange={(e) =>
                      setCards((rows) =>
                        rows.map((row, j) =>
                          i === j ? { ...row, quantity: Number(e.target.value) } : row,
                        ),
                      )
                    }
                  />
                </label>
                <select
                  aria-label={`Zone for ${addedNames[c.reference] ?? names.get(c.reference) ?? c.reference}, line ${i + 1}`}
                  value={c.zone}
                  onChange={(e) =>
                    setCards((rows) =>
                      rows.map((row, j) =>
                        i === j ? { ...row, zone: e.target.value as typeof c.zone } : row,
                      ),
                    )
                  }
                >
                  {["main", "sideboard", "command", "companion"].map((z) => (
                    <option key={z}>{z}</option>
                  ))}
                </select>
              </div>
            ))}
        </div>
        {save.isError && <Failure error={save.error} />}
        <div className="actions">
          <Button type="submit" isLoading={save.isPending}>
            Save new version
          </Button>
          <Button color="secondary" isDisabled={save.isPending} onPress={close}>
            Cancel edit
          </Button>
        </div>
      </form>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          setQuery(search.trim());
        }}
        className="card-search"
      >
        <Field
          label="Search local cards"
          required
          minLength={2}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Name, type or rules text"
        />
        <Button type="submit" color="secondary">
          Search cards
        </Button>
      </form>
      {matches.isFetching && <Loading>Searching the local catalog…</Loading>}
      {matches.isError && (
        <Failure
          error={matches.error}
          retry={() => {
            void matches.refetch();
          }}
        />
      )}
      {matches.data && (
        <ul className="search-results">
          {matches.data.items.map((card) => (
            <li key={card.oracle_id}>
              <div>
                <strong>{card.name}</strong>
                <small>
                  {card.mana_cost} · {card.type_line}
                </small>
              </div>
              <Button
                color="secondary"
                onPress={() => {
                  const reference = `oracle:${card.oracle_id}`;
                  setAddedNames((n) => ({ ...n, [reference]: card.name }));
                  setCards((rows) =>
                    rows.some((c) => c.reference === reference && c.zone === "main")
                      ? rows.map((c) =>
                          c.reference === reference && c.zone === "main"
                            ? { ...c, quantity: c.quantity + 1 }
                            : c,
                        )
                      : [...rows, { reference, quantity: 1, zone: "main" }],
                  );
                }}
              >
                Add {card.name}
              </Button>
            </li>
          ))}
          {matches.data.items.length === 0 && <li>No matching local cards.</li>}
        </ul>
      )}
      {resolved.isError && (
        <Failure
          error={resolved.error}
          retry={() => {
            void resolved.refetch();
          }}
        />
      )}
    </section>
  );
}
