CREATE INDEX capture_events_kind ON capture_events(game_id, kind, state_id);
CREATE INDEX capture_events_match_kind ON capture_events(match_id, kind);
CREATE INDEX games_version ON games(deck_version, id);
CREATE TABLE game_capture_order (
    game_id TEXT PRIMARY KEY NOT NULL REFERENCES games(id),
    ordinal INTEGER UNIQUE NOT NULL
) STRICT;
CREATE TABLE match_capture_order (
    match_id TEXT PRIMARY KEY NOT NULL REFERENCES matches(id),
    ordinal INTEGER UNIQUE NOT NULL
) STRICT;
INSERT INTO game_capture_order SELECT game_id,min(ordinal)
FROM capture_events WHERE game_id IS NOT NULL GROUP BY game_id;
INSERT INTO match_capture_order SELECT match_id,min(ordinal)
FROM capture_events WHERE match_id IS NOT NULL GROUP BY match_id;
CREATE TRIGGER capture_first_observation AFTER INSERT ON capture_events
BEGIN
    INSERT INTO game_capture_order SELECT new.game_id,new.ordinal WHERE new.game_id IS NOT NULL
        ON CONFLICT(game_id) DO NOTHING;
    INSERT INTO match_capture_order SELECT new.match_id,new.ordinal WHERE new.match_id IS NOT NULL
        ON CONFLICT(match_id) DO NOTHING;
END;
