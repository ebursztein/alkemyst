WITH selected AS MATERIALIZED (
    SELECT sequence,id FROM chat_messages WHERE conversation_id=?1 AND sequence>coalesce(?2,0)
        ORDER BY sequence LIMIT ?3+1
), page AS MATERIALIZED (SELECT * FROM selected ORDER BY sequence LIMIT ?3)
SELECT json_object('items',json((SELECT json_group_array(json(d.data) ORDER BY p.sequence)
    FROM page p JOIN chat_message_details d ON d.id=p.id)),
    'next_cursor',CASE WHEN (SELECT count(*) FROM selected)>?3
        THEN printf('m1:%016x',(SELECT max(sequence) FROM page)) ELSE NULL END)
FROM chat_conversations WHERE id=?1 AND deleted=0
