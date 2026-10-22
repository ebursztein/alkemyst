WITH selected AS MATERIALIZED (
    SELECT ordinal,data FROM simulation_details
        WHERE ?2 IS NULL AND ordinal<coalesce(?1,9223372036854775807)
    UNION ALL
    SELECT ordinal,data FROM simulation_details
        WHERE version_id=?2 AND ordinal<coalesce(?1,9223372036854775807)
    ORDER BY ordinal DESC LIMIT ?3+1
), page AS MATERIALIZED (SELECT * FROM selected ORDER BY ordinal DESC LIMIT ?3)
SELECT json_object('items',json((SELECT json_group_array(json(data)) FROM page)),
    'next_cursor',CASE WHEN (SELECT count(*) FROM selected)>?3
        THEN printf('s1:%016x',(SELECT min(ordinal) FROM page)) ELSE NULL END)
