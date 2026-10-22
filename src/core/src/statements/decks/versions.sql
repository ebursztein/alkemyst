WITH selected AS MATERIALIZED (
    SELECT * FROM deck_versions WHERE deck_id=?1 AND sealed=1
        AND ordinal<coalesce(?2,9223372036854775807) ORDER BY ordinal DESC LIMIT ?3+1
), page AS MATERIALIZED (SELECT * FROM selected ORDER BY ordinal DESC LIMIT ?3), totals AS (
    SELECT c.version_id,sum(CASE WHEN c.zone='main' THEN c.quantity ELSE 0 END) AS main_count,
        sum(CASE WHEN c.zone='sideboard' THEN c.quantity ELSE 0 END) AS sideboard_count
    FROM page p JOIN deck_cards c ON c.version_id=p.id GROUP BY c.version_id
)
SELECT json_object('items',json((SELECT json_group_array(json_object(
    'id',p.id,'deck_id',p.deck_id,'ordinal',p.ordinal,'parent_version',p.parent_version,
    'name',p.name,'format',p.format,'origin',p.origin,'created_at',p.created_at,
    'main_count',coalesce(t.main_count,0),'sideboard_count',coalesce(t.sideboard_count,0)
) ORDER BY p.ordinal DESC) FROM page p LEFT JOIN totals t ON t.version_id=p.id)),
    'next_cursor',CASE WHEN (SELECT count(*) FROM selected)>?3
        THEN printf('v1:%016x',(SELECT min(ordinal) FROM page)) ELSE NULL END)
FROM decks WHERE id=?1
