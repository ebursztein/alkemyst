import { execFileSync } from "node:child_process";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { previewVersion } from "./release-source.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const gh = (args) =>
  execFileSync("gh", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });

export function publishPreviewDraft({ release, directory, callGh = gh }) {
  if (previewVersion(release.tag) !== release.version || !/^[a-f0-9]{40}$/.test(release.commit))
    throw new Error("Invalid preview release source");
  let existing;
  try {
    existing = JSON.parse(
      callGh(["release", "view", release.tag, "--json", "isDraft,body,tagName"]),
    );
  } catch (error) {
    // An authentication, transport or malformed-output failure is never absence.
    if (error.status !== 1 || String(error.stderr).trim() !== "release not found") throw error;
  }
  if (existing) {
    if (existing.isDraft !== true)
      throw new Error("A published release cannot be modified; create a new preview version");
    if (
      existing.tagName !== release.tag ||
      !String(existing.body).split(/\r?\n/).includes(`Source commit: ${release.commit}`)
    )
      throw new Error("Existing draft release belongs to different source");
  } else {
    callGh([
      "release",
      "create",
      release.tag,
      "--verify-tag",
      "--target",
      release.commit,
      "--draft",
      "--prerelease",
      "--title",
      `Alkemyst ${release.version}`,
      "--notes-file",
      join(directory, "RELEASE_NOTES.md"),
    ]);
  }
  const files = [
    `Alkemyst_${release.version}_aarch64.dmg`,
    `Alkemyst_${release.version}_x64-setup.exe`,
    "manifest.json",
    "SHA256SUMS",
  ].map((name) => join(directory, name));
  callGh(["release", "upload", release.tag, ...files, "--clobber"]);
  console.info(
    `Draft preview ready: ${release.tag} (${release.commit}). Publication remains an explicit release step.`,
  );
}
