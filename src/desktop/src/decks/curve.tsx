import type { components } from "@alkemyst/sdk";
export function ManaCurve({ bins }: { bins: components["schemas"]["CurveBin"][] }) {
  const maximum = Math.max(1, ...bins.map((b) => b.quantity));
  return (
    <figure className="panel curve" aria-label="Printed mana curve">
      <figcaption>
        <h2>Mana curve</h2>
        <p>Printed mana value · nonland cards</p>
      </figcaption>
      <div className="curve-bars" aria-hidden="true">
        {bins.map((bin) => (
          <div className="curve-column" key={bin.mana_value ?? "unknown"}>
            <span className="curve-number">{bin.quantity}</span>
            <div
              className="curve-bar"
              style={{ height: `${Math.max(2, (bin.quantity / maximum) * 130)}px` }}
            />
            <span>{bin.mana_value ?? "?"}</span>
          </div>
        ))}
      </div>
      <table className="visually-hidden">
        <caption>Printed mana curve values</caption>
        <thead>
          <tr>
            <th>Mana value</th>
            <th>Cards</th>
          </tr>
        </thead>
        <tbody>
          {bins.map((b) => (
            <tr key={b.mana_value ?? "unknown"}>
              <th>{b.mana_value ?? "Unknown"}</th>
              <td>{b.quantity}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {bins.length === 0 && <p>No nonland cards.</p>}
    </figure>
  );
}
