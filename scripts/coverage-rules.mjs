import { readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";

const normalized = (path) => path.split(String.fromCharCode(92)).join("/");

export function maintainedRustSources(root, exemptions) {
  const ignored = new Set(["target", "node_modules", "tests", "benches", "dist"]);
  function walk(directory) {
    return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
      if (ignored.has(entry.name)) return [];
      const path = join(directory, entry.name);
      if (entry.isDirectory()) return walk(path);
      if (!path.endsWith(".rs") || exemptions.includes(normalized(relative(root, path)))) return [];
      // Pure type/trait declarations have no executable lines to instrument.
      const source = readFileSync(path, "utf8");
      return /\bfn\s+\w+[^;{}]*\([^;{}]*\)[^;{}]*\{/.test(source) ? [resolve(path)] : [];
    });
  }
  return walk(join(root, "src"));
}

export function checkRustCoverage(files, expected = []) {
  const groups = [
    ["Overall", 0.85, () => true],
    ["Landlord", 0.9, (path) => /\/src\/landlord\/src\//.test(path)],
    ["SQL engine", 0.9, (path) => /\/src\/core\/src\/(db|statements)\//.test(path)],
    ["Domain", 0.9, (path) => /\/src\/core\/src\/(domain|analysis|practice)\//.test(path)],
  ];
  const errors = [];
  const reported = new Set(files.map((file) => normalized(file.filename)));
  for (const path of expected) {
    if (!reported.has(normalized(path)))
      errors.push(`Maintained Rust source missing from coverage: ${path}`);
  }
  for (const [name, threshold, includes] of groups) {
    const lines = files
      .filter((file) => includes(file.filename.split(String.fromCharCode(92)).join("/")))
      .reduce(
        (sum, file) => ({
          covered: sum.covered + file.summary.lines.covered,
          count: sum.count + file.summary.lines.count,
        }),
        { covered: 0, count: 0 },
      );
    if (lines.count > 0 && lines.covered / lines.count < threshold)
      errors.push(`${name} coverage below ${threshold * 100}%: ${lines.covered}/${lines.count}`);
  }
  return errors;
}
