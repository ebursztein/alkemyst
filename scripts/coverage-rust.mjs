import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";

mkdirSync("coverage/rust", { recursive: true });
execFileSync(
  "cargo",
  [
    "llvm-cov",
    "--workspace",
    "--locked",
    "--json",
    "--output-path",
    "coverage/rust/coverage.json",
    "--ignore-filename-regex",
    "(/tests/|/examples/)",
    "--fail-under-lines",
    "85",
  ],
  { stdio: "inherit" },
);
const report = JSON.parse(readFileSync("coverage/rust/coverage.json", "utf8"));
const files = report.data.flatMap((entry) => entry.files);
const domain = files.filter((file) =>
  new RegExp("/src/core/src/(db|domain|analysis|practice|statements)/").test(
    file.filename.split(String.fromCharCode(92)).join("/"),
  ),
);
const lines = domain.reduce(
  (sum, file) => ({
    covered: sum.covered + file.summary.lines.covered,
    count: sum.count + file.summary.lines.count,
  }),
  { covered: 0, count: 0 },
);
if (lines.count > 0 && lines.covered / lines.count < 0.9)
  throw new Error(`Core coverage below 90%: ${lines.covered}/${lines.count}`);
execFileSync(
  "cargo",
  [
    "llvm-cov",
    "report",
    "--lcov",
    "--output-path",
    "coverage/rust/lcov.info",
    "--ignore-filename-regex",
    "(/tests/|/examples/)",
  ],
  { stdio: "inherit" },
);
