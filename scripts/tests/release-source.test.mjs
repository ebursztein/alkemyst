import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { verifyReleaseSource, workspaceReleaseVersion } from "../release-source.mjs";

function fixture(run) {
  const root = mkdtempSync(join(tmpdir(), "alkemyst release source "));
  const version = "0.1.0-preview.1";
  const tag = `v${version}`;
  const git = (...args) =>
    execFileSync("git", args, {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  const write = (path, content) => {
    mkdirSync(join(root, path, ".."), { recursive: true });
    writeFileSync(join(root, path), content);
  };
  const commit = () => {
    git("add", ".");
    git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "-c",
      "commit.gpgsign=false",
      "commit",
      "-qm",
      "candidate",
    );
  };
  git("init", "--initial-branch=main");
  write(".gitignore", "work/\n");
  write("Cargo.toml", `[workspace.package]\nversion = "${version}"\n`);
  for (const path of [
    "package.json",
    "src/desktop/package.json",
    "src/sdk/package.json",
    "src/site/package.json",
    "src/desktop/src-tauri/tauri.conf.json",
  ])
    write(path, JSON.stringify({ version }));
  commit();
  git("update-ref", "refs/remotes/origin/main", "HEAD");
  git("tag", tag);
  try {
    run({ root, tag, version, git, write, commit });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test("workspace version consistency can be checked before any release tag exists", () =>
  fixture(({ root, tag, version, git, write }) => {
    git("tag", "-d", tag);
    assert.equal(workspaceReleaseVersion(root), version);
    write("src/sdk/package.json", JSON.stringify({ version: "0.1.0-preview.2" }));
    assert.throws(() => workspaceReleaseVersion(root), /version/i);
  }));

test("only a clean version-matched preview tag on protected main can become a release", () =>
  fixture(({ root, tag, version, git, write }) => {
    const source = verifyReleaseSource({ root, tag });
    assert.deepEqual(source, { tag, version, commit: git("rev-parse", "HEAD") });
    for (const invalid of [
      "v0.1.0",
      "v0.1.0-beta.1",
      "v0.1.0-preview.0",
      "v01.1.0-preview.1",
      "../escape",
      undefined,
    ])
      assert.throws(() => verifyReleaseSource({ root, tag: invalid }), /preview tag/i);
    write("uncommitted.txt", "uncommitted source");
    assert.throws(() => verifyReleaseSource({ root, tag }), /clean/i);
  }));

test("component-version drift, moved source and unmerged commits are rejected", () =>
  fixture(({ root, tag, git, write, commit }) => {
    write("src/sdk/package.json", JSON.stringify({ version: "0.1.0" }));
    commit();
    git("tag", "-f", tag);
    git("update-ref", "refs/remotes/origin/main", "HEAD");
    assert.throws(() => verifyReleaseSource({ root, tag }), /version/i);
  }));

test("tagged and main-reachable commits must match the checked-out candidate", () =>
  fixture(({ root, tag, git, write, commit }) => {
    write("change.txt", "new commit");
    commit();
    assert.throws(() => verifyReleaseSource({ root, tag }), /tag.*source/i);
    git("tag", "-f", tag);
    assert.throws(() => verifyReleaseSource({ root, tag }), /main/i);
  }));
