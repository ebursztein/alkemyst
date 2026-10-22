import { test } from "node:test";
import assert from "node:assert/strict";
import { checkNpmDependencies, checkCargoDependencies } from "../dependency-pins.mjs";

test("npm dependency versions must be exact while workspace references stay local", () => {
  assert.deepEqual(
    checkNpmDependencies({ dependencies: { react: "19.3.1", sdk: "workspace:*" } }),
    [],
  );
  for (const range of [
    "^19.3.1",
    "~19.3.1",
    "latest",
    "*",
    ">=19",
    "git+https://example.com/repo",
  ]) {
    assert.match(
      checkNpmDependencies({ devDependencies: { react: range } }).join("\n"),
      /react.*exact/,
    );
  }
});

test("Cargo pin checks cover string, table, workspace and target dependencies", () => {
  assert.deepEqual(
    checkCargoDependencies({
      dependencies: {
        serde: "=1.0.229",
        core: { path: "../core" },
        tokio: { workspace: true },
        axum: { version: "=0.8.9", features: ["json"] },
      },
    }),
    [],
  );
  for (const manifest of [
    { dependencies: { serde: "1.0.229" } },
    { "build-dependencies": { serde: { version: "^1.0.229" } } },
    { workspace: { dependencies: { serde: "*" } } },
    {
      target: {
        "cfg(windows)": {
          "dev-dependencies": { serde: { git: "https://example.com/serde", branch: "main" } },
        },
      },
    },
  ])
    assert.match(checkCargoDependencies(manifest).join("\n"), /serde.*exact|serde.*revision/);
  assert.deepEqual(
    checkCargoDependencies({
      dependencies: { fork: { git: "https://example.com/fork", rev: "a".repeat(40) } },
    }),
    [],
  );
});

test("the repository scan parses nested manifests and honors only explicit exemptions", async () => {
  const { mkdtempSync, mkdirSync, writeFileSync, rmSync } = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const { join } = await import("node:path");
  const { checkDependencyPins } = await import("../dependency-pins.mjs");
  const root = mkdtempSync(join(tmpdir(), "alkemyst-pins-"));
  try {
    mkdirSync(join(root, "src/core"), { recursive: true });
    writeFileSync(join(root, "architecture.json"), JSON.stringify({ exemptions: [] }));
    writeFileSync(
      join(root, "package.json"),
      JSON.stringify({ devDependencies: { eslint: "10.13.0" } }),
    );
    const manifest = join(root, "src/core/Cargo.toml");
    writeFileSync(
      manifest,
      "[target.'cfg(windows)'.dependencies]\nserde = {version=\"=1.0.229\"}\n",
    );
    assert.deepEqual(checkDependencyPins(root), []);
    writeFileSync(
      manifest,
      "[target.'cfg(windows)'.dependencies]\nserde = {version=\"^1.0.229\"}\n",
    );
    assert.match(checkDependencyPins(root).join("\n"), /src\/core\/Cargo.toml.*serde.*exact/);
    writeFileSync(
      join(root, "architecture.json"),
      JSON.stringify({ exemptions: ["src/core/Cargo.toml"] }),
    );
    assert.deepEqual(checkDependencyPins(root), []);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
