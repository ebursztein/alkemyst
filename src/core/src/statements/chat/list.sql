WITH selected AS MATERIALIZED (
    SELECT ordinal,id FROM chat_conversations WHERE deleted=0
        AND ordinal<coalesce(?1,9223372036854775807) ORDER BY ordinal DESC LIMIT ?2+1
), page AS MATERIALIZED (SELECT * FROM selected ORDER BY ordinal DESC LIMIT ?2)
SELECT json_object('items',json((SELECT json_group_array(json(d.data) ORDER BY p.ordinal DESC)
    FROM page p JOIN chat_conversation_details d ON d.id=p.id)),
    'next_cursor',CASE WHEN (SELECT count(*) FROM selected)>?2
        THEN printf('c1:%016x',(SELECT min(ordinal) FROM page)) ELSE NULL END)
