CREATE TABLE catalog_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    source_url TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('staging', 'active', 'retired'))
) STRICT;
CREATE TABLE catalog_state (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    active_snapshot TEXT NOT NULL REFERENCES catalog_snapshots(id)
) STRICT;
CREATE TABLE card_printings (
    snapshot_id TEXT NOT NULL REFERENCES catalog_snapshots(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    oracle_id TEXT NOT NULL,
    arena_id INTEGER,
    lang TEXT NOT NULL,
    released_at TEXT NOT NULL,
    data TEXT NOT NULL CHECK (json_valid(data)),
    PRIMARY KEY (snapshot_id, id)
) STRICT;
CREATE INDEX card_arena ON card_printings(snapshot_id, arena_id, lang, released_at DESC, id);
CREATE INDEX card_oracle ON card_printings(snapshot_id, oracle_id, lang, released_at DESC, id);
CREATE TABLE cards (
    snapshot_id TEXT NOT NULL REFERENCES catalog_snapshots(id) ON DELETE CASCADE,
    oracle_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE,
    mana_value REAL,
    identity_mask INTEGER NOT NULL,
    type_line TEXT NOT NULL,
    oracle_text TEXT NOT NULL,
    legendary INTEGER NOT NULL,
    arena_available INTEGER NOT NULL,
    legalities TEXT NOT NULL CHECK (json_valid(legalities)),
    data TEXT NOT NULL CHECK (json_valid(data)),
    UNIQUE (snapshot_id, oracle_id)
) STRICT;
CREATE INDEX card_order ON cards(snapshot_id, name, oracle_id);
CREATE VIRTUAL TABLE cards_fts USING fts5(
    name, oracle_text, type_line,
    content='cards', content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER cards_insert AFTER INSERT ON cards BEGIN
    INSERT INTO cards_fts(rowid, name, oracle_text, type_line)
    VALUES (new.rowid, new.name, new.oracle_text, new.type_line);
END;
CREATE TRIGGER cards_delete AFTER DELETE ON cards BEGIN
    INSERT INTO cards_fts(cards_fts, rowid, name, oracle_text, type_line)
    VALUES ('delete', old.rowid, old.name, old.oracle_text, old.type_line);
END;
PRAGMA user_version=2;
