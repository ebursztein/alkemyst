import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import openapiTS, { astToString } from "openapi-typescript";

const schema = execFileSync(
  "cargo",
  ["run", "--quiet", "--locked", "-p", "alkemyst-core", "--example", "export_openapi"],
  { encoding: "utf8" },
);
const parsed = JSON.parse(schema);
const generated = astToString(await openapiTS(parsed));
const outputs = [
  ["src/sdk/schema/openapi.json", schema],
  ["src/sdk/src/generated/schema.d.ts", generated],
];
if (process.argv.includes("--check")) {
  for (const [path, expected] of outputs) {
    if (readFileSync(path, "utf8") !== expected)
      throw new Error(`Generated contract drift: ${path}. Run pnpm sdk:generate.`);
  }
  console.log("OpenAPI and generated SDK are current.");
} else {
  for (const [path, content] of outputs) {
    mkdirSync(path.slice(0, path.lastIndexOf("/")), { recursive: true });
    writeFileSync(path, content);
  }
}
