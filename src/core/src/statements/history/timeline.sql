WITH boundary AS (
    SELECT coalesce(?3,(SELECT max(ordinal) FROM capture_events WHERE game_id=?1),0) AS high
), candidates AS (
    SELECT e.*,(SELECT p.payload FROM capture_events p WHERE p.game_id=e.game_id AND p.state_id=e.state_id
        AND p.kind IN ('state','result') AND p.ordinal<=b.high
        AND json_extract(p.payload,'$.turn_info.turn_number')>0
        ORDER BY CASE p.kind WHEN 'state' THEN 0 ELSE 1 END,p.ordinal LIMIT 1) AS reference_payload
    FROM capture_events e,boundary b
    WHERE e.game_id=?1 AND e.ordinal>coalesce(?2,0) AND e.ordinal<=b.high
), resolved AS (
    SELECT *,coalesce(CASE WHEN json_extract(payload,'$.turn_info.turn_number')>0 THEN json_extract(payload,'$.turn_info.turn_number') END,
        json_extract(reference_payload,'$.turn_info.turn_number')) AS known_turn FROM candidates
), selected AS MATERIALIZED (
    SELECT * FROM resolved WHERE (?5 IS NULL OR known_turn=?5) ORDER BY ordinal LIMIT ?4+1
), page AS (SELECT * FROM selected ORDER BY ordinal LIMIT ?4)
SELECT json_object('found',json(CASE WHEN EXISTS(SELECT 1 FROM games WHERE id=?1) THEN 'true' ELSE 'false' END),
    'game_id',?1,'through_ordinal',(SELECT high FROM boundary),
    'items',json((SELECT json_group_array(json(item)) FROM (
        SELECT json_object('ordinal',ordinal,'id',id,'kind',kind,'state_id',state_id,'turn_number',known_turn,
            'phase',coalesce(nullif(json_extract(payload,'$.turn_info.phase'),''),nullif(json_extract(reference_payload,'$.turn_info.phase'),'')),
            'step',coalesce(nullif(json_extract(payload,'$.turn_info.step'),''),nullif(json_extract(reference_payload,'$.turn_info.step'),'')),
            'local_seat',coalesce(json_extract(payload,'$.local_seat'),json_extract(reference_payload,'$.local_seat')),
            'local_time',local_time,'instant_utc',instant_utc,'data',json(payload)) AS item
        FROM page ORDER BY ordinal
    ))),'next_cursor',CASE WHEN (SELECT count(*) FROM selected)>?4
        THEN printf('h1:%016x:%016x',(SELECT high FROM boundary),(SELECT max(ordinal) FROM page)) END)
