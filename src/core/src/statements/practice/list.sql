WITH selected AS MATERIALIZED (
    SELECT ordinal,id FROM practice_sessions
        WHERE ?2 IS NULL AND ordinal<coalesce(?1,9223372036854775807)
    UNION ALL
    SELECT ordinal,id FROM practice_sessions
        WHERE version_id=?2 AND ordinal<coalesce(?1,9223372036854775807)
    ORDER BY ordinal DESC LIMIT ?3+1
), page AS MATERIALIZED (SELECT * FROM selected ORDER BY ordinal DESC LIMIT ?3)
SELECT json_object('items',json((SELECT json_group_array(json_object(
    'id',s.id,'version_id',s.version_id,'name',json_extract(s.snapshot,'$.name'),
    'revision',p.revision,'created_at',s.created_at,'phase',json_extract(p.state,'$.phase'),
    'turn',json_extract(p.state,'$.turn'),'mulligans',json_extract(p.state,'$.mulligans'),
    'decked',json(CASE WHEN json_extract(p.state,'$.decked')=1 THEN 'true' ELSE 'false' END)
) ORDER BY a.ordinal DESC) FROM page a JOIN practice_sessions s ON s.id=a.id JOIN practice_steps p
    ON p.session_id=s.id AND p.revision=(SELECT max(revision) FROM practice_steps WHERE session_id=s.id))),
    'next_cursor',CASE WHEN (SELECT count(*) FROM selected)>?3
        THEN printf('p1:%016x',(SELECT min(ordinal) FROM page)) ELSE NULL END)
