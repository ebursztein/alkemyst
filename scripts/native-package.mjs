import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, lstatSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const desktop = join(root, "src", "desktop");
const tauriCli = join(desktop, "node_modules", "@tauri-apps", "cli", "tauri.js");

function platformSettings(platform) {
  if (platform === "darwin")
    return {
      target: "aarch64-apple-darwin",
      bundles: "app,dmg",
      folder: "dmg",
      suffix: "aarch64.dmg",
    };
  if (platform === "win32")
    return {
      target: "x86_64-pc-windows-msvc",
      bundles: "nsis",
      folder: "nsis",
      suffix: "x64-setup.exe",
    };
  throw new Error(`Unsupported preview platform: ${platform}`);
}

export function buildNativePackage({
  platform = process.platform,
  cli = tauriCli,
  stdio = "inherit",
} = {}) {
  const { target, bundles } = platformSettings(platform);
  const config =
    platform === "darwin"
      ? ["--config", JSON.stringify({ bundle: { macOS: { signingIdentity: "-" } } })]
      : [];
  return execFileSync(
    process.execPath,
    [cli, "build", "--ci", "--target", target, "--bundles", bundles, ...config, "--", "--locked"],
    { cwd: desktop, stdio, encoding: "utf8", env: { ...process.env, CI: "true" } },
  );
}

export function stageNativePackage({
  platform = process.platform,
  bundleDir,
  outputDir,
  version,
  commit,
}) {
  const { target, folder, suffix } = platformSettings(platform);
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version))
    throw new Error("Invalid package version");
  if (!/^[a-f0-9]{40}$/.test(commit)) throw new Error("Invalid source commit");
  const name = `Alkemyst_${version}_${suffix}`;
  const installer = join(bundleDir, folder, name);
  let stat;
  try {
    stat = lstatSync(installer);
  } catch {
    throw new Error(`Missing expected installer: ${name}`);
  }
  if (!stat.isFile() || stat.size === 0)
    throw new Error("Installer is empty or is not a regular file");
  const sha256 = createHash("sha256").update(readFileSync(installer)).digest("hex");
  const manifest = {
    version,
    commit,
    target,
    file: name,
    bytes: stat.size,
    sha256,
    distribution: "preview",
    trusted_signing: false,
  };
  mkdirSync(outputDir, { recursive: true });
  copyFileSync(installer, join(outputDir, name));
  writeFileSync(join(outputDir, "SHA256SUMS"), `${sha256}  ${name}\n`);
  writeFileSync(join(outputDir, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  buildNativePackage();
  const { target } = platformSettings(process.platform);
  const { version } = JSON.parse(
    readFileSync(join(desktop, "src-tauri", "tauri.conf.json"), "utf8"),
  );
  const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  const manifest = stageNativePackage({
    bundleDir: join(root, "target", target, "release", "bundle"),
    outputDir: join(root, "work", "packages", target),
    version,
    commit,
  });
  console.info(`Preview installer verified and staged: ${manifest.file} (${manifest.sha256})`);
}
