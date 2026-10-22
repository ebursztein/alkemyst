CREATE TABLE arena_card_aliases (
    arena_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    set_code TEXT NOT NULL,
    collector_number TEXT NOT NULL
) STRICT;
CREATE INDEX card_set_number ON card_printings(
    snapshot_id, json_extract(data, '$.set'), json_extract(data, '$.collector_number'),
    lang, released_at DESC, id
);
PRAGMA user_version=3;
