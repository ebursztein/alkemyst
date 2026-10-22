CREATE TABLE decks (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    format TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;
CREATE TABLE table_revisions (
    table_name TEXT PRIMARY KEY NOT NULL,
    revision INTEGER NOT NULL CHECK (revision >= 0)
) STRICT;
PRAGMA user_version=1;
