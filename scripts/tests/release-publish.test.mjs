import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { publishPreviewDraft } from "../release-publish.mjs";

function fixture(run) {
  const directory = mkdtempSync(join(tmpdir(), "alkemyst draft release "));
  const release = {
    tag: "v0.1.0-preview.1",
    version: "0.1.0-preview.1",
    commit: "a".repeat(40),
    platforms: [
      { target: "aarch64-apple-darwin", file: "Alkemyst_0.1.0-preview.1_aarch64.dmg" },
      { target: "x86_64-pc-windows-msvc", file: "Alkemyst_0.1.0-preview.1_x64-setup.exe" },
    ],
  };
  writeFileSync(
    join(directory, "RELEASE_NOTES.md"),
    `Preview\n\nSource commit: ${release.commit}\n`,
  );
  const calls = [];
  let existing;
  let readError;
  let failUpload = false;
  const callGh = (args) => {
    calls.push(args);
    if (args[1] === "view") {
      if (readError) throw readError;
      if (existing) return JSON.stringify(existing);
      throw Object.assign(new Error("missing"), { status: 1, stderr: "release not found\n" });
    }
    if (args[1] === "create")
      existing = { isDraft: true, tagName: release.tag, body: `Source commit: ${release.commit}` };
    if (args[1] === "upload" && failUpload) throw new Error("Upload interrupted");
    return "";
  };
  try {
    run({
      directory,
      release,
      calls,
      callGh,
      setExisting: (value) => {
        existing = value;
      },
      setReadError: (value) => {
        readError = value;
      },
      failUpload: (value) => {
        failUpload = value;
      },
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("new previews stay draft/prerelease and an interrupted upload can resume", () =>
  fixture((context) => {
    context.failUpload(true);
    assert.throws(() => publishPreviewDraft(context), /Upload interrupted/);
    const create = context.calls.find((args) => args[1] === "create");
    assert.ok(create.includes("--draft"));
    assert.ok(create.includes("--prerelease"));
    assert.ok(create.includes("--verify-tag"));
    assert.ok(create.includes("--notes-file"));
    context.failUpload(false);
    publishPreviewDraft(context);
    assert.equal(context.calls.filter((args) => args[1] === "create").length, 1);
    const upload = context.calls.filter((args) => args[1] === "upload").at(-1);
    assert.ok(upload.includes("--clobber"));
    assert.ok(upload.includes(join(context.directory, "SHA256SUMS")));
  }));

test("published releases and unrelated drafts are never modified", () => {
  for (const existing of [
    { isDraft: false, tagName: "v0.1.0-preview.1", body: "" },
    { isDraft: true, tagName: "v0.1.0-preview.1", body: `Source commit: ${"b".repeat(40)}` },
  ])
    fixture((context) => {
      context.setExisting(existing);
      assert.throws(() => publishPreviewDraft(context), /published|source/i);
      assert.equal(context.calls.filter((args) => args[1] !== "view").length, 0);
    });
});

test("authentication/transient read errors cannot be mistaken for a missing release", () =>
  fixture((context) => {
    context.setReadError(
      Object.assign(new Error("HTTP 401"), { status: 1, stderr: "Authentication failed" }),
    );
    assert.throws(() => publishPreviewDraft(context), /HTTP 401/);
    assert.equal(context.calls.length, 1);
  }));
