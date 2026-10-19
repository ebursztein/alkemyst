# Alkemyst

Local-first Arena deck laboratory. Original code is Apache-2.0, copyright Elie Bursztein.

## Work order

Use Sprinty: info, resume, claim the owning item before edits. Gate zero must pass
before product features. Keep the approved design in CONTRIBUTING.md and the
Sprinty ledger aligned. Never declare a milestone complete without its evidence.

## Structure

Production code belongs under src/: desktop, core, landlord, sdk, site.
Modules have one responsibility and at most 300 physical lines. Tests have their
own files and the same limit. Split along real seams; never minify to evade it.
Generated and unchanged vendor exceptions must be listed in architecture.json.
Never exempt modified vendor code by treating it as unchanged.
No circular dependencies or imports into another package's private modules.

## Data and API

Only the core statement engine accesses SQLite. SQL belongs in dedicated domain
statement files or migrations; SQL owns relational work. One domain statement per
public read. The engine owns the only result cache below the API and invalidates
it on committed dependency changes. Preserve missing values and sample counts.
OpenAPI generates SDK types; never hand-maintain parallel API schemas. REST and
MCP share application operations. No raw SQL MCP tool. Local API binds loopback.
Captured game history is immutable; deck edits create versions.

## Verification

Write a separate failing behavior test before implementing behavior. Run pnpm
verify, pnpm security, and relevant browser/native checks. Clippy denies warnings.
Do not lower coverage or disable gates to get a green build. Exemptions must be
specific, justified and reviewed. Keep tests for retries and interrupted writes.

## Privacy and dependencies

Never commit raw Arena logs, credentials, personal identifiers, or local user paths.
Real-game fixtures must be minimal and sanitized, retaining provenance without PII.
Preserve upstream licenses. Original landlord source is MIT. No cloud game storage.
Keep exact dependency/tool versions and lockfiles. No blanket vulnerability ignores.

## Release

Preview installers may be unsigned/ad-hoc. Stable releases require trusted signing.
Cloudflare hosts only marketing/docs. Credentials never belong in the repository.
