import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { readCardCacheHealth, updateCardCache } from "../app/native-card-cache.js";
import { Button } from "./button.js";
import { Failure, Loading } from "./states.js";
const key = ["native-card-cache-status"] as const;

export function ArenaCacheControls({ hasIdentities }: { hasIdentities: boolean }) {
  const client = useQueryClient();
  const health = useQuery({ queryKey: key, queryFn: readCardCacheHealth, refetchInterval: 2000 });
  const update = useMutation({
    mutationFn: updateCardCache,
    retry: false,
    onSuccess: () => {
      void client.invalidateQueries({ queryKey: key });
    },
  });
  const stopped = health.data?.state === "stopped" || health.data?.last_error === "worker_stopped";
  return (
    <div className="form-section">
      <h3>Arena card identities</h3>
      {health.isPending && <Loading>Checking Arena installation…</Loading>}
      {health.isError && (
        <Failure
          error={health.error}
          retry={() => {
            void health.refetch();
          }}
        />
      )}
      {health.data && (
        <>
          <p role={health.data.state === "degraded" ? "alert" : "status"}>
            {stopped
              ? "Card discovery stopped. Restart Alkemyst to resume."
              : {
                  disabled: "Automatic card discovery is disabled for this session.",
                  searching:
                    health.data.last_error === "metadata_busy"
                      ? "Waiting for the metadata update to finish…"
                      : "Checking installed Arena identities…",
                  ready: "Installed Arena identities imported.",
                  missing: "No installed Arena card cache found.",
                  degraded: "Could not read the installed Arena card cache.",
                  stopped: "Card discovery stopped.",
                }[health.data.state]}
          </p>
          {hasIdentities && health.data.state !== "ready" && (
            <p className="muted">Previously imported identities remain available locally.</p>
          )}
          <p className="muted">
            Arena uses additional card IDs. Alkemyst reads its public card database without changing
            game files. For a custom install, choose its Raw_CardDatabase_*.mtga file.
          </p>
          <div className="actions">
            <Button
              color="secondary"
              isDisabled={update.isPending || stopped}
              onPress={() => update.mutate("retry")}
            >
              Retry card discovery
            </Button>
            <Button
              color="secondary"
              isDisabled={stopped || (update.isPending && update.variables !== "choose")}
              aria-disabled={update.isPending || undefined}
              onPress={() => {
                if (!update.isPending) update.mutate("choose");
              }}
            >
              Choose Arena card cache…
            </Button>
          </div>
        </>
      )}
      {update.isPending && <Loading>Updating card discovery…</Loading>}
      {update.isError && <Failure error={update.error} />}
    </div>
  );
}
