SELECT json_object(
    'snapshot_id', s.active_snapshot,
    'source', CASE WHEN g.id IS NULL THEN NULL
        ELSE json_object('url', g.source_url, 'updated_at', g.updated_at) END,
    'cards', (SELECT count(*) FROM cards WHERE snapshot_id=s.active_snapshot),
    'printings', (SELECT count(*) FROM card_printings WHERE snapshot_id=s.active_snapshot),
    'arena_ids', (SELECT count(DISTINCT arena_id) FROM card_printings
        WHERE snapshot_id=s.active_snapshot AND arena_id IS NOT NULL),
    'arena_aliases', (SELECT count(*) FROM arena_card_aliases),
    'arena_alias_revision', coalesce((SELECT revision FROM table_revisions
        WHERE table_name='arena_card_aliases'), 0)
)
FROM (SELECT 1) AS singleton
LEFT JOIN catalog_state s ON s.singleton=1
LEFT JOIN catalog_snapshots g ON g.id=s.active_snapshot
