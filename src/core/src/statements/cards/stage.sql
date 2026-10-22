INSERT INTO card_printings(snapshot_id, id, oracle_id, arena_id, lang, released_at, data)
SELECT ?1, json_extract(value, '$.id'),
    coalesce(json_extract(value, '$.oracle_id'), json_extract(value, '$.id')),
    json_extract(value, '$.arena_id'), coalesce(json_extract(value, '$.lang'), 'en'),
    coalesce(json_extract(value, '$.released_at'), ''), value
FROM json_each(?2)
WHERE EXISTS (SELECT 1 FROM catalog_snapshots WHERE id=?1 AND state='staging')
