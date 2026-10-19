import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

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
execFileSync(
  binary,
  [
    "scan",
    "source",
    "--lockfile",
    "Cargo.lock",
    "--lockfile",
    "pnpm-lock.yaml",
    "--format",
    "json",
    "--output-file",
    "work/security/osv.json",
  ],
  { stdio: "inherit" },
);
console.log(`OSV-Scanner ${version}: both lockfiles passed.`);
