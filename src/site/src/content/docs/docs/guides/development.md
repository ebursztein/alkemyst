---
title: Development
description: Workspace layout, pinned toolchains, quality gates and contribution rules.
---

Alkemyst's original code is Apache-2.0, copyright Elie Bursztein. Adopted `landlord`
code and redistributable Untitled UI components retain their MIT licenses and
attribution in the repository.

## Build a reviewed revision

Install the versions pinned in `.node-version`, `package.json` and
`rust-toolchain.toml`. Rustup supplies the Rust toolchain and requested components.
Use the operating system prerequisites from the
[Tauri documentation](https://v2.tauri.app/start/prerequisites/).

Clone the repository, then replace `REVISION` below with a reviewed release tag or
pull-request head commit. Unreleased preview work may not be on `main` yet.

```sh
git clone https://github.com/ebursztein/alkemyst.git
cd alkemyst
git checkout REVISION
npm install --global pnpm@10.33.2
pnpm install --frozen-lockfile
pnpm --filter @alkemyst/desktop tauri dev
```

The browser development server alone cannot supply native credentials. It shows
connection guidance rather than falling back to an unauthenticated API.

## Workspace

| Package        | Responsibility                                                        |
| -------------- | --------------------------------------------------------------------- |
| `src/core`     | SQLite worker, statements, ingestion, domain operations, REST and MCP |
| `src/landlord` | Maintained deterministic Rust simulation engine                       |
| `src/sdk`      | OpenAPI-generated TypeScript types and client access                  |
| `src/desktop`  | React/TanStack UI and the thin Tauri native shell                     |
| `src/site`     | Astro marketing pages and Starlight documentation                     |

Application data uses the SDK. Native commands handle OS integration and
connection bootstrap. Only the core statement engine accesses SQLite, and only
its dependency-aware result cache sits below the transports. SQL owns filtering,
joins, aggregation, ordering and cursor pagination.

## Required checks

```sh
pnpm verify
pnpm security
pnpm test:e2e
pnpm test:site
pnpm native:build
pnpm --filter @alkemyst/desktop native:smoke
```

Verification includes strict TypeScript and Astro checks, lint and formatting,
Clippy with warnings denied, separate test files, coverage thresholds, OpenAPI and
SDK drift checks, and architecture rules. Native and browser tests use isolated
fixtures. The OSV gate scans both lockfiles; exact expiring target-exclusion
exceptions and their owners are documented in the repository's `SECURITY.md`.

## Keep changes reviewable

Maintained source and test files have a maximum of **300 physical lines**. Split
by responsibility rather than compressing formatting. Tests belong in their own
files. Generated and unchanged upstream exceptions must be explicit. Imports use
public package exports; there are no circular package dependencies.

Start with a failing behavior test, preserve immutable captured history, and
include uncertainty and missing-data indicators in user-visible results. Maintain
seeded simulation replay compatibility or explicitly version a changed model.

Never contribute raw Arena logs, local databases, credentials or private player
identifiers. Minimal real-play fixtures must be sanitized. Read `AGENTS.md`,
`CONTRIBUTING.md` and the active Sprinty items before implementation.

## Public site and releases

Marketing and docs are one static Astro build. The planned deployment is
Cloudflare Workers Static Assets; desktop installers belong on GitHub Releases.
Domain ownership, Cloudflare deployment access and signing/notarization credentials
are separate release prerequisites, not requirements for running the app locally.
