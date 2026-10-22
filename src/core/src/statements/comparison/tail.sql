), grouped AS (
    SELECT identity,zone,min(json_extract(data,'$.name')) AS name,
        sum(CASE WHEN version_id=?1 THEN quantity ELSE 0 END) AS base_quantity,
        sum(CASE WHEN version_id=?2 THEN quantity ELSE 0 END) AS candidate_quantity
    FROM resolved GROUP BY identity,zone
), curves AS (
    SELECT mana_value,
        sum(CASE WHEN version_id=?1 THEN quantity ELSE 0 END) AS base_quantity,
        sum(CASE WHEN version_id=?2 THEN quantity ELSE 0 END) AS candidate_quantity
    FROM resolved WHERE zone='main' AND coalesce(instr(front_type,'Land'),0)=0
    GROUP BY mana_value
), versions AS (
    SELECT v.id,json_object('id',v.id,'deck_id',v.deck_id,'ordinal',v.ordinal,'name',v.name,'format',v.format,
        'main',coalesce(sum(CASE WHEN r.zone='main' THEN r.quantity ELSE 0 END),0),
        'sideboard',coalesce(sum(CASE WHEN r.zone='sideboard' THEN r.quantity ELSE 0 END),0),
        'unresolved_main',coalesce(sum(CASE WHEN r.zone='main' AND r.data IS NULL THEN r.quantity ELSE 0 END),0)
    ) AS data
    FROM deck_versions v LEFT JOIN resolved r ON r.version_id=v.id
    WHERE v.id IN (?1,?2) AND v.sealed=1 GROUP BY v.id
)
SELECT json_object('base',json(b.data),'candidate',json(c.data),
    'catalog_snapshot',s.active_snapshot,'catalog_updated_at',snapshot.updated_at,
    'cards',json((SELECT json_group_array(json_object(
        'identity',identity,'zone',zone,'name',name,'base_quantity',base_quantity,
        'candidate_quantity',candidate_quantity,'delta',candidate_quantity-base_quantity
    )) FROM (SELECT * FROM grouped WHERE base_quantity!=candidate_quantity ORDER BY zone,identity))),
    'curve',json((SELECT json_group_array(json_object(
        'mana_value',mana_value,'base_quantity',base_quantity,'candidate_quantity',candidate_quantity,
        'delta',candidate_quantity-base_quantity
    )) FROM (SELECT * FROM curves ORDER BY mana_value IS NULL,mana_value)))
)
FROM versions b JOIN versions c ON c.id=?2
LEFT JOIN catalog_state s ON s.singleton=1
LEFT JOIN catalog_snapshots snapshot ON snapshot.id=s.active_snapshot
WHERE b.id=?1
