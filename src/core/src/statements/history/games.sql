WITH selected AS (
    SELECT o.game_id,o.ordinal FROM game_capture_order o JOIN games g ON g.id=o.game_id
    LEFT JOIN deck_versions v ON v.id=g.deck_version
    WHERE (?1 IS NULL OR o.ordinal<?1) AND (?2 IS NULL OR v.deck_id=?2)
        AND (?3 IS NULL OR g.deck_version=?3) AND (?4 IS NULL OR g.match_id=?4)
        AND (?5 IS NULL OR (SELECT mode FROM history_game_facts f WHERE f.id=g.id)=?5)
    ORDER BY o.ordinal DESC LIMIT ?6+1
), page AS (
    SELECT s.* FROM selected s ORDER BY ordinal DESC LIMIT ?6
)
SELECT json_object('items',json((SELECT json_group_array(json(summary)) FROM (
    SELECT f.summary FROM page p JOIN history_game_summaries f ON f.id=p.game_id ORDER BY p.ordinal DESC
))), 'next_cursor',CASE WHEN (SELECT count(*) FROM selected)>?6
    THEN printf('h1:%016x',(SELECT min(ordinal) FROM page)) END)
