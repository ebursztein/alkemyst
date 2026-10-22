CREATE TABLE capture_sources (
    id TEXT PRIMARY KEY NOT NULL,
    sequence INTEGER NOT NULL,
    checkpoint TEXT NOT NULL CHECK(json_valid(checkpoint))
) STRICT;
CREATE TABLE arena_decks (
    alias TEXT PRIMARY KEY NOT NULL,
    deck_id TEXT NOT NULL REFERENCES decks(id)
) STRICT;
CREATE TABLE matches (
    id TEXT PRIMARY KEY NOT NULL
) STRICT;
CREATE TABLE games (
    id TEXT PRIMARY KEY NOT NULL,
    match_id TEXT NOT NULL REFERENCES matches(id),
    number INTEGER NOT NULL CHECK(number > 0),
    deck_version TEXT REFERENCES deck_versions(id),
    UNIQUE(match_id, number)
) STRICT;
CREATE TABLE capture_events (
    ordinal INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT UNIQUE NOT NULL,
    match_id TEXT REFERENCES matches(id),
    game_id TEXT REFERENCES games(id),
    kind TEXT NOT NULL,
    state_id INTEGER,
    payload TEXT NOT NULL CHECK(json_valid(payload)),
    local_time TEXT,
    instant_utc TEXT
) STRICT;
CREATE INDEX capture_events_game ON capture_events(game_id, ordinal);
CREATE INDEX capture_events_state ON capture_events(game_id, state_id, kind);
CREATE TRIGGER capture_events_no_update BEFORE UPDATE ON capture_events
BEGIN SELECT RAISE(ABORT, 'captured events are immutable'); END;
CREATE TRIGGER capture_events_no_delete BEFORE DELETE ON capture_events
BEGIN SELECT RAISE(ABORT, 'captured events are immutable'); END;
CREATE TRIGGER games_no_update BEFORE UPDATE ON games
BEGIN SELECT RAISE(ABORT, 'captured games are immutable'); END;
CREATE TRIGGER games_no_delete BEFORE DELETE ON games
BEGIN SELECT RAISE(ABORT, 'captured games are immutable'); END;
CREATE TRIGGER matches_no_update BEFORE UPDATE ON matches
BEGIN SELECT RAISE(ABORT, 'captured matches are immutable'); END;
CREATE TRIGGER matches_no_delete BEFORE DELETE ON matches
BEGIN SELECT RAISE(ABORT, 'captured matches are immutable'); END;
PRAGMA user_version=6;
