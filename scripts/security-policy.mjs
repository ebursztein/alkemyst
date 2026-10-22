import { basename } from "node:path";
const supported = ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"];
const advisoryId = /^(RUSTSEC-\d{4}-\d{4}|GHSA-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4})$/;

export function requireScannerSuccess(result) {
  if (result.status !== 0 || result.signal || result.error)
    throw new Error(`Scanner did not complete successfully (status ${result.status}).`);
}

function validatePolicy(policy, graphs, now) {
  if (JSON.stringify(policy.supportedTargets) !== JSON.stringify(supported))
    throw new Error("Security target policy must match the two supported desktop targets.");
  if (!Array.isArray(policy.exceptions)) throw new Error("Invalid exceptions policy");
  const seen = new Set();
  for (const entry of policy.exceptions) {
    if (
      !/^[a-z][a-z0-9_-]+$/.test(entry.package) ||
      !/^\d+\.\d+\.\d+$/.test(entry.version) ||
      !Array.isArray(entry.advisories) ||
      entry.advisories.length === 0 ||
      !entry.advisories.every((id) => advisoryId.test(id)) ||
      ![entry.owner, entry.reason, entry.remediation].every(
        (value) => typeof value === "string" && value.trim(),
      )
    )
      throw new Error("Invalid narrowly scoped security exception");
    const expiry = new Date(`${entry.expires}T00:00:00Z`);
    if (
      !/^\d{4}-\d{2}-\d{2}$/.test(entry.expires) ||
      !Number.isFinite(expiry.valueOf()) ||
      expiry.toISOString().slice(0, 10) !== entry.expires ||
      expiry <= now ||
      expiry - now > 30 * 86400000
    )
      throw new Error(`Invalid, expired or over-30-day exception for ${entry.package}`);
    const key = `${entry.package}@${entry.version}`;
    if (seen.has(key)) throw new Error(`Duplicate exception for ${key}`);
    seen.add(key);
    for (const target of supported) {
      if (typeof graphs[target] !== "string" || !graphs[target].trim())
        throw new Error(`Missing target dependency graph: ${target}`);
      const reachable = graphs[target].split("\n").some((line) => {
        const match = /^(\S+) v(\S+)/.exec(line.trim());
        return match?.[1] === entry.package && match[2] === entry.version;
      });
      if (reachable) throw new Error(`Exception ${key} is reachable in supported target ${target}`);
    }
  }
}

export function assessSecurity(report, policy, graphs, now = new Date()) {
  validatePolicy(policy, graphs, now);
  if (!Array.isArray(report?.results) || report.results.length !== 2)
    throw new Error("Incomplete scanner report: both lockfile sources are required");
  const sources = new Set();
  const accepted = [];
  const aliasDuplicates = new Set();
  for (const source of report.results) {
    const name = basename(source.source?.path?.replaceAll("\\", "/") ?? "");
    if (
      source.source?.type !== "lockfile" ||
      !["Cargo.lock", "pnpm-lock.yaml"].includes(name) ||
      sources.has(name)
    )
      throw new Error("Unexpected or duplicate scanner source");
    sources.add(name);
    if (!Array.isArray(source.packages) || source.packages.length === 0)
      throw new Error(`Incomplete package inventory for ${name}`);
    for (const item of source.packages) {
      if (
        ![item.package?.name, item.package?.version, item.package?.ecosystem].every(
          (value) => typeof value === "string" && value,
        )
      )
        throw new Error("Malformed scanner package");
      if (item.vulnerabilities !== undefined && !Array.isArray(item.vulnerabilities))
        throw new Error("Malformed vulnerabilities report");
      for (const vulnerability of item.vulnerabilities ?? []) {
        const entry = policy.exceptions.find(
          (candidate) =>
            name === "Cargo.lock" &&
            item.package.ecosystem === "crates.io" &&
            candidate.package === item.package.name &&
            candidate.version === item.package.version &&
            candidate.advisories.includes(vulnerability.id),
        );
        if (!entry)
          throw new Error(
            `Unaccepted finding: ${item.package.name}@${item.package.version} ${vulnerability.id}`,
          );
        accepted.push({
          package: entry.package,
          version: entry.version,
          advisory: vulnerability.id,
          expires: entry.expires,
        });
      }
      for (const group of item.groups ?? []) {
        if (
          Array.isArray(group.ids) &&
          group.ids.every((id) =>
            accepted.some(
              (finding) =>
                finding.package === item.package.name &&
                finding.version === item.package.version &&
                finding.advisory === id,
            ),
          )
        ) {
          for (const id of [...group.ids].sort().slice(1)) aliasDuplicates.add(id);
        }
      }
    }
  }
  const config = [...new Set(accepted.map((finding) => finding.advisory))]
    .filter((id) => !aliasDuplicates.has(id))
    .map((id) => {
      const entry = policy.exceptions.find((candidate) => candidate.advisories.includes(id));
      return `[[IgnoredVulns]]\nid = "${id}"\nignoreUntil = ${entry.expires}\nreason = "Exact package and target review enforced by Alkemyst; see SECURITY.md."\n`;
    })
    .join("\n");
  return { accepted, config };
}

export function reviewSecurityScan({ scan, readReport, policy, graphs, now = new Date() }) {
  validatePolicy(policy, graphs, now);
  const raw = scan("raw", "");
  // Exit 1 supplies findings for review only; it never makes the gate pass.
  if (raw.status !== 1 || raw.signal || raw.error) requireScannerSuccess(raw);
  const review = assessSecurity(readReport("raw"), policy, graphs, now);
  const checked = scan("reviewed", review.config);
  requireScannerSuccess(checked);
  assessSecurity(readReport("reviewed"), policy, graphs, now);
  return review;
}
