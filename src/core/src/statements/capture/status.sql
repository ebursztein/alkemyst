SELECT json_object(
    'sources',(SELECT count(*) FROM capture_sources),
    'matches',(SELECT count(*) FROM matches),
    'games',(SELECT count(*) FROM games),
    'events',(SELECT count(*) FROM capture_events),
    'deck_versions',(SELECT count(*) FROM deck_versions WHERE origin='arena'),
    'unknown_results',(
        SELECT count(*) FROM games g WHERE (
            SELECT count(DISTINCT json_extract(e.payload,'$.outcome'))
            FROM capture_events e WHERE e.game_id=g.id AND e.kind='result'
        ) != 1
    )
)
