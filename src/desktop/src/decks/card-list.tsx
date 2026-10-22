import type { components } from "@alkemyst/sdk";
import { useServices } from "../app/services.js";
import { cardImage, metadataText } from "./card-data.js";
import { Failure, Loading } from "../ui/states.js";
export function CardList({ version }: { version: components["schemas"]["DeckVersion"] }) {
  const { api } = useServices();
  const result = api.useQuery("post", "/v1/cards/resolve-references", {
    body: { references: version.cards.map((c) => c.reference) },
  });
  const data = new Map(result.data?.map((row) => [row.reference, row.card]));
  return (
    <section className="panel">
      <div className="panel-heading">
        <h2>Card list</h2>
        <span className="muted">Expand a card to inspect its rules</span>
      </div>
      {result.isPending && <Loading>Resolving local cards…</Loading>}
      {result.isError && (
        <Failure
          error={result.error}
          retry={() => {
            void result.refetch();
          }}
        />
      )}
      {result.data &&
        version.cards.map((card) => {
          const record = data.get(card.reference);
          const image = cardImage(record);
          const name = metadataText(record, "name");
          return (
            <details className="card-row" key={`${card.zone}:${card.reference}`}>
              <summary>
                <span className="quantity">{card.quantity}</span>
                <strong>{name ?? "Unresolved card"}</strong>
                <span className="card-cost">{metadataText(record, "mana_cost")}</span>
                <span className="badge">{card.zone}</span>
              </summary>
              <div className="card-expanded">
                {image && <img src={image} alt={`${name} card`} loading="lazy" width="160" />}
                <div>
                  <p className="muted">{metadataText(record, "type_line") ?? card.reference}</p>
                  <p className="rules-text">
                    {metadataText(record, "oracle_text") ??
                      "Rules text is unavailable for this reference."}
                  </p>
                </div>
              </div>
            </details>
          );
        })}
    </section>
  );
}
