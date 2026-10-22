import { readFileSync, readdirSync } from "node:fs";
import { extname, join, relative, resolve, sep } from "node:path";
import ts from "typescript";

const ignored = new Set([
  "node_modules",
  "dist",
  "target",
  "coverage",
  "work",
  ".git",
  ".sprinty",
  ".astro",
]);
const extensions = new Set([
  ".rs",
  ".ts",
  ".tsx",
  ".js",
  ".mjs",
  ".astro",
  ".sql",
  ".css",
  ".sh",
  ".ps1",
]);
const normalized = (path) => path.split(sep).join("/");

function files(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    if (ignored.has(entry.name)) return [];
    const path = join(dir, entry.name);
    return entry.isDirectory() ? files(path) : [path];
  });
}

export function checkArchitecture(root) {
  const config = JSON.parse(readFileSync(join(root, "architecture.json"), "utf8"));
  const errors = [];
  const packageEdges = new Map(Object.keys(config.packages).map((name) => [name, new Set()]));
  const isTest = (name) => /(^|\/)tests?\/|\.(?:test|spec)\.[cm]?[jt]sx?$/.test(name);
  for (const path of files(root)) {
    const name = normalized(relative(root, path));
    if (config.exemptions.includes(name) || !extensions.has(extname(name))) continue;
    const source = readFileSync(path, "utf8");
    const lines = source === "" ? 0 : source.replace(/\n$/, "").split("\n").length;
    if (lines > config.maxLines) errors.push(`${name}: ${lines} lines exceeds ${config.maxLines}`);
    if (!name.startsWith("src/")) continue;
    const owner = name.split("/")[1];
    if (
      !isTest(name) &&
      extname(name) === ".rs" &&
      /#\[(?:tokio::)?test\]|#\[cfg\(test\)\]\s*mod\s+\w+\s*\{/.test(source)
    ) {
      errors.push(`${name}: Rust tests must live in separate files`);
    }
    if (
      !isTest(name) &&
      !name.includes("/db/") &&
      /rusqlite::(?:Connection|Transaction)|\.prepare_cached\(|\.execute_batch\(/.test(source)
    ) {
      errors.push(`${name}: raw database access belongs to the statement engine`);
    }
    if (
      extname(name) === ".sql" &&
      !name.startsWith("src/core/migrations/") &&
      !name.includes("/statements/")
    ) {
      errors.push(`${name}: SQL must live in statement files or migrations`);
    }
    if (
      !isTest(name) &&
      !name.includes("/statements/") &&
      /["`]\s*(?:SELECT\s|INSERT\s+INTO\s|UPDATE\s+\w+\s+SET\s|DELETE\s+FROM\s|CREATE\s+TABLE\s)/i.test(
        source,
      )
    ) {
      errors.push(`${name}: SQL literals belong in statement modules`);
    }
    if (!/\.[cm]?[jt]sx?$/.test(name)) continue;
    const ast = ts.createSourceFile(name, source, ts.ScriptTarget.Latest, true);
    function inspect(node) {
      if (
        !isTest(name) &&
        ts.isCallExpression(node) &&
        ts.isIdentifier(node.expression) &&
        ["test", "it", "describe"].includes(node.expression.text)
      ) {
        errors.push(`${name}: tests must live in separate files`);
      }
      if (
        (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) &&
        node.moduleSpecifier &&
        ts.isStringLiteral(node.moduleSpecifier)
      ) {
        const spec = node.moduleSpecifier.text;
        const target = spec.startsWith("@alkemyst/")
          ? spec.split("/")[1]
          : spec.startsWith(".")
            ? normalized(relative(root, resolve(path, "..", spec))).split("/")[1]
            : undefined;
        if (target && target !== owner && packageEdges.has(target)) {
          packageEdges.get(owner)?.add(target);
          if (!(config.packages[owner] ?? []).includes(target))
            errors.push(`${name}: forbidden dependency on ${target}`);
          if (!/^@alkemyst\/[^/]+$/.test(spec))
            errors.push(`${name}: use the public ${target} package export`);
        }
      }
      ts.forEachChild(node, inspect);
    }
    inspect(ast);
  }
  function visit(name, path) {
    if (path.includes(name)) {
      errors.push(`Circular package dependency: ${[...path, name].join(" -> ")}`);
      return;
    }
    for (const target of packageEdges.get(name) ?? []) visit(target, [...path, name]);
  }
  for (const name of packageEdges.keys()) visit(name, []);
  return [...new Set(errors)];
}
