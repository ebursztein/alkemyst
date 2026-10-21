# Arena regression sample

`arena-game.txt` was derived on 2026-10-21 from an authorized local retained
Arena log. The source was read without modification. Raw logs are never committed.
The pinned parser is manasight-parser 0.7.3 (MIT OR Apache-2.0).

Local validation recovered 12 numbered games in 12 matches and 12 Standard
Ladder deck submissions, each with 60 main-deck cards. One submitted deck ID
contained three different main-deck hashes, with 22 or 25 distinct card IDs.
This is retained history, not evidence that all past games are recoverable.

The fixture retains three submitted card lists and sampled events from one
match: its start, an initial state, representative main-phase states, a mulligan,
a few client actions, a game result and match completion. Source event order is
preserved. It intentionally has gaps and must never be treated as a complete
turn-by-turn replay or used to establish missing-data defaults.

Sanitization reconstructed deck requests with only card lists, a fixed format,
and pseudonymous deck IDs. Other JSON envelopes were recursively reduced to
numeric/boolean/null values, known gameplay enum strings and pseudonymous match
IDs. Names, user IDs, account/session values, request identifiers, timestamps and
unrecognized strings were removed or nulled; nested payload JSON was decoded
before sanitization. No authentication/session events were selected. Card IDs,
quantities, game-state IDs and in-game object IDs retain their gameplay meaning.
The privacy regression test rejects unexpected strings and dynamic keys.

Revalidate local files without exporting raw content:

```sh
cargo run -p alkemyst-core --example inspect_arena -- <local-log-path>
cargo test -p alkemyst-core --test arena_fixtures
```

This command reports counts only. It does not import into SQLite or establish
wins/losses from the local player's perspective. The ingestion adapter must
resolve the local seat, missing states and deck-to-game association explicitly.
