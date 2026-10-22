# First-launch card setup

Tracking can capture games before any card metadata has been downloaded. Until
metadata is available, missing identities stay unresolved and unsupported analysis
remains blocked. A workspace notice links to Settings rather than inventing card
names, costs, or probabilities.

## Scryfall metadata

Choose **Download card metadata** in Settings for the first import. It downloads
the public default-cards archive into a local SQLite snapshot. The download and
import can take a few minutes; the application remains available. Failed or
interrupted refreshes preserve the previous complete snapshot. Retry using the
same button. Later, **Update card metadata** obtains a newer snapshot if available.

Once imported, card search, rules metadata, deck curves and supported simulations
use local data. Card artwork still loads online. The snapshot timestamp is shown
in Settings. The catalog includes cards the bounded simulator cannot model;
unsupported interactions remain explicit.

## Installed Arena identities

While the tray application runs, Alkemyst discovers installed public Arena card
databases and imports their ID mappings read-only. This is separate from Scryfall
metadata: installed IDs alone do not provide the rules catalog. Standard macOS,
Windows, Steam and common standalone/Epic locations are searched. Steam library
configuration and the Arena installation manifest support custom library folders;
the Windows Steam registry key is read without changing it.

New cache files are checked periodically. Unchanged files avoid repeated imports.
A failed read keeps previously imported mappings. **Retry card discovery** checks
again. For an installation that cannot be discovered, **Choose Arena card cache…**
opens the native file picker for its `Raw_CardDatabase_*.mtga` file. The selection
is used for this session; its path is not stored in the Alkemyst database. Imported
identities remain available after restart even if the source cannot be found.

Settings reports installed identities separately from Scryfall's direct Arena
IDs; these counts overlap and should not be added together. An imported identity
resolves only when an exact corresponding printing exists in the metadata.
Unknown or mismatched records remain unknown.

## API and client cache

`GET /v1/catalog` exposes `arena_aliases` and `arena_alias_revision` through the
OpenAPI-generated SDK and the same MCP application operation. Its prepared SQL
statement owns the counts and revision. Committed imports invalidate dependent
SQL results. The desktop observes snapshot/revision changes and refreshes active
application queries, including previously unresolved cards, without invalidating
native OS-settings queries or issuing per-card requests.

## Verification

Isolated tests cover a cold database, missing/invalid caches, read-only imports,
custom Steam configuration, escaped Windows paths, source reuse, new cache
versions, native-window guards and stale client reads. A browser workflow starts
with captured history but no metadata, injects one external-provider failure,
retries through the real Rust API/SQLite/SDK path, and verifies unchanged history
and resolved deck curves. The development-only fixture provider is not read by
the native application. Live validation additionally exercised installed Arena
data and an actual Scryfall refresh using a disposable database and sanitized logs.
