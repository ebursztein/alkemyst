#!/usr/bin/env bash
set -euo pipefail

# Uses an isolated install destination; never replaces /Applications or user data.
repo=$(cd "$(dirname "$0")/.." && pwd)
packages=${1:?Pass the staged macOS package directory}
packages=$(cd "$packages" && pwd)
manifest="$packages/manifest.json"
name=$(node -p 'JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")).file' "$manifest")
version=$(node -p 'JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")).version' "$manifest")
test_dir=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/alkemyst-install.XXXXXX")
mounted=0
cleanup() {
  if [[ $mounted -eq 1 ]]; then hdiutil detach "$test_dir/mount"; fi
  rm -rf "$test_dir"
}
trap cleanup EXIT
mkdir "$test_dir/mount" "$test_dir/installed"
(cd "$packages" && shasum -a 256 -c SHA256SUMS)
hdiutil verify "$packages/$name"
hdiutil attach -readonly -nobrowse -noautoopen -mountpoint "$test_dir/mount" "$packages/$name"
mounted=1
test -L "$test_dir/mount/Applications"
ditto "$test_dir/mount/Alkemyst.app" "$test_dir/installed/Alkemyst.app"
hdiutil detach "$test_dir/mount"
mounted=0
app="$test_dir/installed/Alkemyst.app"
codesign --verify --deep --strict --verbose=2 "$app"
test "$(lipo -archs "$app/Contents/MacOS/alkemyst-desktop")" = arm64
test "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$app/Contents/Info.plist")" = cards.alkemyst.desktop
test "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$app/Contents/Info.plist")" = "$version"
test "$(/usr/libexec/PlistBuddy -c 'Print :LSMinimumSystemVersion' "$app/Contents/Info.plist")" = 13.0
if [[ ${GITHUB_ACTIONS:-} != true ]]; then
  echo 'Package inspection passed. Installed runtime launch is restricted to disposable CI runners.'
  exit 0
fi
node "$repo/scripts/installed-runtime.mjs" "$app/Contents/MacOS/alkemyst-desktop"
echo 'macOS preview copy-install and installed runtime verification passed.'
