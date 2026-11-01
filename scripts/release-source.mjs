import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { parse } from "smol-toml";

export function previewVersion(tag) {
  if (!/^v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)-preview\.[1-9]\d*$/.test(tag)) {
    throw new Error("Only a vMAJOR.MINOR.PATCH-preview.N preview tag is accepted");
  }
  return tag.slice(1);
}

export function workspaceReleaseVersion(root) {
  const version = JSON.parse(readFileSync(join(root, "package.json"), "utf8")).version;
  previewVersion(`v${version}`);
  for (const path of [
    "package.json",
    "src/desktop/package.json",
    "src/sdk/package.json",
    "src/site/package.json",
    "src/desktop/src-tauri/tauri.conf.json",
  ]) {
    if (JSON.parse(readFileSync(join(root, path), "utf8")).version !== version)
      throw new Error(`Release version differs in ${path}`);
  }
  if (
    parse(readFileSync(join(root, "Cargo.toml"), "utf8")).workspace?.package?.version !== version
  ) {
    throw new Error("Cargo workspace version differs from the preview tag");
  }
  return version;
}

export function verifyReleaseSource({ root, tag }) {
  const version = previewVersion(tag);
  const git = (...args) =>
    execFileSync("git", args, {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  if (git("status", "--porcelain")) throw new Error("Release source must be clean and committed");
  const commit = git("rev-parse", "HEAD");
  let tagged;
  try {
    tagged = git("rev-parse", "--verify", `refs/tags/${tag}^{commit}`);
  } catch {
    throw new Error("Release tag must identify the checked-out source commit");
  }
  if (tagged !== commit) throw new Error("Release tag differs from the checked-out source commit");
  try {
    git("merge-base", "--is-ancestor", commit, "refs/remotes/origin/main");
  } catch {
    throw new Error("Release source must be reachable from origin/main");
  }
  if (workspaceReleaseVersion(root) !== version)
    throw new Error("Workspace version differs from the preview tag");
  return { tag, version, commit };
}
