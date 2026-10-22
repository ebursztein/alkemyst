import { appendFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { verifyReleaseSource, workspaceReleaseVersion } from "./release-source.mjs";
import { assembleReleaseAssets } from "./release-assets.mjs";
import { publishPreviewDraft } from "./release-publish.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const [mode, tag] = process.argv.slice(2);
if (mode === "--check-version") {
  console.info(`Workspace preview version is consistent: ${workspaceReleaseVersion(root)}`);
} else {
  if (!["--check-source", "--assemble", "--publish-draft"].includes(mode))
    throw new Error("Use --check-source, --assemble or --publish-draft followed by a preview tag");
  const source = verifyReleaseSource({ root, tag });
  if (mode !== "--check-source") {
    const release = assembleReleaseAssets({
      source,
      artifacts: join(root, "work", "release-artifacts"),
      output: join(root, "work", "release"),
    });
    console.info(`Verified both preview packages for ${release.tag} at ${release.commit}`);
    if (mode === "--publish-draft")
      publishPreviewDraft({ release, directory: join(root, "work", "release") });
  } else console.info(`Verified preview source ${source.tag} at ${source.commit}`);
  if (process.env.GITHUB_OUTPUT)
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `version=${source.version}\ncommit=${source.commit}\n`,
    );
}
