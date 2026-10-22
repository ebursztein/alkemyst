import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";
import { parse } from "smol-toml";

const version = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;
const groups = ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"];
const cargoGroups = ["dependencies", "dev-dependencies", "build-dependencies"];

export function checkNpmDependencies(manifest) {
  return groups.flatMap((group) =>
    Object.entries(manifest[group] ?? {}).flatMap(([name, pin]) =>
      typeof pin === "string" && (version.test(pin) || /^workspace:(?:\*|\^|~)$/.test(pin))
        ? []
        : [`${group}.${name}: dependency must use an exact version or local workspace reference`],
    ),
  );
}

export function checkCargoDependencies(manifest) {
  const errors = [];
  for (const group of cargoGroups) {
    for (const [name, dependency] of Object.entries(manifest[group] ?? {})) {
      const spec = typeof dependency === "string" ? { version: dependency } : dependency;
      if (spec.workspace === true) continue;
      if (spec.git && (!/^[0-9a-f]{40}$/i.test(spec.rev ?? "") || spec.branch || spec.tag))
        errors.push(`${group}.${name}: git dependency requires an immutable full revision`);
      if (spec.version !== undefined) {
        if (!spec.version.startsWith("=") || !version.test(spec.version.slice(1)))
          errors.push(`${group}.${name}: dependency must use an exact =major.minor.patch version`);
      } else if (!spec.path && !spec.git) {
        errors.push(`${group}.${name}: dependency requires an exact version or local path`);
      }
    }
  }
  if (manifest.workspace) errors.push(...checkCargoDependencies(manifest.workspace));
  for (const target of Object.values(manifest.target ?? {}))
    errors.push(...checkCargoDependencies(target));
  return errors;
}

export function checkDependencyPins(root) {
  const config = JSON.parse(readFileSync(join(root, "architecture.json"), "utf8"));
  const ignored = new Set([
    ".git",
    ".sprinty",
    "node_modules",
    "target",
    "dist",
    "work",
    "coverage",
    ".astro",
  ]);
  function walk(directory) {
    return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
      if (ignored.has(entry.name)) return [];
      const path = join(directory, entry.name);
      if (entry.isDirectory()) return walk(path);
      const name = relative(root, path).replaceAll("\\", "/");
      if (config.exemptions.includes(name)) return [];
      if (entry.name === "package.json")
        return checkNpmDependencies(JSON.parse(readFileSync(path, "utf8"))).map(
          (error) => `${name}: ${error}`,
        );
      if (entry.name === "Cargo.toml")
        return checkCargoDependencies(parse(readFileSync(path, "utf8"))).map(
          (error) => `${name}: ${error}`,
        );
      return [];
    });
  }
  return walk(root);
}
