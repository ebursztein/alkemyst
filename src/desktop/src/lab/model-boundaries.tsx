import type { components } from "@alkemyst/sdk";
export function ModelBoundaries({ profile }: { profile: components["schemas"]["Profile"] }) {
  const issues = profile.cards.flatMap((c) =>
    c.issues.map((issue) => ({ ...issue, name: c.name ?? c.reference })),
  );
  return (
    <section className="panel model-boundaries">
      <div className="panel-heading">
        <h2>Model boundaries</h2>
        <span className={`badge ${profile.can_simulate ? "badge-success" : "badge-warning"}`}>
          {profile.can_simulate ? "Ready to simulate" : "Unsupported inputs"}
        </span>
      </div>
      <p>
        No opponent interaction. This model plays a precombat main phase using a named greedy
        policy; it does not find an optimal line.
      </p>
      {profile.assumptions.length > 0 && (
        <details>
          <summary>Policies and assumptions</summary>
          <ul>
            {profile.assumptions.map((a) => (
              <li key={a}>{a}</li>
            ))}
          </ul>
        </details>
      )}
      {issues.length > 0 && (
        <details open={!profile.can_simulate}>
          <summary>
            {issues.length} card interaction {issues.length === 1 ? "note" : "notes"}
          </summary>
          <ul className="issue-list">
            {issues.map((issue, i) => (
              <li key={`${issue.name}:${issue.code}:${i}`}>
                <strong>{issue.name}</strong> ·{" "}
                {issue.blocking ? "Blocks simulation: " : "Excluded or bounded: "}
                {issue.detail}
              </li>
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
