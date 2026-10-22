import { checkArchitecture } from "./architecture-rules.mjs";
import { checkDependencyPins } from "./dependency-pins.mjs";
const root = process.argv[2] ?? process.cwd();
const errors = [...checkArchitecture(root), ...checkDependencyPins(root)];
if (errors.length) {
  console.error(errors.join("\n"));
  process.exitCode = 1;
} else console.log("Architecture checks passed.");
