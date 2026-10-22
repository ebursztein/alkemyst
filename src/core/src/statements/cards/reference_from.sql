FROM refs r
    LEFT JOIN catalog_state s ON s.singleton=1
    LEFT JOIN card_printings p ON p.rowid=CASE
        WHEN r.reference LIKE 'scryfall:%' THEN (
            SELECT rowid FROM card_printings WHERE snapshot_id=s.active_snapshot AND id=substr(r.reference,10)
        )
        WHEN r.reference LIKE 'oracle:%' THEN (
            SELECT candidate.rowid FROM cards c JOIN card_printings candidate
                ON candidate.snapshot_id=c.snapshot_id AND candidate.id=c.id
            WHERE c.snapshot_id=s.active_snapshot AND c.oracle_id=substr(r.reference,8)
        )
        WHEN r.reference LIKE 'arena:%' THEN coalesce(
            (SELECT rowid FROM card_printings WHERE snapshot_id=s.active_snapshot AND arena_id=CAST(substr(r.reference,7) AS INTEGER)
                ORDER BY lang='en' DESC, released_at DESC, id LIMIT 1),
            (SELECT candidate.rowid FROM arena_card_aliases alias JOIN card_printings candidate
                ON candidate.snapshot_id=s.active_snapshot
                AND json_extract(candidate.data,'$.set')=alias.set_code
                AND json_extract(candidate.data,'$.collector_number')=alias.collector_number
                AND json_extract(candidate.data,'$.name')=alias.name
                WHERE alias.arena_id=CAST(substr(r.reference,7) AS INTEGER)
                ORDER BY candidate.lang='en' DESC, candidate.id LIMIT 1)
        ) END
