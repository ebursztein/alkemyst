import { createHash } from "node:crypto";
import { copyFileSync, lstatSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { previewVersion } from "./release-source.mjs";

const platforms = [
  { target: "aarch64-apple-darwin", suffix: "aarch64.dmg" },
  { target: "x86_64-pc-windows-msvc", suffix: "x64-setup.exe" },
];

export function assembleReleaseAssets({ source, artifacts, output }) {
  if (previewVersion(source.tag) !== source.version || !/^[a-f0-9]{40}$/.test(source.commit)) {
    throw new Error("Invalid release source manifest");
  }
  // Validate both platforms completely before creating any release output.
  const verified = platforms.map(({ target, suffix }) => {
    const dir = join(artifacts, `preview-${target}`);
    let manifest;
    try {
      manifest = JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8"));
    } catch {
      throw new Error(`Missing or invalid artifact manifest for ${target}`);
    }
    const file = `Alkemyst_${source.version}_${suffix}`;
    if (
      manifest.target !== target ||
      manifest.file !== file ||
      manifest.version !== source.version ||
      manifest.commit !== source.commit ||
      manifest.distribution !== "preview"
    ) {
      throw new Error(`Artifact manifest differs from the release source: ${target}`);
    }
    if (manifest.trusted_signing !== false)
      throw new Error("Preview artifact must not claim trusted signing");
    const path = join(dir, file);
    let stat;
    try {
      stat = lstatSync(path);
    } catch {
      throw new Error(`Missing package artifact: ${file}`);
    }
    if (!stat.isFile() || stat.size === 0 || stat.size !== manifest.bytes)
      throw new Error(`Invalid package artifact size/type: ${file}`);
    const sha256 = createHash("sha256").update(readFileSync(path)).digest("hex");
    if (sha256 !== manifest.sha256) throw new Error(`Package artifact checksum differs: ${file}`);
    return { path, metadata: { target, file, bytes: stat.size, sha256 } };
  });
  mkdirSync(dirname(output), { recursive: true });
  if (lstatExists(output))
    throw new Error("Release output must be a new directory; stale artifacts are not reused");
  mkdirSync(output);
  const release = {
    ...source,
    distribution: "preview",
    trusted_signing: false,
    platforms: verified.map(({ metadata }) => metadata),
  };
  for (const { path, metadata } of verified) copyFileSync(path, join(output, metadata.file));
  writeFileSync(
    join(output, "SHA256SUMS"),
    verified.map(({ metadata: m }) => `${m.sha256}  ${m.file}\n`).join(""),
  );
  writeFileSync(join(output, "manifest.json"), `${JSON.stringify(release, null, 2)}\n`);
  writeFileSync(
    join(output, "RELEASE_NOTES.md"),
    [
      `# Alkemyst ${source.version}`,
      "",
      "Local Arena tracking, immutable deck history, seeded analysis, manual practice and authenticated MCP.",
      "",
      "Preview distribution: macOS Apple Silicon/macOS 13+ uses ad-hoc signing without notarization; Windows 11 x64 is unsigned. Stable distribution requires trusted signing and notarization.",
      "",
      `Download the installer for your platform and verify it against SHA256SUMS. See the [desktop installation guide](https://github.com/ebursztein/alkemyst/blob/${source.commit}/src/desktop/INSTALLERS.md) for platform instructions and limitations.`,
      "",
      `Source commit: ${source.commit}`,
      `Preview tag: ${source.tag}`,
      "",
    ].join("\n"),
  );
  return release;
}

function lstatExists(path) {
  try {
    lstatSync(path);
    return true;
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}
