import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { writeSiteManifest } from "../site-build.mjs";

test("deployable sites require an HTTPS root origin and a complete static output", () => {
  const dist = mkdtempSync(join(tmpdir(), "alkemyst site manifest "));
  const options = {
    dist,
    origin: "https://alkemyst.example",
    commit: "a".repeat(40),
    tree: "b".repeat(40),
  };
  try {
    for (const origin of [
      undefined,
      "http://example.com",
      "https://example.com/path",
      "https://user:secret@example.com",
      "https://example.com/?key=x",
      "https://example.com/#fragment",
    ]) {
      assert.throws(() => writeSiteManifest({ ...options, origin }), /origin/i);
    }
    assert.throws(() => writeSiteManifest(options), /static output/i);
    for (const name of [
      "index.html",
      "download/index.html",
      "docs/index.html",
      "pagefind/pagefind.js",
      "404.html",
    ]) {
      const path = join(dist, name);
      mkdirSync(join(path, ".."), { recursive: true });
      writeFileSync(path, "fixture static output");
    }
    assert.throws(() => writeSiteManifest({ ...options, commit: "unknown" }), /commit/i);
    assert.throws(() => writeSiteManifest({ ...options, tree: "unknown" }), /tree/i);
    const manifest = writeSiteManifest(options);
    assert.deepEqual(manifest, {
      origin: "https://alkemyst.example",
      commit: options.commit,
      tree: options.tree,
    });
    assert.deepEqual(JSON.parse(readFileSync(join(dist, "deployment.json"), "utf8")), manifest);
  } finally {
    rmSync(dist, { recursive: true, force: true });
  }
});
