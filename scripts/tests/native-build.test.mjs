import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import assert from "node:assert/strict";
import { runNativeBuild } from "../native-build.mjs";

test("native builds preserve Cargo's argument separator through the Node CLI", () => {
  const dir = mkdtempSync(join(tmpdir(), "alkemyst native arguments "));
  try {
    const cli = join(dir, "fake tauri.mjs");
    writeFileSync(
      cli,
      "console.log(JSON.stringify({args:process.argv.slice(2),cwd:process.cwd()}));\n",
    );
    const result = JSON.parse(runNativeBuild({ cli, stdio: "pipe" }));
    assert.equal(result.args[result.args.indexOf("--locked") - 1], "--");
    assert.ok(result.args.includes("--ci"));
    assert.ok(result.args.includes("--no-bundle"));
    assert.match(result.cwd.replaceAll("\\", "/"), /\/src\/desktop$/);
    writeFileSync(cli, "process.exit(23);\n");
    assert.throws(
      () => runNativeBuild({ cli, stdio: "pipe" }),
      (error) => error.status === 23,
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
