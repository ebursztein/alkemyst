import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { buildNativePackage, stageNativePackage } from "../native-package.mjs";

test("preview packaging preserves release Cargo flags and disables Finder automation", () => {
  const dir = mkdtempSync(join(tmpdir(), "alkemyst package arguments "));
  try {
    const cli = join(dir, "tauri fixture.mjs");
    writeFileSync(
      cli,
      "console.log(JSON.stringify({args:process.argv.slice(2),ci:process.env.CI}));\n",
    );
    const result = JSON.parse(buildNativePackage({ platform: "darwin", cli, stdio: "pipe" }));
    assert.equal(result.ci, "true");
    assert.equal(result.args[result.args.indexOf("--locked") - 1], "--");
    assert.ok(result.args.includes("aarch64-apple-darwin"));
    assert.ok(result.args.includes("app,dmg"));
    assert.ok(!result.args.includes("--debug"));
    assert.ok(!result.args.includes("--no-bundle"));
    const windows = JSON.parse(buildNativePackage({ platform: "win32", cli, stdio: "pipe" }));
    assert.ok(windows.args.includes("x86_64-pc-windows-msvc"));
    assert.ok(windows.args.includes("nsis"));
    writeFileSync(cli, "process.exit(23);\n");
    assert.throws(
      () => buildNativePackage({ platform: "win32", cli, stdio: "pipe" }),
      (error) => error.status === 23,
    );
    assert.throws(() => buildNativePackage({ platform: "linux", cli }), /Unsupported/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("only the expected nonempty installer is staged with content-derived checksums", () => {
  const dir = mkdtempSync(join(tmpdir(), "alkemyst packages "));
  try {
    const bundleDir = join(dir, "bundle");
    const outputDir = join(dir, "artifacts");
    mkdirSync(join(bundleDir, "dmg"), { recursive: true });
    const options = {
      platform: "darwin",
      bundleDir,
      outputDir,
      version: "0.1.0",
      commit: "a".repeat(40),
    };
    assert.throws(() => stageNativePackage(options), /expected installer/i);
    writeFileSync(join(bundleDir, "dmg", "Alkemyst_0.0.9_aarch64.dmg"), "stale");
    assert.throws(() => stageNativePackage(options), /expected installer/i);
    const name = "Alkemyst_0.1.0_aarch64.dmg";
    writeFileSync(join(bundleDir, "dmg", name), "");
    assert.throws(() => stageNativePackage(options), /empty/i);
    const payload = Buffer.from("fixture package contents");
    writeFileSync(join(bundleDir, "dmg", name), payload);
    const manifest = stageNativePackage(options);
    const hash = createHash("sha256").update(payload).digest("hex");
    assert.equal(manifest.sha256, hash);
    assert.equal(manifest.target, "aarch64-apple-darwin");
    assert.equal(manifest.commit, options.commit);
    assert.deepEqual(readFileSync(join(outputDir, name)), payload);
    assert.equal(readFileSync(join(outputDir, "SHA256SUMS"), "utf8"), `${hash}  ${name}\n`);
    assert.deepEqual(JSON.parse(readFileSync(join(outputDir, "manifest.json"), "utf8")), manifest);
    assert.throws(() => stageNativePackage({ ...options, version: "../escape" }), /version/i);
    assert.throws(() => stageNativePackage({ ...options, commit: "unknown" }), /commit/i);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
