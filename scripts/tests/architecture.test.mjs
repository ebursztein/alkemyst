import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import assert from "node:assert/strict";
import { checkArchitecture } from "../architecture-rules.mjs";

function fixture(run) {
  const dir = mkdtempSync(join(tmpdir(), "alkemyst-architecture-"));
  function write(name, content) {
    const path = join(dir, name);
    mkdirSync(join(path, ".."), { recursive: true });
    writeFileSync(path, content);
  }
  write(
    "architecture.json",
    JSON.stringify({ maxLines: 300, exemptions: [], packages: { desktop: ["sdk"], sdk: [] } }),
  );
  try {
    run(dir, write);
  } finally {
    rmSync(dir, { recursive: true });
  }
}

test("rejects large source and tests, but allows a separate test", () =>
  fixture((dir, write) => {
    write("src/sdk/src/index.ts", "export const x = 1;\n");
    write("src/sdk/tests/client.test.ts", 'test("works", () => {});\n');
    assert.deepEqual(checkArchitecture(dir), []);
    write("src/sdk/tests/large.test.ts", "// line\n".repeat(301));
    assert.match(checkArchitecture(dir).join("\n"), /301 lines exceeds 300/);
  }));

test("rejects embedded tests and private cross-package imports", () =>
  fixture((dir, write) => {
    write(
      "src/desktop/src/app.ts",
      'import { x } from "../../sdk/src/index.js";\ntest("bad", () => x);\n',
    );
    const errors = checkArchitecture(dir).join("\n");
    assert.match(errors, /tests must live in separate files/);
    assert.match(errors, /public sdk package export/);
  }));

test("rejects package cycles", () =>
  fixture((dir, write) => {
    write("src/desktop/src/app.ts", 'import "@alkemyst/sdk";\n');
    write("src/sdk/src/index.ts", 'import "@alkemyst/desktop";\n');
    assert.match(checkArchitecture(dir).join("\n"), /Circular package dependency/);
  }));

test("rejects inline Rust tests and misplaced SQL access", () =>
  fixture((dir, write) => {
    write("src/core/src/api.rs", "#[test]\nfn example() {}\nuse rusqlite::Connection;\n");
    write("src/core/src/query.sql", "SELECT 1;");
    const errors = checkArchitecture(dir).join("\n");
    assert.match(errors, /Rust tests must live in separate files/);
    assert.match(errors, /raw database access/);
    assert.match(errors, /SQL must live in statement files/);
  }));

test("rejects SQL hidden in an API module", () =>
  fixture((dir, write) => {
    write("src/core/src/api.rs", 'let query = "SELECT name FROM decks";');
    assert.match(checkArchitecture(dir).join("\n"), /SQL literals belong in statement modules/);
  }));

test("Wrangler runtime artifacts are excluded while maintained tooling stays checked", () =>
  fixture((dir, write) => {
    write(".wrangler/tmp/worker.js", "// generated\n".repeat(301));
    write("scripts/owned.mjs", "// maintained\n".repeat(301));
    assert.deepEqual(checkArchitecture(dir), ["scripts/owned.mjs: 301 lines exceeds 300"]);
  }));
