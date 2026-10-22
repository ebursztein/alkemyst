# Local backup and restore

In Settings, choose **Save backup…** and select a new `.sqlite3` file name. Existing
files are never replaced. SQLite takes a consistent snapshot of committed data,
including its write-ahead log, while the application remains available.

The file includes decks and immutable versions, notes, captured matches/games and
events, ingestion checkpoints, practice sessions, simulation inputs/results, and
card metadata. It contains private play history and is not encrypted. Card artwork
and provider credentials are not included. Keep backups somewhere you control.

## Restore

1. Choose **Restore backup…** and select an Alkemyst backup.
2. Alkemyst validates a private copy without modifying the selected file. The
   supported storage schema, SQLite integrity, CHECK constraints, and foreign keys
   must match. Unrelated SQLite files and newer or incompatible schemas are refused.
3. Review the confirmation. **Keep current library** cancels the selection. Merely
   selecting a file never schedules a restore.
4. **Restart and restore** publishes a confirmed request and restarts Alkemyst
   through its normal shutdown path. Capture, API/MCP and simulations stop first.
5. Before services start again, Alkemyst saves the current database under its
   application-data `backups/before-restore-*/alkemyst.sqlite3` directory and applies
   the validated snapshot in a SQLite transaction. Settings reports the outcome
   and displays the safety backup location.

If validation or restoration fails, the current library is retained and the
pending file is quarantined under `failed-restore-*`. If the request cannot be
consumed safely, startup stops rather than risk reapplying it over later captures.
Safety backups remain until you remove them yourself. Unconfirmed temporary
selections are never applied, including after a crash.

The local MCP credential rotates on restart. Recopy its Settings configuration
when reconnecting external clients. Reimporting retained logs uses restored
checkpoints and event identities; it does not duplicate already captured games.

## Recovering when the current database cannot open

Quit all Alkemyst instances first. Preserve the entire dedicated application-data
folder elsewhere, including any SQLite `-wal` and `-shm` files; do not delete them.
Launch Alkemyst with a fresh application-data folder, then restore your known-good
backup from Settings. Keep the original folder until you have verified the
restored versions and history. Installers and stable signed distribution are
tracked separately; this workflow does not imply an installer has been published.

## Verification

Core tests exercise standalone WAL snapshots, cached reads after restore,
immutable versions, capture checkpoints, unchanged source files, and malformed
schemas/data. Native runtime tests exercise cancellation, confirmed restart,
safety backups and quarantine. Browser tests simulate OS dialogs while using the
real Rust application API; they verify keyboard focus, confirmation and WCAG
accessibility. Native smoke tests render the Settings controls through actual IPC.
OS file dialogs still require platform-specific installer verification.
