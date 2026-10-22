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
triggers protect sealed versions, including their card lists. REST and the generated
TypeScript SDK and MCP now expose these operations; desktop integration is next.

## Local API development

The Axum service exposes catalog search/refresh, card resolution, deck versions,
Arena import/export and notes. `/openapi.json` is the OpenAPI 3.1 contract;
`/v1/discovery` lists implemented operations. Both require a local bearer credential.
Only `/v1/health`, containing build identity, is public. The listener refuses
non-loopback addresses and protects application data with host, origin and bearer
checks. The production origin allowlist covers Tauri; the developer example also
permits Vite on localhost port 1420. Never expose the service through a tunnel.

```sh
# Generate a disposable credential without placing it in shell history.
export ALKEMYST_TOKEN=$(openssl rand -hex 32)
cargo run -p alkemyst-core --example local_service -- work/catalog.sqlite3
```

The example prints its base URL and process ID, never the credential. Keep the
credential out of URLs and logs. Ctrl-C stops the listener. Native bootstrap will
supply connection details to the desktop; this example is a developer entry point.

`@alkemyst/sdk` provides `createAlkemystClient(baseUrl, token)` and
`createAlkemystQueryClient(baseUrl, token)`. Types are generated from Rust's
OpenAPI contract with `pnpm sdk:generate`; `pnpm sdk:check` rejects drift. Commands
carry an `idempotency_key` and `input`; retrying the same logical command returns
the original result, including when retried through another transport. Changed
input with a reused key returns a typed conflict. Pagination uses opaque cursors.

SDK integration tests launch an isolated Rust service with synthetic card data.
They exercise real HTTP authentication, deck retries, search, errors and TanStack
Query access without downloading cards or reading personal Arena history.

## Local MCP development

The same service exposes authenticated Streamable HTTP at `/mcp` on its printed
loopback base URL. Configure an MCP client with that HTTP URL and an
`Authorization: Bearer <local credential>` header. Use the credential already
provided to the service, without pasting it into URLs, repository files or logs.
The native Settings connection screen will expose this configuration when the
desktop shell lands. The developer example must currently remain running.

MCP exposes discovery, card search/metadata/resolution, catalog status/refresh,
decks, immutable versions, Arena import/export and notes. Input and output schemas
come from OpenAPI through an explicit operation allowlist. Adding an HTTP endpoint
does not automatically grant agents access. There is no SQL or file-path tool.
Game/statistics and simulation tools arrive with their corresponding operations.

Successful structured results wrap the REST response in `data`; tool failures
return `isError` with the same typed problem as REST. MCP mutations record agent
provenance. SQLite, prepared statements, the bounded result cache, and refresh
coordination are shared with REST. Sessions contain no separate domain storage.
Validation uses local JSON schemas with HTTP and file retrieval disabled.

## Local Arena capture development

The pinned native parser now feeds a durable ingestion adapter. It discovers the
normal Arena log pair, imports retained `Player-prev.log` history and follows
`Player.log`. Missing files are normal while Arena is closed. Tracking health
reports waiting, tracking, degraded or stopped; temporary errors can recover.
Native desktop lifecycle and timeline screens are still being built.

```sh
cargo run -p alkemyst-core --example capture -- import work/plays.sqlite3 /path/to/Player-prev.log
cargo run -p alkemyst-core --example capture -- status work/plays.sqlite3
cargo run -p alkemyst-core --example capture -- watch work/plays.sqlite3
```

`watch` uses native discovery by default. To select files explicitly, pass the
previous log and current log after the database path. Ctrl-C stops tracking and
prints final counts. The utility emits only counts and health, never raw events,
player identities or credentials. The original Arena files open read-only.

Checkpoint updates commit in the same transaction as normalized events and game
records. A partial line or entry is reread after restart. File identity and content
anchors detect replacement and truncation. Large files stream in bounded batches;
a single entry over 8 MiB reports a capture error and preserves the checkpoint.
Reimporting a log or its retained copy never creates duplicate games. Captured
facts and game attribution are protected by SQLite immutability triggers.

Deck submissions and sideboard responses create immutable versions; explicit
reconnect lists can recover missing deck submissions. A fresh match cannot inherit
an old submission without new evidence. Match and game identities remain separate,
including Bo3. Missing seats, teams, results and winners remain unknown. The
`unknown_results` count includes unfinished games and conflicting outcomes. It is
never interpreted as a loss. Sparse captures retain their gaps for timeline and
statistics queries to expose.

Only normalized gameplay data is stored: account/session records, player names,
chat, opaque raw envelopes and credentials are excluded. Source identifiers and
external match/deck identifiers are hashed. Saved deck names remain local user
data. Personal databases and raw logs must stay outside Git.

## History API and interpretation

The generated SDK and MCP expose the same SQL-backed operations:

| REST resource                     | MCP operation             | Meaning                                                                              |
| --------------------------------- | ------------------------- | ------------------------------------------------------------------------------------ |
| `/v1/games`, `/v1/games/{id}`     | `gamesList`, `gameGet`    | Games, immutable deck versions, observed counters and capture flags                  |
| `/v1/matches`, `/v1/matches/{id}` | `matchesList`, `matchGet` | Matches and their games; match-scoped outcomes stay distinct                         |
| `/v1/games/{id}/events`           | `gameTimeline`            | Captured events; optional `turn_number` filter                                       |
| `/v1/games/{id}/turns`            | `gameTurns`               | SQL-grouped turns, observed actions, life, land counters and unassigned-event counts |
| `/v1/statistics`                  | `statisticsGet`           | Game-level performance with explicit denominators                                    |
| `/v1/capture`                     | `captureStatusGet`        | Durable ingestion counts                                                             |

Lists support deck, version, match and mode filters where applicable. They sort
by first capture order, not an invented game-start timestamp. Keep cursors opaque.
Event and turn pagination retain a fixed upper capture boundary, including state
lookups: later backfills cannot reassign events inside an older snapshot. Actions
join turns through explicit state IDs; unassigned events remain visible.

`capture_complete` means no defined missing-data condition was detected. It does
not certify a complete Magic rules replay. Flags cover absent initial/terminal
states, broken or unverifiable state chains, truncation, explicit interruptions,
unknown attribution/seats and conflicting results. Healthy connection transitions
and post-result teardown do not alone prove missing gameplay. Unclassified legacy
connection markers remain in the event feed. A known outcome can coexist with a
partial capture. Conflicting outcomes become unknown, never an arbitrary win/loss.

`mode` currently recognizes the observed `Ladder` and `Traditional_Ladder` queue
identifiers as Bo1 and Bo3. Other queues and older captures without this evidence
remain `unknown`; a single observed game is not proof of Bo1. Match outcomes require
an explicit match-scoped result and are never inferred from a game's outcome.

`decisive_win_rate` is wins divided by wins plus losses. Draws and unknown results
are excluded; `decisive_games` is its denominator. Empty samples return null rates.
Game counts, unique matches, draws, unknowns, partial captures and per-mode counts
are separate. `complete_only` is optional. Recorded land counters cover sampled
local turns, excluding stale counters on the opponent's turn. Mulligan averages
use explicit observed counters and report their sample count.

`local_time` is a zone-less first observed game clock. `match_instant_utc` is
explicit match-clock evidence, not a reconstructed game-start timestamp. Event
`instant_utc` values exist only where the source supplies an actual UTC instant.

## Deck analysis profiles

`GET /v1/deck-versions/{id}/analysis` (`analysisProfileGet` in MCP) returns the
printed mana curve, source counts, catalog/compiler versions and per-card model
support. The generated SDK exposes the same operation. Missing cards and unknown
rules remain explicit even when curves can be displayed. The compiler keeps the
resolved immutable input for persisted simulation jobs. See [compiler boundaries](src/core/src/analysis/compiler/README.md).

## Simulation jobs

The local API now queues seeded simulations with frozen inputs and durable
progress. REST, the generated SDK and MCP expose creation, status, listing,
inputs and cancellation; exact draw/joint probability endpoints are separate.
Seeds are hexadecimal strings to preserve every bit in JavaScript. See
[job semantics](src/core/src/analysis/jobs/README.md) for scope, replay versions,
cancellation, recovery and sampling intervals.

## Deck version comparison

`GET /v1/deck-versions/{id}/compare/{candidate_id}` (`deckVersionsCompare` in MCP)
compares two immutable versions against the same current catalog snapshot. SQL
normalizes resolved printings to Oracle identities, keeps zones separate and
returns quantity changes and printed nonland curve deltas. Unresolved references
remain explicit, including null mana-value bins. Printing-only changes cancel
out when both references resolve to the same card. These deltas contain no
inferred win rates or simulation results; compare those through their separately
labeled operations and sample counts.

## Manual practice

`/v1/practice` provides saved opening hands, London mulligans, ordered bottoming,
draws, basic turns and visible-card controls through REST, the generated SDK and
MCP. Sessions freeze their deck metadata and seed; immutable steps use revision
checks and idempotency keys. Practice is a manual sandbox, with mana payments,
card abilities and combat left to the player. See [practice semantics](src/core/src/practice/README.md).

`GET /v1/decks/{id}/versions` (`deckVersionsList` in MCP) provides paged immutable
version summaries with SQL-computed main/sideboard counts. The generated SDK
exposes the same opaque-cursor operation for version browsing and comparison.
