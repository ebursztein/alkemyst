CREATE TABLE mutation_receipts (
    key TEXT PRIMARY KEY NOT NULL,
    operation TEXT NOT NULL,
    body TEXT NOT NULL CHECK (json_valid(body)),
    result_id TEXT NOT NULL
) STRICT;
CREATE TABLE deck_versions (
    id TEXT PRIMARY KEY NOT NULL,
    deck_id TEXT NOT NULL REFERENCES decks(id),
    ordinal INTEGER NOT NULL CHECK (ordinal > 0),
    parent_version TEXT,
    name TEXT NOT NULL,
    format TEXT NOT NULL,
    origin TEXT NOT NULL CHECK (origin IN ('user', 'agent', 'arena', 'import')),
    created_at INTEGER NOT NULL,
    sealed INTEGER NOT NULL DEFAULT 0 CHECK (sealed IN (0, 1)),
    UNIQUE (deck_id, ordinal),
    UNIQUE (id, deck_id),
    FOREIGN KEY (parent_version, deck_id) REFERENCES deck_versions(id, deck_id)
) STRICT;
CREATE TABLE deck_cards (
    version_id TEXT NOT NULL REFERENCES deck_versions(id),
    reference TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity BETWEEN 1 AND 250),
    zone TEXT NOT NULL CHECK (zone IN ('main', 'sideboard', 'command', 'companion')),
    PRIMARY KEY (version_id, zone, reference)
) STRICT;
CREATE TRIGGER deck_versions_immutable BEFORE UPDATE ON deck_versions
WHEN old.sealed=1 OR new.sealed!=1 OR new.id IS NOT old.id
    OR new.deck_id IS NOT old.deck_id OR new.ordinal IS NOT old.ordinal
    OR new.parent_version IS NOT old.parent_version OR new.name IS NOT old.name
    OR new.format IS NOT old.format OR new.origin IS NOT old.origin
    OR new.created_at IS NOT old.created_at
BEGIN SELECT RAISE(ABORT, 'deck versions are immutable'); END;
CREATE TRIGGER deck_versions_no_delete BEFORE DELETE ON deck_versions
BEGIN SELECT RAISE(ABORT, 'deck versions are immutable'); END;
CREATE TRIGGER deck_cards_no_insert BEFORE INSERT ON deck_cards
WHEN (SELECT sealed FROM deck_versions WHERE id=new.version_id)=1
BEGIN SELECT RAISE(ABORT, 'deck versions are immutable'); END;
CREATE TRIGGER deck_cards_no_update BEFORE UPDATE ON deck_cards
BEGIN SELECT RAISE(ABORT, 'deck versions are immutable'); END;
CREATE TRIGGER deck_cards_no_delete BEFORE DELETE ON deck_cards
BEGIN SELECT RAISE(ABORT, 'deck versions are immutable'); END;
CREATE TABLE notes (
    id TEXT PRIMARY KEY NOT NULL,
    deck_id TEXT NOT NULL REFERENCES decks(id),
    version_id TEXT,
    body TEXT NOT NULL,
    origin TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (version_id, deck_id) REFERENCES deck_versions(id, deck_id)
) STRICT;
CREATE INDEX notes_by_deck ON notes(deck_id, id);
CREATE INDEX card_face_name ON cards(snapshot_id, json_extract(data, '$.card_faces[0].name') COLLATE NOCASE);
PRAGMA user_version=4;
