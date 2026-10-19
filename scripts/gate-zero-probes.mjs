import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";

mkdirSync("work/gate-zero", { recursive: true });
const results = [];
function probe(name, path, contents, command, args) {
  let old;
  try {
    old = readFileSync(path);
  } catch {
    old = undefined;
  }
  try {
    writeFileSync(path, contents);
    const result = spawnSync(command, args, {
      encoding: "utf8",
      shell: process.platform === "win32",
    });
    writeFileSync(`work/gate-zero/${name}.log`, `${result.stdout ?? ""}\n${result.stderr ?? ""}`);
    if (result.error || result.status === null || result.status === 0)
      throw new Error(`${name}: gate did not reject the deliberate fault`);
    results.push({ name, rejected: true, exitCode: result.status });
    console.log(`${name}: rejected deliberate fault (exit ${result.status})`);
  } finally {
    if (old === undefined) rmSync(path, { force: true });
    else writeFileSync(path, old);
  }
}
probe("lint", "src/sdk/src/gate-probe.ts", "const unused = 1;\n", "pnpm", [
  "exec",
  "eslint",
  "src/sdk/src/gate-probe.ts",
]);
probe(
  "test",
  "src/sdk/tests/gate-probe.test.ts",
  'import { it, expect } from "vitest";\nit("rejects", () => expect(1).toBe(2));\n',
  "pnpm",
  ["exec", "vitest", "run", "src/sdk/tests/gate-probe.test.ts"],
);
probe(
  "coverage",
  "src/sdk/src/gate-probe.ts",
  Array.from({ length: 100 }, (_, i) => `export function untested${i}() { return ${i}; }`).join(
    "\n",
  ),
  "pnpm",
  ["test:coverage"],
);
probe("file-size", "src/sdk/src/gate-probe.ts", "// deliberate excess\n".repeat(301), "node", [
  "scripts/check-architecture.mjs",
]);
const schemaPath = "src/sdk/src/generated/schema.d.ts";
probe(
  "contract-drift",
  schemaPath,
  `${readFileSync(schemaPath, "utf8")}\n// deliberate drift\n`,
  "pnpm",
  ["sdk:check"],
);
writeFileSync("work/gate-zero/results.json", JSON.stringify(results, null, 2));
