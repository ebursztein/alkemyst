# Alkemyst

Brew. Play. Refine.

A local-first Magic: The Gathering Arena deck laboratory for macOS and Windows.
Automatic match capture, deck versions, mana analysis, and local MCP access.

**Status: foundation under construction. No downloadable app yet.**

## Development

Requirements: Node 24.14.1, pnpm 10.33.2, Rust 1.95.0.

```sh
pnpm install --frozen-lockfile
pnpm verify
pnpm security
pnpm dev
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for architecture and required checks.
Original code: [Apache-2.0](LICENSE). Third-party code retains its own licenses.
Alkemyst is an independent project, not affiliated with Wizards of the Coast.

## Local catalog development

The Rust core can refresh a complete Scryfall default-card catalog, search it
without a network connection, and resolve Arena card IDs. Search covers names,
rules and card faces, with color, mana value, type, legality and legendary filters.
A complete refresh publishes atomically; failure leaves the previous catalog active.
Card images are referenced by URL and are not downloaded by the catalog importer.

```sh
mkdir -p work
cargo run -p alkemyst-core --example catalog -- refresh work/catalog.sqlite3
cargo run -p alkemyst-core --example catalog -- search work/catalog.sqlite3 equipment
cargo run -p alkemyst-core --example catalog -- status work/catalog.sqlite3
```

For Arena-only printing IDs, import the installed `Raw_CardDatabase_*.mtga` file
using `catalog arena <db> <asset-path>`. The asset opens read-only. Resolution
requires matching set, collector number and name; unresolved IDs remain explicit.
No game assets or downloaded card databases are committed to this repository.

The catalog retains every field supplied in Scryfall's default-card export,
including faces, legalities and external image links. This is the English-preferred
printing catalog, not every translation or a separate rulings/tag download.
Searchable rules text does not imply simulation support for a card's mechanics.

## Deck core

The core provides immutable deck versions, idempotent creation, append-only
agent edits, notes, and Arena text import/export. Captured card references retain
Arena IDs; library selections can use Oracle IDs or exact Scryfall printings.
Unknown imports are reported instead of silently dropping card lines. SQLite
triggers protect sealed versions, including their card lists. REST/MCP and desktop
interfaces are being built on these shared operations.
