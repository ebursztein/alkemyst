)
SELECT json_object('version_id',v.id,'deck_id',v.deck_id,'name',v.name,'format',v.format,
    'catalog_snapshot',s.active_snapshot,'catalog_updated_at',snapshot.updated_at,
    'counts',json((SELECT json_object(
        'main',coalesce(sum(CASE WHEN zone='main' THEN quantity ELSE 0 END),0),
        'sideboard',coalesce(sum(CASE WHEN zone='sideboard' THEN quantity ELSE 0 END),0),
        'lands',coalesce(sum(CASE WHEN zone='main' AND instr(front_type,'Land')>0 THEN quantity ELSE 0 END),0),
        'equipment',coalesce(sum(CASE WHEN zone='main' AND instr(front_type,'Equipment')>0 THEN quantity ELSE 0 END),0),
        'creatures',coalesce(sum(CASE WHEN zone='main' AND instr(front_type,'Creature')>0 THEN quantity ELSE 0 END),0),
        'legendary',coalesce(sum(CASE WHEN zone='main' AND instr(front_type,'Legendary')>0 THEN quantity ELSE 0 END),0),
        'unresolved_main',coalesce(sum(CASE WHEN zone='main' AND data IS NULL THEN quantity ELSE 0 END),0)
    ) FROM resolved)),
    'curve',json((SELECT json_group_array(json_object('mana_value',mana_value,'quantity',quantity)) FROM (
        SELECT mana_value,sum(quantity) AS quantity FROM resolved
        WHERE zone='main' AND coalesce(instr(front_type,'Land'),0)=0
        GROUP BY mana_value ORDER BY mana_value IS NULL,mana_value
    ))),
    'cards',json((SELECT json_group_array(json_object('reference',reference,'quantity',quantity,'zone',zone,'data',json(data)))
        FROM (SELECT reference,quantity,zone,data FROM resolved ORDER BY zone,reference)))
)
FROM deck_versions v
LEFT JOIN catalog_state s ON s.singleton=1
LEFT JOIN catalog_snapshots snapshot ON snapshot.id=s.active_snapshot
WHERE v.id=?1 AND v.sealed=1
