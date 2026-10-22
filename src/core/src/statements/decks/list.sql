SELECT json_object('id', d.id, 'name', v.name, 'format', v.format,
    'latest_version', v.id,
    'version_count', (SELECT count(*) FROM deck_versions WHERE deck_id=d.id AND sealed=1),
    'main_count', (SELECT coalesce(sum(quantity),0) FROM deck_cards WHERE version_id=v.id AND zone='main'))
FROM decks d JOIN deck_versions v ON v.deck_id=d.id
    AND v.ordinal=(SELECT max(ordinal) FROM deck_versions WHERE deck_id=d.id AND sealed=1)
WHERE (?1 IS NULL OR d.id>?1) AND v.sealed=1
ORDER BY d.id LIMIT ?2
