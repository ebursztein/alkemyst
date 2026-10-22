import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { readNativeSettings, updateNativeSetting } from "../app/native-settings.js";
import { Failure, Loading } from "./states.js";
const key = ["native-settings"] as const;
export function NativeControls() {
  const client = useQueryClient();
  const settings = useQuery({
    queryKey: key,
    queryFn: readNativeSettings,
    staleTime: 0,
    refetchInterval: 2000,
  });
  const update = useMutation({
    mutationFn: updateNativeSetting,
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: key });
    },
  });
  return (
    <section className="panel">
      <h2>Desktop behavior</h2>
      <p className="muted">
        Closing the window keeps tracking and MCP running in the tray. Choose Quit Alkemyst to stop
        them.
      </p>
      {settings.isPending && <Loading>Reading desktop settings…</Loading>}
      {settings.isError && (
        <Failure
          error={settings.error}
          retry={() => {
            void settings.refetch();
          }}
        />
      )}
      {settings.data && (
        <p role="status">
          {settings.data.tracking_paused
            ? "Arena tracking paused"
            : {
                waiting: "Waiting for Arena logs",
                tracking: "Following Arena logs",
                degraded: "Arena tracking needs attention",
                stopped: "Arena tracking stopped",
              }[settings.data.tracking.state]}
        </p>
      )}
      {settings.data?.tracking.state === "degraded" && (
        <p className="muted">
          Capture is unavailable. Check that Arena is installed and its logs and local data are
          accessible, then pause and resume tracking to retry.
        </p>
      )}
      {settings.data && (
        <fieldset disabled={update.isPending} className="form-section">
          <legend className="visually-hidden">Native desktop settings</legend>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={settings.data.launch_at_login}
              onChange={(event) => {
                update.mutate({ setting: "launch_at_login", value: event.target.checked });
              }}
            />{" "}
            Launch Alkemyst at login
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={settings.data.tracking_paused}
              onChange={(event) => {
                update.mutate({ setting: "tracking_paused", value: event.target.checked });
              }}
            />{" "}
            Pause Arena tracking
          </label>
          <p className="muted">
            Pausing tracking leaves your library and MCP connection available.
          </p>
        </fieldset>
      )}
      {update.isPending && <p role="status">Updating desktop settings…</p>}
      {update.isError && <Failure error={update.error} />}
    </section>
  );
}
