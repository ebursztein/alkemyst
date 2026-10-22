import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { reviewSecurityScan } from "./security-policy.mjs";

const version = "2.7.0";
const platform = { darwin: "darwin", linux: "linux", win32: "windows" }[process.platform];
const arch = { arm64: "arm64", x64: "amd64" }[process.arch];
if (!platform || !arch) throw new Error("Unsupported OSV-Scanner platform");
const name = `osv-scanner_${platform}_${arch}${platform === "windows" ? ".exe" : ""}`;
const dir = resolve("work/tools", `osv-${version}`);
const binary = resolve(dir, name);
const base = `https://github.com/google/osv-scanner/releases/download/v${version}`;
async function download(path) {
  const response = await fetch(`${base}/${path}`);
  if (!response.ok) throw new Error(`OSV download failed: ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}
mkdirSync(dir, { recursive: true });
const checksums = await download("osv-scanner_SHA256SUMS");
const expected = checksums
  .toString()
  .split("\n")
  .find((line) => line.trim().endsWith(name))
  ?.split(/\s+/)[0];
if (!expected) throw new Error("OSV checksum is missing");
if (!existsSync(binary)) writeFileSync(binary, await download(name));
const actual = createHash("sha256").update(readFileSync(binary)).digest("hex");
if (actual !== expected) throw new Error("OSV checksum mismatch");
chmodSync(binary, 0o755);
mkdirSync("work/security", { recursive: true });
const policy = JSON.parse(readFileSync("security-exceptions.json", "utf8"));
const graphs = {};
for (const target of policy.supportedTargets) {
  const graph = execFileSync(
    "cargo",
    [
      "tree",
      "--workspace",
      "--all-features",
      "--locked",
      "--target",
      target,
      "--prefix",
      "none",
      "--format",
      "{p}",
    ],
    { encoding: "utf8" },
  );
  graphs[target] = graph;
  writeFileSync(`work/security/${target}.txt`, graph);
}
const locks = ["Cargo.lock", "pnpm-lock.yaml"];
const fingerprint = () =>
  locks.map((path) => createHash("sha256").update(readFileSync(path)).digest("hex")).join(":");
const before = fingerprint();
const review = reviewSecurityScan({
  policy,
  graphs,
  scan(phase, config) {
    const configPath = resolve(`work/security/${phase}.toml`);
    writeFileSync(configPath, config);
    rmSync(`work/security/${phase}.json`, { force: true });
    return spawnSync(
      binary,
      [
        "scan",
        "source",
        "--config",
        configPath,
        "--lockfile",
        "Cargo.lock",
        "--lockfile",
        "pnpm-lock.yaml",
        "--all-packages",
        "--all-vulns",
        "--format",
        "json",
        "--output-file",
        `work/security/${phase}.json`,
      ],
      { stdio: "inherit" },
    );
  },
  readReport: (phase) => JSON.parse(readFileSync(`work/security/${phase}.json`, "utf8")),
});
if (before !== fingerprint()) throw new Error("Lockfiles changed during security review");
writeFileSync("work/security/review.json", JSON.stringify(review.accepted, null, 2) + "\n");
console.log(
  `OSV-Scanner ${version}: both lockfiles passed with ${review.accepted.length} exact, expiring advisory exceptions. See work/security/review.json.`,
);
