import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useServices } from "../app/services.js";
import { useRequestKey } from "../app/request-key.js";
import { Button } from "../ui/button.js";
import { Field } from "../ui/field.js";
import { Failure } from "../ui/states.js";
export function ImportDeck() {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [format, setFormat] = useState("standard");
  const [text, setText] = useState("");
  const { api, queryClient } = useServices();
  const key = useRequestKey();
  const navigate = useNavigate();
  const mutation = api.useMutation("post", "/v1/decks/import-arena", {
    onSuccess: (version) => {
      key.clear();
      void queryClient.invalidateQueries();
      setOpen(false);
      void navigate({ to: "/decks/$versionId", params: { versionId: version.id } });
    },
  });
  if (!open)
    return (
      <Button onPress={() => setOpen(true)}>
        Import deck <span aria-hidden="true">＋</span>
      </Button>
    );
  return (
    <section className="panel import-panel" aria-label="Import Arena deck">
      <h2>Import from Arena</h2>
      <p>Paste an exported deck. The original list becomes its first immutable version.</p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const input = { name, format, text };
          mutation.mutate({ body: { idempotency_key: key.forBody(input), input } });
        }}
      >
        <Field
          label="Deck name"
          required
          value={name}
          onChange={(e) => setName(e.target.value)}
          maxLength={200}
        />
        <label className="field">
          <span>Format</span>
          <select value={format} onChange={(e) => setFormat(e.target.value)}>
            {["standard", "historic", "timeless", "explorer", "alchemy", "brawl"].map((f) => (
              <option key={f}>{f}</option>
            ))}
          </select>
        </label>
        <label className="field">
          <span>Arena deck text</span>
          <textarea
            required
            rows={7}
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={"Deck\n4 Card Name (SET) 123"}
          />
        </label>
        {mutation.isError && <Failure error={mutation.error} />}
        <div className="actions">
          <Button type="submit" isLoading={mutation.isPending}>
            Import
          </Button>
          <Button color="secondary" onPress={() => setOpen(false)} isDisabled={mutation.isPending}>
            Cancel
          </Button>
        </div>
      </form>
    </section>
  );
}
