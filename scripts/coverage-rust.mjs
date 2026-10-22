import { execFileSync } from "node:child_process";
import { appendFileSync, existsSync, mkdirSync, readFileSync } from "node:fs";
import { coverRustCli } from "./coverage-cli.mjs";
import { checkRustCoverage, maintainedRustSources } from "./coverage-rules.mjs";

const ignored = "(/rustc/|/\\.cargo/|/\\.rustup/|/target/|/tests/)";

mkdirSync("coverage/rust", { recursive: true });
mkdirSync("work", { recursive: true });
execFileSync("cargo", ["llvm-cov", "clean", "--workspace"], { stdio: "inherit" });
execFileSync(
  "cargo",
  [
    "llvm-cov",
    "--workspace",
    "--locked",
    "--features",
    "alkemyst-desktop/native-smoke,alkemyst-desktop/custom-protocol",
    "--no-report",
  ],
  { stdio: "inherit" },
);
await coverRustCli();
execFileSync(
  "cargo",
  [
    "llvm-cov",
    "report",
    "--include-build-script",
    "--json",
    "--output-path",
    "coverage/rust/coverage.json",
    "--no-default-ignore-filename-regex",
    "--ignore-filename-regex",
    ignored,
    "--fail-under-lines",
    "85",
  ],
  { stdio: "inherit" },
);
const report = JSON.parse(readFileSync("coverage/rust/coverage.json", "utf8"));
const files = report.data.flatMap((entry) => entry.files);
const architecture = JSON.parse(readFileSync("architecture.json", "utf8"));
const errors = checkRustCoverage(
  files,
  maintainedRustSources(process.cwd(), architecture.exemptions),
);
if (errors.length > 0) throw new Error(errors.join("\n"));
execFileSync(
  "cargo",
  [
    "llvm-cov",
    "report",
    "--include-build-script",
    "--lcov",
    "--output-path",
    "coverage/rust/lcov.info",
    "--no-default-ignore-filename-regex",
    "--ignore-filename-regex",
    ignored,
  ],
  { stdio: "inherit" },
);

const total = report.data[0].totals.lines;
console.info(`Rust line coverage: ${total.percent.toFixed(2)}% (${total.covered}/${total.count})`);
const adopted = files.filter((file) =>
  file.filename.replaceAll("\\", "/").includes("/src/landlord/src/"),
);
const landlord = adopted.reduce(
  (sum, file) => ({
    covered: sum.covered + file.summary.lines.covered,
    count: sum.count + file.summary.lines.count,
  }),
  { covered: 0, count: 0 },
);
const landlordSummary =
  landlord.count > 0
    ? `Maintained landlord lines: **${((100 * landlord.covered) / landlord.count).toFixed(2)}%** (${landlord.covered}/${landlord.count}), independently requires 90%.\n`
    : "";
if (landlordSummary) console.info(landlordSummary.trim());
if (process.env.GITHUB_STEP_SUMMARY) {
  let summary =
    `## Coverage\n\nRust lines: **${total.percent.toFixed(2)}%** (${total.covered}/${total.count}). ` +
    "SQL engine and domain each independently require 90%.\n" +
    landlordSummary;
  if (existsSync("coverage/coverage-summary.json")) {
    const ts = JSON.parse(readFileSync("coverage/coverage-summary.json", "utf8")).total;
    summary +=
      `\nTypeScript: ${ts.lines.pct}% lines, ${ts.statements.pct}% statements, ` +
      `${ts.functions.pct}% functions, ${ts.branches.pct}% branches.\n`;
  }
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
}
