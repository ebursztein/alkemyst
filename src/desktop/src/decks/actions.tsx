import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import type { components } from "@alkemyst/sdk";
import { useServices } from "../app/services.js";
import { useRequestKey } from "../app/request-key.js";
import { Button } from "../ui/button.js";
import { Failure } from "../ui/states.js";
import { DeckEditor } from "./editor.js";
export function DeckActions({ version }: { version: components["schemas"]["DeckVersion"] }) {
  const { api, client, queryClient } = useServices();
  const key = useRequestKey();
  const navigate = useNavigate();
  const [editing, setEditing] = useState(false);
  const [exported, setExported] = useState<string>();
  const [exportError, setExportError] = useState<unknown>();
  const [exporting, setExporting] = useState(false);
  const [copied, setCopied] = useState(false);
  const [copyFailed, setCopyFailed] = useState(false);
  const duplicate = api.useMutation("post", "/v1/decks", {
    onSuccess: (v) => {
      key.clear();
      void queryClient.invalidateQueries();
      void navigate({ to: "/decks/$versionId", params: { versionId: v.id } });
    },
  });
  return (
    <>
      <div className="actions">
        <Button color="secondary" onPress={() => setEditing(!editing)}>
          {editing ? "Hide editor" : "Edit deck"}
        </Button>
        <Button
          color="secondary"
          isLoading={duplicate.isPending}
          onPress={() => {
            const input = {
              name: `${version.name} (copy)`,
              format: version.format,
              cards: version.cards,
            };
            duplicate.mutate({ body: { input, idempotency_key: key.forBody(input) } });
          }}
        >
          Duplicate deck
        </Button>
        <Button
          color="secondary"
          isLoading={exporting}
          onPress={async () => {
            setExporting(true);
            setExportError(undefined);
            try {
              const result = await client.GET("/v1/deck-versions/{id}/arena", {
                params: { path: { id: version.id } },
              });
              if (result.error) throw result.error;
              setExported(result.data.text);
              setCopied(false);
            } catch (error) {
              setExportError(error);
            } finally {
              setExporting(false);
            }
          }}
        >
          Export for Arena
        </Button>
      </div>
      {duplicate.isError && <Failure error={duplicate.error} />}
      {exportError !== undefined && <Failure error={exportError} />}
      {editing && <DeckEditor key={version.id} version={version} close={() => setEditing(false)} />}
      {exported !== undefined && (
        <section className="panel">
          <h2>Arena export</h2>
          <label className="field">
            <span>Copy this text into Arena</span>
            <textarea aria-label="Arena export text" readOnly rows={7} value={exported} />
          </label>
          <div className="actions">
            <Button
              color="secondary"
              onPress={async () => {
                try {
                  await navigator.clipboard.writeText(exported);
                  setCopied(true);
                  setCopyFailed(false);
                } catch {
                  setCopyFailed(true);
                }
              }}
            >
              Copy deck text
            </Button>
            <Button color="quiet" onPress={() => setExported(undefined)}>
              Close export
            </Button>
          </div>
          {copied && <p role="status">Deck text copied.</p>}
          {copyFailed && <p role="alert">Clipboard unavailable. Select and copy the deck text.</p>}
        </section>
      )}
    </>
  );
}
