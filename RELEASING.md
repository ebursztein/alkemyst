# Preview releases

The preview pipeline accepts only `vMAJOR.MINOR.PATCH-preview.N` tags, with a
positive preview number. Root npm, desktop, SDK, site, Cargo workspace and Tauri
versions must all equal the tag without `v`. `pnpm check:release-version` enforces
that consistency before a tag exists; the normal quality gate includes it.

Commit matching workspace versions, Cargo.lock and the regenerated OpenAPI schema.
Complete the required source, security and actual installer checks and merge the
candidate through protected main before creating its tag. Do not move published
tags or reuse preview numbers. Signing/notarization is still required before stable
distribution; this workflow rejects stable tags instead of labeling unsigned
packages as stable.

For the initial version, after the release holds are cleared:

```sh
git tag v0.1.0-preview.1
git push origin v0.1.0-preview.1
```

## What the workflow proves

The source gate requires a clean checkout, a tag pointing at that checkout, matching
versions and a commit reachable from origin/main. It then runs the reusable full
quality, OSV and macOS/Windows installer workflows on the tagged source. A failed
job prevents draft creation. Only the final assembly job has repository write
permission; build jobs use read-only permissions.

Both installer manifests must identify the same source commit and version. The
assembler checks exact filenames/targets, regular nonempty files, byte counts and
SHA256 hashes before creating a fresh output directory. It rejects missing,
tampered, stale and mismatched input. The resulting draft contains the DMG, NSIS
installer, combined manifest and SHA256SUMS. The release body links to installation
guidance at that exact source commit and records its commit/tag.

The publish job rechecks the remote tag before using the artifacts. It creates a
**draft prerelease**, never a stable release or an automatically published download.
An interrupted upload can resume into a matching draft. A published release, an
unrelated draft or an authentication/read error fails instead of silently replacing
release contents. Rerun the failed workflow after fixing an external interruption;
source changes require a new version/tag.

## Publication and remaining prerequisites

Before publishing a draft, verify both platform results, package/source manifests,
checksums and [installation guidance](src/desktop/INSTALLERS.md). Keep distribution
holds active while quality, installation or signing status is unresolved. The
website must describe the actual public download status and point only to verified
published assets. A local build, CI artifact or draft is not a public release.

Preview macOS packages use ad-hoc signing without notarization, and Windows
previews are unsigned. Stable distribution needs the appropriate Apple Developer
signing/notarization credentials and a trusted Windows signing setup. No signing
credential is stored in the repository or required for preview compilation.
