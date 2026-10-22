import { test } from "node:test";
import assert from "node:assert/strict";
import { assessSecurity, requireScannerSuccess } from "../security-policy.mjs";
const targets = ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"];
const graphs = Object.fromEntries(targets.map((t) => [t, "alkemyst-desktop v0.1.0\n"]));
const exception = {
  package: "glib",
  version: "0.18.5",
  advisories: ["RUSTSEC-2024-0429"],
  expires: "2026-11-05",
  owner: "ebursztein",
  reason: "Linux-only GTK dependency, absent from both supported desktop targets.",
  remediation: "Replace when Tauri supports the patched GTK dependency family.",
};
function report(packageOverride = {}, id = "RUSTSEC-2024-0429") {
  return {
    results: [
      {
        source: { type: "lockfile", path: "/repo/Cargo.lock" },
        packages: [
          {
            package: {
              name: "glib",
              version: "0.18.5",
              ecosystem: "crates.io",
              ...packageOverride,
            },
            vulnerabilities: [{ id }],
          },
        ],
      },
      {
        source: { type: "lockfile", path: "/repo/pnpm-lock.yaml" },
        packages: [
          {
            package: { name: "react", version: "19.3.1", ecosystem: "npm" },
          },
        ],
      },
    ],
  };
}
const assess = (
  value = report(),
  entries = [exception],
  tree = graphs,
  now = new Date("2026-10-22"),
) => assessSecurity(value, { supportedTargets: targets, exceptions: entries }, tree, now);

test("exceptions require exact advisory, ecosystem, package and version", () => {
  assert.equal(assess().accepted.length, 1);
  for (const change of [{ name: "other" }, { version: "0.18.6" }, { ecosystem: "npm" }])
    assert.throws(() => assess(report(change)), /Unaccepted/);
  assert.throws(() => assess(report({}, "RUSTSEC-2099-9999")), /Unaccepted/);
  assert.throws(() => assess(report(), []), /Unaccepted/);
});
test("expired, malformed and overly long exceptions fail before scanning", () => {
  for (const change of [
    { expires: "2026-10-22" },
    { expires: "bad-date" },
    { expires: "2026-11-31" },
    { expires: "2027-01-01" },
    { owner: "" },
    { reason: "" },
    { version: "*" },
  ])
    assert.throws(() => assess(report(), [{ ...exception, ...change }]));
});
test("an exception is rejected if its crate becomes reachable in any supported target", () => {
  assert.throws(
    () => assess(report(), [exception], { ...graphs, [targets[1]]: "glib v0.18.5\n" }),
    /reachable/,
  );
  assert.throws(() => assess(report(), [exception], {}), /target/);
});
test("partial, empty, malformed, duplicate or unexpected scan sources fail", () => {
  for (const value of [
    {},
    { results: [] },
    { results: report().results.slice(0, 1) },
    { results: [report().results[0], report().results[0]] },
  ])
    assert.throws(() => assess(value), /report|source/);
  const value = report();
  value.results[1].packages = [];
  assert.throws(() => assess(value), /package/);
});
test("scanner errors cannot be accepted by policy, including a findings exit status", () => {
  requireScannerSuccess({ status: 0, signal: null });
  for (const status of [1, 127, 128, 129, 130, null])
    assert.throws(() => requireScannerSuccess({ status, signal: null }), /Scanner/);
  assert.throws(() => requireScannerSuccess({ status: 0, signal: "SIGTERM" }), /Scanner/);
});
test("configuration names only accepted advisory IDs and expires in UTC", () => {
  const result = assess();
  assert.match(result.config, /id = "RUSTSEC-2024-0429"/);
  assert.match(result.config, /ignoreUntil = 2026-11-05/);
  assert.throws(() => assess(report(), [exception], graphs, new Date("2026-11-05T00:00:00Z")));
});

test("the gate requires a second native scanner success after exact preflight review", async () => {
  const { reviewSecurityScan } = await import("../security-policy.mjs");
  const calls = [];
  const run = (status) =>
    reviewSecurityScan({
      scan: (phase, config) => {
        calls.push({ phase, config });
        return { status: phase === "raw" ? 1 : status };
      },
      readReport: () => report(),
      policy: { supportedTargets: targets, exceptions: [exception] },
      graphs,
      now: new Date("2026-10-22"),
    });
  assert.equal(run(0).accepted.length, 1);
  assert.equal(calls[0].config, "");
  assert.match(calls[1].config, /RUSTSEC-2024-0429/);
  for (const status of [1, 127, 129]) assert.throws(() => run(status), /Scanner/);
});

test("OSV alias groups generate one ignore while every advisory is reviewed", () => {
  const value = report();
  const alias = "GHSA-wrw7-89jp-8q8g";
  const item = value.results[0].packages[0];
  item.vulnerabilities.push({ id: alias });
  item.groups = [{ ids: [exception.advisories[0], alias] }];
  const result = assess(value, [{ ...exception, advisories: [...exception.advisories, alias] }]);
  assert.equal(result.accepted.length, 2);
  assert.equal(result.config.match(/\[\[IgnoredVulns\]\]/g).length, 1);
});
