SELECT json_object('arena_id', requested.value, 'card', json(coalesce(p.data, f.data)),
    'resolved_by', CASE WHEN p.id IS NOT NULL THEN 'scryfall_arena_id'
        WHEN f.id IS NOT NULL THEN 'arena_asset_printing' ELSE NULL END)
FROM json_each(?1) requested
LEFT JOIN catalog_state s ON s.singleton=1
LEFT JOIN card_printings p ON p.rowid = (
    SELECT candidate.rowid FROM card_printings candidate
    WHERE candidate.snapshot_id=s.active_snapshot AND candidate.arena_id=requested.value
    ORDER BY candidate.lang='en' DESC, candidate.released_at DESC, candidate.id LIMIT 1
)
LEFT JOIN arena_card_aliases alias ON alias.arena_id=requested.value AND p.id IS NULL
LEFT JOIN card_printings f ON f.rowid = (
    SELECT candidate.rowid FROM card_printings candidate
    WHERE p.id IS NULL AND candidate.snapshot_id=s.active_snapshot
        AND json_extract(candidate.data, '$.set')=alias.set_code
        AND json_extract(candidate.data, '$.collector_number')=alias.collector_number
        AND json_extract(candidate.data, '$.name')=alias.name
    ORDER BY candidate.lang='en' DESC, candidate.released_at DESC, candidate.id LIMIT 1
)
ORDER BY CAST(requested.key AS INTEGER)
