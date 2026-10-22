import { NativeControls } from "../ui/native-controls.js";
import { useState } from "react";
import { useServices } from "../app/services.js";
import { Button } from "../ui/button.js";
import { Failure, Loading } from "../ui/states.js";
export function SettingsPage() {
  const { api, connection, queryClient } = useServices();
  const catalog = api.useQuery("get", "/v1/catalog");
  const capture = api.useQuery("get", "/v1/capture");
  const [revealed, setRevealed] = useState(false);
  const [copied, setCopied] = useState(false);
  const [copyError, setCopyError] = useState(false);
  const refresh = api.useMutation("post", "/v1/catalog/refresh", {
    onSuccess: () => {
      void queryClient.invalidateQueries();
    },
  });
  const config = {
    mcpServers: {
      alkemyst: {
        url: `${connection.base_url}/mcp`,
        headers: { Authorization: `Bearer ${revealed ? connection.token : "<local credential>"}` },
      },
    },
  };
  return (
    <>
      <header className="page-heading">
        <div>
          <p className="eyebrow">ON YOUR DEVICE</p>
          <h1>Settings</h1>
          <p className="subtitle">Local data, card metadata and agent connections.</p>
        </div>
      </header>
      <NativeControls />
      <section className="panel">
        <h2>Card library</h2>
        {catalog.isPending && <Loading />}
        {catalog.isError && (
          <Failure
            error={catalog.error}
            retry={() => {
              void catalog.refetch();
            }}
          />
        )}
        {catalog.data && (
          <>
            <p>
              <strong>{catalog.data.cards.toLocaleString()}</strong> card identities ·{" "}
              {catalog.data.printings.toLocaleString()} printings
            </p>
            <p className="muted">
              Snapshot: {catalog.data.source?.updated_at ?? "No catalog imported"}. Existing
              metadata remains usable without internet.
            </p>
          </>
        )}
        <Button color="secondary" isLoading={refresh.isPending} onPress={() => refresh.mutate({})}>
          Update card metadata
        </Button>
        {refresh.isError && <Failure error={refresh.error} />}
      </section>
      <section className="panel">
        <h2>Arena capture</h2>
        {capture.isPending && <Loading />}
        {capture.isError && (
          <Failure
            error={capture.error}
            retry={() => {
              void capture.refetch();
            }}
          />
        )}
        {capture.data && (
          <p>
            {capture.data.sources} tracked sources · {capture.data.games} games ·{" "}
            {capture.data.events} events · {capture.data.unknown_results} unknown results
          </p>
        )}
        <p className="muted">
          Captured history stays local. Native log discovery and tray controls are configured by the
          desktop shell.
        </p>
      </section>
      <section className="panel">
        <div className="panel-heading">
          <h2>MCP connection</h2>
          <span className="badge">Authenticated · loopback only</span>
        </div>
        <p>
          Connect your own agent while Alkemyst is running. Deck edits create new versions; captured
          history remains immutable. Connection credentials rotate when the app restarts; copy the
          current configuration after restarting.
        </p>
        <pre className="connection-config">{JSON.stringify(config, null, 2)}</pre>
        <div className="actions">
          <Button
            color="secondary"
            onPress={() => {
              setRevealed(!revealed);
              setCopied(false);
            }}
          >
            {" "}
            {revealed ? "Hide credential" : "Reveal credential"}
          </Button>
          {revealed && (
            <Button
              color="secondary"
              onPress={async () => {
                try {
                  await navigator.clipboard.writeText(JSON.stringify(config, null, 2));
                  setCopied(true);
                  setCopyError(false);
                } catch {
                  setCopyError(true);
                }
              }}
            >
              Copy connection config
            </Button>
          )}
        </div>
        {copied && <p role="status">Connection config copied.</p>}
        {copyError && (
          <p role="alert">Clipboard unavailable. Select and copy the displayed configuration.</p>
        )}
      </section>
    </>
  );
}
