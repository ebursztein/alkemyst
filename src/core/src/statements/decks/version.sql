SELECT json_object('id', v.id, 'deck_id', v.deck_id, 'ordinal', v.ordinal,
    'parent_version', v.parent_version, 'name', v.name, 'format', v.format,
    'origin', v.origin, 'created_at', v.created_at,
    'cards', json((SELECT json_group_array(json_object('reference', reference,
        'quantity', quantity, 'zone', zone)) FROM (
        SELECT reference, quantity, zone FROM deck_cards WHERE version_id=v.id
        ORDER BY CASE zone WHEN 'main' THEN 0 WHEN 'sideboard' THEN 1 WHEN 'command' THEN 2 ELSE 3 END, reference
    )))
)
FROM deck_versions v
WHERE v.sealed=1 AND v.id=coalesce(?1, (SELECT result_id FROM mutation_receipts WHERE key=?2))
