import { useState } from "react";
import type { components } from "@alkemyst/sdk";
import { useServices } from "../app/services.js";
import { Button } from "../ui/button.js";
import { Failure, Loading } from "../ui/states.js";
import { SimulationForm } from "./simulation-form.js";
import { SimulationReport } from "./report.js";
import { ModelBoundaries } from "./model-boundaries.js";
const active = (status?: string) =>
  status === "queued" || status === "running" || status === "cancelling";
export function Experiments({ profile }: { profile: components["schemas"]["Profile"] }) {
  const { api, queryClient } = useServices();
  const [selected, setSelected] = useState<string>();
  const runs = api.useQuery("get", "/v1/simulations", {
    params: { query: { limit: 20, version_id: profile.version_id } },
  });
  const id = selected ?? runs.data?.items[0]?.id;
  const job = api.useQuery(
    "get",
    "/v1/simulations/{id}",
    { params: { path: { id: id ?? "" } } },
    { enabled: !!id, refetchInterval: (q) => (active(q.state.data?.status) ? 350 : false) },
  );
  const inputs = api.useQuery(
    "get",
    "/v1/simulations/{id}/inputs",
    { params: { path: { id: id ?? "" } } },
    { enabled: !!id },
  );
  const cancel = api.useMutation("post", "/v1/simulations/{id}/cancel", {
    onSuccess: () => {
      void queryClient.invalidateQueries();
    },
  });
  return (
    <>
      <SimulationForm
        versionId={profile.version_id}
        supported={profile.can_simulate}
        onCreated={setSelected}
      />
      {runs.isError && (
        <Failure
          error={runs.error}
          retry={() => {
            void runs.refetch();
          }}
        />
      )}
      {!!runs.data?.items.length && (
        <label className="field run-picker">
          <span>Saved experiments (latest 20)</span>
          <select value={id} onChange={(e) => setSelected(e.target.value)}>
            {runs.data.items.map((r) => (
              <option key={r.id} value={r.id}>
                {r.status} · {r.requested_runs.toLocaleString()} trials · {r.seed}
              </option>
            ))}
          </select>
        </label>
      )}
      {id && job.isPending && <Loading>Loading saved results…</Loading>}
      {job.isError && (
        <Failure
          error={job.error}
          retry={() => {
            void job.refetch();
          }}
        />
      )}
      {job.data && (
        <>
          <SimulationReport job={job.data} />
          {active(job.data.status) && (
            <div className="actions">
              <Button
                color="secondary"
                isLoading={cancel.isPending}
                onPress={() => cancel.mutate({ params: { path: { id: job.data.id } } })}
              >
                Cancel simulation
              </Button>
            </div>
          )}
          <details className="panel">
            <summary>Frozen assumptions for this run</summary>
            {inputs.isPending && <Loading />}
            {inputs.isError && (
              <Failure
                error={inputs.error}
                retry={() => {
                  void inputs.refetch();
                }}
              />
            )}
            {inputs.data && <ModelBoundaries profile={inputs.data.profile} />}
          </details>
        </>
      )}
      {cancel.isError && <Failure error={cancel.error} />}
    </>
  );
}
