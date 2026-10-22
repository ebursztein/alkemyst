import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));

function parseOrigin(origin) {
  let url;
  try {
    url = new URL(origin);
  } catch {
    throw new Error("A valid site origin is required");
  }
  if (
    url.protocol !== "https:" ||
    url.pathname !== "/" ||
    url.username ||
    url.password ||
    url.search ||
    url.hash
  ) {
    throw new Error("Site origin must be an HTTPS root without credentials, query or fragment");
  }
  return url;
}

export function writeSiteManifest({ dist, origin, commit, tree }) {
  const url = parseOrigin(origin);
  if (!/^[a-f0-9]{40}$/.test(commit)) throw new Error("Invalid site source commit");
  if (!/^[a-f0-9]{40}$/.test(tree)) throw new Error("Invalid site source tree");
  for (const file of [
    "index.html",
    "download/index.html",
    "docs/index.html",
    "pagefind/pagefind.js",
    "404.html",
  ]) {
    if (!existsSync(join(dist, file))) throw new Error(`Incomplete static output: ${file}`);
  }
  const manifest = { origin: url.origin, commit, tree };
  writeFileSync(join(dist, "deployment.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const status = execFileSync("git", ["status", "--porcelain"], { cwd: root, encoding: "utf8" });
  if (status.trim())
    throw new Error("Commit the site candidate before building deployment provenance");
  const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  const tree = execFileSync("git", ["rev-parse", "HEAD^{tree}"], {
    cwd: root,
    encoding: "utf8",
  }).trim();
  const origin = process.env.SITE_URL;
  // Validate the public origin before invoking Astro, without writing a fake manifest.
  const url = parseOrigin(origin);
  const site = join(root, "src", "site");
  const astro = join(site, "node_modules", "astro");
  const pkg = JSON.parse(readFileSync(join(astro, "package.json"), "utf8"));
  const bin = typeof pkg.bin === "string" ? pkg.bin : pkg.bin.astro;
  execFileSync(process.execPath, [resolve(astro, bin), "build"], { cwd: site, stdio: "inherit" });
  writeSiteManifest({ dist: join(site, "dist"), origin, commit, tree });
  console.info(`Static site ready: ${url.origin} at ${commit}`);
}
