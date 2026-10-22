# Preview installer verification

`pnpm native:package` builds the release profile with the pinned Tauri CLI and
Cargo lockfile. macOS targets Apple Silicon/macOS 13+ and produces an ad-hoc signed
app and DMG; Windows targets x64 and produces an unsigned NSIS installer. These
are preview artifacts. Trusted signing and Apple notarization remain required
before stable distribution.

The packaging wrapper sets `CI=true`: Tauri then skips create-dmg's Finder cosmetic
layout. The interactive layout step can fail with a Finder AppleEvent timeout in
automated sessions. The DMG still includes the app and Applications shortcut.

Packages are copied into `work/packages/<target>/`, alongside `SHA256SUMS` and a
manifest recording the version, exact source commit, target, byte count and hash.
Missing, empty and wrong-version artifacts fail instead of becoming a release.
Build failures propagate. No installer is published by the verification workflow.

## Platform checks

The **Preview installer verification** workflow builds both targets on disposable
GitHub-hosted runners. It runs for relevant pull requests and supports manual
dispatch once the workflow exists on the default branch.

- macOS verifies the checksum and DMG, mounts read-only, copies the app into a
  temporary installation, detaches the image, verifies the copied signature,
  architecture, bundle identity, version and minimum OS, then launches that copy.
- Windows verifies the checksum, silently installs into a temporary path containing
  spaces, verifies the installed x64 executable's product/version, launches it,
  then silently uninstalls and verifies the executable was removed.

Installed-runtime checks use the actual packaged executable: public build-health
must succeed, and deck reads and MCP must reject unauthenticated requests.
An existing listener fails the check
before launch. The helper terminates only its own child after the probe; this
cleanup does not count as graceful-Quit verification. The separate native smoke
test exercises the real React window, tray, background capture, MCP and normal Quit.

Runtime installation checks run only on disposable CI profiles. Local macOS
inspection is available with
`bash scripts/verify-macos-install.sh work/packages/aarch64-apple-darwin`; it checks
and copy-installs the DMG but skips launching an app against the user's profile.
Windows install/uninstall verification refuses to run outside CI.

Successful workflow artifacts contain installers, checksums and manifests for
14 days. Download-page links must point to a verified GitHub Release once the
separate preview-release workflow is enabled. Source builds and verification
artifacts alone do not establish an available public release.
