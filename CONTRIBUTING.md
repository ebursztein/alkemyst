# Contributing

## Architecture

SQLite → prepared statement engine → application operations → REST/MCP.
OpenAPI generates the TypeScript SDK; React uses that SDK with TanStack Query.
The desktop UI and MCP never access storage directly. SQL owns joins, filtering,
grouping and pagination. A bounded statement result cache uses committed table
revisions. All application sources live under src/.

Use small modules (300 physical lines maximum), explicit package exports and
separate test files. Rust unit-test modules reference separate files; integration
tests belong in tests/. TypeScript tests use *.test.ts or *.test.tsx.

## Gates

`pnpm verify` runs formatting, architecture, lint, types, tests, coverage and
OpenAPI drift checks. `pnpm security` runs pinned OSV-Scanner against both
lockfiles. `pnpm test:e2e` runs browser workflows. Native checks remain required
for menus, tray lifecycle, installers and OS integration.

Coverage includes untested owned modules. Rust minimum: 85% lines overall and
90% for SQL/domain code. TypeScript minimum: 85% lines/statements/functions and
80% branches. Generated and unmodified third-party source is excluded explicitly.
Coverage reports are CI artifacts; thresholds are never lowered to mask failures.

CI keeps PR coverage/browser and security reports for one day, and main/tag reports
for seven days. Failed Windows loader diagnostics remain available for seven days.
Installer verification still runs on PRs, but installer binaries are uploaded only
for explicit manual builds or release tags, with one-day transfer retention.
Long-lived installers belong in GitHub Releases. Existing archives retain their
original expiry unless separately removed.

Dependency findings block CI. Any exceptional suppression must identify the
advisory, explain applicability, name an owner, and expire. Scanner errors fail.

## Development sequence

Track work in Sprinty and claim a dedicated item before editing. Write a failing
behavior test, implement it, then refactor. Foundation gates precede features;
backend and MCP precede screens. Record normal, retry, interruption and relevant
concurrency evidence. Commits use conventional messages.

## Fixtures

Do not commit raw Arena logs. Real-game fixtures contain only needed game events,
with identities pseudonymized and secrets, account names and paths removed. Keep
source logs untouched. Fixtures are reviewed for privacy before publication.

## Website and releases

One Astro/Starlight site hosts marketing at /, downloads at /download/, and docs
at /docs/. Cloudflare serves static assets. Desktop artifacts live on GitHub.
Unsigned preview builds are labeled; stable builds require signing/notarization.
