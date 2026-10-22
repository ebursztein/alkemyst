SELECT json_object(
    'oracle_id', c.oracle_id, 'id', c.id, 'name', c.name,
    'mana_value', c.mana_value, 'type_line', c.type_line, 'oracle_text', c.oracle_text,
    'mana_cost', json_extract(c.data, '$.mana_cost'),
    'color_identity', json(coalesce(json_extract(c.data, '$.color_identity'), '[]')),
    'image_uri', coalesce(json_extract(c.data, '$.image_uris.normal'),
        json_extract(c.data, '$.card_faces[0].image_uris.normal')),
    'arena_available', json(CASE c.arena_available WHEN 1 THEN 'true' ELSE 'false' END)
)
FROM cards c JOIN catalog_state s ON c.snapshot_id=s.active_snapshot
WHERE s.singleton=1 AND (?1 IS NULL OR c.rowid IN (SELECT rowid FROM cards_fts WHERE cards_fts MATCH ?1))
    AND (?2 IS NULL OR instr(lower(c.type_line), lower(?2)) > 0)
    AND (?3 IS NULL OR (c.identity_mask & ?3) = c.identity_mask)
    AND (?4 IS NULL OR json_extract(c.legalities, '$.' || ?4) = 'legal')
    AND (?5 IS NULL OR c.mana_value >= ?5)
    AND (?6 IS NULL OR c.mana_value <= ?6)
    AND (?7=0 OR c.arena_available=1)
    AND (?8 IS NULL OR c.legendary=?8)
    AND (?9 IS NULL OR c.name COLLATE NOCASE > ?9 COLLATE NOCASE
        OR (c.name COLLATE NOCASE = ?9 COLLATE NOCASE AND c.oracle_id > ?10))
ORDER BY c.name COLLATE NOCASE, c.oracle_id
LIMIT ?11
