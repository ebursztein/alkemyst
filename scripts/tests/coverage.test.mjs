import assert from "node:assert/strict";
import test from "node:test";
import { checkRustCoverage } from "../coverage-rules.mjs";

const file = (area, covered, count) => ({
  filename: `/repo/src/core/src/${area}/example.rs`,
  summary: { lines: { covered, count } },
});

test("SQL coverage cannot mask a domain regression", () => {
  assert.match(
    checkRustCoverage([file("db", 200, 200), file("domain", 1, 10)]).join("\n"),
    /Domain/,
  );
});

test("domain coverage cannot mask an engine regression", () => {
  assert.match(
    checkRustCoverage([file("db", 1, 10), file("analysis", 200, 200)]).join("\n"),
    /SQL engine/,
  );
});

test("independent thresholds preserve the overall owned-code minimum", () => {
  assert.deepEqual(checkRustCoverage([file("db", 90, 100), file("domain", 90, 100)]), []);
  assert.match(checkRustCoverage([file("catalog", 80, 100)]).join("\n"), /Overall/);
  const windows = file("practice", 1, 10);
  windows.filename = windows.filename.replaceAll("/", "\\");
  assert.match(checkRustCoverage([windows]).join("\n"), /Domain/);
});

test("maintained executable Rust files cannot disappear from coverage", () => {
  assert.match(
    checkRustCoverage([], ["/repo/src/core/src/unreferenced.rs"]).join("\n"),
    /missing from coverage/,
  );
});
