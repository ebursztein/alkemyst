import type { components } from "@alkemyst/sdk";
import { Empty } from "../ui/states.js";
const percent = (p: number) => `${(p * 100).toFixed(1)}%`;
function Probability({ value }: { value: components["schemas"]["Estimate"] }) {
  return (
    <>
      <strong>{percent(value.probability)}</strong>
      <small>
        {value.interval95.map(percent).join("–")} · {value.successes.toLocaleString()} /{" "}
        {value.samples.toLocaleString()}
      </small>
    </>
  );
}
const labels: Record<components["schemas"]["JobStatus"], string> = {
  queued: "Queued",
  running: "Running",
  cancelling: "Cancelling",
  completed: "Completed",
  cancelled: "Cancelled",
  interrupted: "Interrupted",
  failed: "Failed",
};
export function SimulationReport({ job }: { job: components["schemas"]["SimulationJob"] }) {
  return (
    <section className="panel simulation-report">
      <div className="panel-heading">
        <h2>Results</h2>
        <span className="badge" role="status">
          {labels[job.status]} · {job.completed_runs.toLocaleString()} /{" "}
          {job.requested_runs.toLocaleString()} trials
        </span>
      </div>
      {job.error_code && (
        <p className="notice" role="status">
          Run stopped: {job.error_code}. Any reported samples below are completed trials.
        </p>
      )}
      {job.report ? (
        <>
          <p>
            At each turn: at least {job.report.goal.equipment} Equipment, {job.report.goal.attached}{" "}
            attached, and {job.report.goal.attack_ready} attached to an attack-ready creature.
          </p>
          <div className="table-scroll">
            <table className="probability-table" aria-label="Simulation probabilities by turn">
              <thead>
                <tr>
                  <th>Turn</th>
                  <th>Joint goal</th>
                  <th>Equipment</th>
                  <th>Attached</th>
                  <th>Attack-ready</th>
                  <th>Mean legendary bricks</th>
                </tr>
              </thead>
              <tbody>
                {job.report.turns.map((t) => (
                  <tr key={t.turn}>
                    <th>{t.turn}</th>
                    <td className="joint">
                      <Probability value={t.joint} />
                    </td>
                    <td>
                      <Probability value={t.equipment} />
                    </td>
                    <td>
                      <Probability value={t.attached} />
                    </td>
                    <td>
                      <Probability value={t.attack_ready} />
                    </td>
                    <td>{t.mean_legendary_bricks.toFixed(2)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="caption">
            Sampling intervals are 95% Wilson intervals, conditional on this model and its policy.
            They do not measure model uncertainty or win rate.
          </p>
        </>
      ) : (
        <Empty title="No completed samples yet">
          The native worker will save results as complete trials finish.
        </Empty>
      )}
      <details>
        <summary>Replay metadata</summary>
        <dl className="metadata">
          <dt>Seed</dt>
          <dd>
            <code>{job.seed}</code>
          </dd>
          <dt>Compiler</dt>
          <dd>{job.compiler_version}</dd>
          <dt>Model</dt>
          <dd>{job.model_version}</dd>
          <dt>Policy</dt>
          <dd>{job.policy_version}</dd>
          <dt>Random stream</dt>
          <dd>{job.replay_version}</dd>
          <dt>Catalog snapshot</dt>
          <dd>{job.catalog_snapshot ?? "Unavailable"}</dd>
        </dl>
      </details>
    </section>
  );
}
