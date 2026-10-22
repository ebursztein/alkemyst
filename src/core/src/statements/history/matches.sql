WITH selected AS (
    SELECT o.match_id,o.ordinal FROM match_capture_order o
    WHERE (?1 IS NULL OR o.ordinal<?1) AND (?4 IS NULL OR o.match_id=?4)
        AND ((?2 IS NULL AND ?3 IS NULL) OR EXISTS(
            SELECT 1 FROM games g LEFT JOIN deck_versions v ON v.id=g.deck_version
            WHERE g.match_id=o.match_id AND (?2 IS NULL OR v.deck_id=?2) AND (?3 IS NULL OR g.deck_version=?3)
        )) AND (?5 IS NULL OR (SELECT mode FROM history_match_facts f WHERE f.id=o.match_id)=?5)
    ORDER BY o.ordinal DESC LIMIT ?6+1
), page AS (SELECT * FROM selected ORDER BY ordinal DESC LIMIT ?6)
SELECT json_object('items',json((SELECT json_group_array(json(summary)) FROM (
    SELECT f.summary FROM page p JOIN history_match_summaries f ON f.id=p.match_id ORDER BY p.ordinal DESC
))), 'next_cursor',CASE WHEN (SELECT count(*) FROM selected)>?6
    THEN printf('h1:%016x',(SELECT min(ordinal) FROM page)) END)
