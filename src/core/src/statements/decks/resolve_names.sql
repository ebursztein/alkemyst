SELECT json_object('reference', CASE WHEN p.id IS NOT NULL THEN 'scryfall:' || p.id ELSE NULL END)
FROM json_each(?1) requested
LEFT JOIN catalog_state s ON s.singleton=1
LEFT JOIN card_printings p ON p.rowid=CASE
    WHEN json_extract(requested.value, '$.set_code') IS NOT NULL THEN (
        SELECT candidate.rowid FROM card_printings candidate
        WHERE candidate.snapshot_id=s.active_snapshot
            AND json_extract(candidate.data, '$.set')=json_extract(requested.value, '$.set_code')
            AND json_extract(candidate.data, '$.collector_number')=json_extract(requested.value, '$.collector_number')
            AND (json_extract(candidate.data, '$.name')=json_extract(requested.value, '$.name') COLLATE NOCASE
                OR json_extract(candidate.data, '$.card_faces[0].name')=json_extract(requested.value, '$.name') COLLATE NOCASE)
        ORDER BY candidate.lang='en' DESC, candidate.id LIMIT 1
    )
    ELSE (
        SELECT candidate.rowid FROM cards c JOIN card_printings candidate
            ON candidate.snapshot_id=c.snapshot_id AND candidate.id=c.id
        WHERE c.snapshot_id=s.active_snapshot AND c.rowid=coalesce(
            (SELECT rowid FROM cards WHERE snapshot_id=s.active_snapshot AND name=json_extract(requested.value,'$.name') COLLATE NOCASE LIMIT 1),
            (SELECT rowid FROM cards WHERE snapshot_id=s.active_snapshot AND json_extract(data,'$.card_faces[0].name')=json_extract(requested.value,'$.name') COLLATE NOCASE LIMIT 1)
        )
    ) END
ORDER BY CAST(requested.key AS INTEGER)
