import { useState } from "react";
import type { components } from "@alkemyst/sdk";
import { Button } from "../ui/button.js";
import { cardImage } from "../decks/card-data.js";
type Card = components["schemas"]["PracticeCard"];
type Action = components["schemas"]["PracticeAction"];
type Zone = components["schemas"]["PracticeZone"];
export function PracticeArtwork({ card, face = 0 }: { card: Card; face?: number }) {
  const image = cardImage({
    image_uris: { normal: card.faces[face]?.image_uri ?? card.image_uri },
  });
  return image ? (
    <img src={image} alt={`${card.name ?? "Unresolved"} card`} loading="lazy" />
  ) : (
    <div className="card-placeholder" aria-hidden="true">
      <span>✦</span>
      <small>{card.mana_cost ?? "?"}</small>
    </div>
  );
}
export function VisibleCard({
  card,
  zone,
  busy,
  action,
  tapped,
  face: currentFace = 0,
}: {
  card: Card;
  zone: Zone;
  busy: boolean;
  action: (a: Action) => void;
  tapped?: boolean;
  face?: number;
}) {
  const [face, setFace] = useState(currentFace);
  const [destination, setDestination] = useState<Zone>(
    zone === "graveyard" || zone === "exile" ? "hand" : "graveyard",
  );
  const type = card.faces[face]?.type_line ?? card.type_line ?? "";
  const name = card.name ?? "Unresolved card";
  return (
    <article className={`practice-card ${tapped ? "tapped" : ""}`}>
      <PracticeArtwork card={card} face={face} />
      <h3>{name}</h3>
      <small>
        Copy {card.id + 1} · {card.faces[face]?.mana_cost ?? card.mana_cost}
      </small>
      {card.faces.length > 1 && (
        <label className="field">
          <span>
            Face for {name}, copy {card.id + 1}
          </span>
          <select
            disabled={busy || zone !== "hand"}
            value={face}
            onChange={(e) => setFace(Number(e.target.value))}
          >
            {card.faces.map((f, i) => (
              <option key={i} value={i}>
                {f.name ?? `Face ${i + 1}`}
              </option>
            ))}
          </select>
        </label>
      )}
      {zone === "hand" && (
        <Button
          color="secondary"
          isDisabled={busy}
          onPress={() =>
            action(
              type.includes("Land")
                ? { kind: "play_land", card: card.id, face }
                : {
                    kind: "play",
                    card: card.id,
                    face,
                    destination: /Instant|Sorcery/.test(type) ? "graveyard" : "battlefield",
                  },
            )
          }
        >
          {type.includes("Land")
            ? "Play land"
            : /Instant|Sorcery/.test(type)
              ? "Resolve to graveyard"
              : "Play to battlefield"}
        </Button>
      )}
      {zone === "battlefield" && (
        <Button
          color="secondary"
          isDisabled={busy}
          onPress={() => action({ kind: "tap", card: card.id, tapped: !tapped })}
        >
          {tapped ? "Untap" : "Tap"} {name}
        </Button>
      )}
      <div className="card-move">
        <label>
          <span className="visually-hidden">
            Move {name}, copy {card.id + 1}, to
          </span>
          <select
            disabled={busy}
            value={destination}
            onChange={(e) => setDestination(e.target.value as Zone)}
          >
            {(["hand", "battlefield", "graveyard", "exile"] as const)
              .filter((z) => z !== zone)
              .map((z) => (
                <option key={z} value={z}>
                  {z}
                </option>
              ))}
          </select>
        </label>
        <Button
          color="quiet"
          isDisabled={busy}
          onPress={() => action({ kind: "move", card: card.id, destination })}
        >
          Move
        </Button>
      </div>
    </article>
  );
}
