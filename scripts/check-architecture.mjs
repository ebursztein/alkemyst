import { checkArchitecture } from "./architecture-rules.mjs";
const errors = checkArchitecture(process.argv[2] ?? process.cwd());
if (errors.length) {
  console.error(errors.join("\n"));
  process.exitCode = 1;
} else console.log("Architecture checks passed.");
