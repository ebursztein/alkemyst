INSERT INTO cards(snapshot_id, oracle_id, id, name, mana_value, identity_mask,
    type_line, oracle_text, legendary, arena_available, legalities, data)
SELECT snapshot_id, oracle_id, id, json_extract(data, '$.name'),
    json_extract(data, '$.cmc'),
    (SELECT coalesce(sum(CASE value WHEN 'W' THEN 1 WHEN 'U' THEN 2
        WHEN 'B' THEN 4 WHEN 'R' THEN 8 WHEN 'G' THEN 16 ELSE 0 END), 0)
        FROM json_each(data, '$.color_identity')),
    coalesce(json_extract(data, '$.type_line'), ''),
    coalesce(json_extract(data, '$.oracle_text'),
        (SELECT group_concat(json_extract(value, '$.oracle_text'), char(10))
            FROM json_each(data, '$.card_faces')), ''),
    coalesce(json_extract(data, '$.type_line') LIKE '%Legendary%', 0),
    has_arena, coalesce(json_extract(data, '$.legalities'), '{}'), data
FROM (
    SELECT p.*, row_number() OVER (
        PARTITION BY oracle_id ORDER BY (lang='en') DESC, released_at DESC, id
    ) AS preference,
    max(arena_id IS NOT NULL) OVER (PARTITION BY oracle_id) AS has_arena
    FROM card_printings p WHERE snapshot_id=?1
)
WHERE preference=1
