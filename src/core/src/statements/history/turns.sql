WITH boundary AS (
    SELECT coalesce(?3,(SELECT max(ordinal) FROM capture_events WHERE game_id=?1),0) AS high
), candidates AS (
    SELECT e.*,(SELECT p.payload FROM capture_events p WHERE p.game_id=e.game_id AND p.state_id=e.state_id
        AND p.kind IN ('state','result') AND p.ordinal<=b.high
        AND json_extract(p.payload,'$.turn_info.turn_number')>0
        ORDER BY CASE p.kind WHEN 'state' THEN 0 ELSE 1 END,p.ordinal LIMIT 1) AS reference_payload
    FROM capture_events e,boundary b WHERE e.game_id=?1 AND e.ordinal<=b.high
), resolved AS MATERIALIZED (
    SELECT *,coalesce(CASE WHEN json_extract(payload,'$.turn_info.turn_number')>0 THEN json_extract(payload,'$.turn_info.turn_number') END,
        json_extract(reference_payload,'$.turn_info.turn_number')) AS known_turn,
        coalesce(nullif(json_extract(payload,'$.turn_info.active_player'),0),json_extract(reference_payload,'$.turn_info.active_player')) AS active,
        coalesce(nullif(json_extract(payload,'$.local_seat'),0),json_extract(reference_payload,'$.local_seat')) AS local_seat,
        coalesce(nullif(json_extract(payload,'$.turn_info.phase'),''),nullif(json_extract(reference_payload,'$.turn_info.phase'),'')) AS phase
    FROM candidates
), grouped AS (
    SELECT known_turn AS number,count(*) AS events,sum(kind='state') AS states,sum(kind='action') AS actions,
        min(ordinal) AS first_ordinal,max(ordinal) AS last_ordinal,
        CASE WHEN count(DISTINCT active)=1 THEN min(active) END AS active_player,
        CASE WHEN count(DISTINCT CASE WHEN active>0 AND local_seat>0 THEN active=local_seat END)=1
            THEN min(CASE WHEN active>0 AND local_seat>0 THEN active=local_seat END) END AS is_local_turn,
        count(phase) AS phase_samples
    FROM resolved WHERE known_turn>coalesce(?2,0) GROUP BY known_turn ORDER BY known_turn LIMIT ?4+1
), page AS (SELECT * FROM grouped ORDER BY number LIMIT ?4), players AS MATERIALIZED (
    SELECT e.known_turn,e.state_id,e.ordinal,json_extract(p.value,'$.lifeTotal') AS life,
        CASE WHEN json_type(p.value,'$.landsPlayedThisTurn')='integer' AND json_extract(p.value,'$.landsPlayedThisTurn')>=0
            AND e.active=e.local_seat THEN json_extract(p.value,'$.landsPlayedThisTurn') END AS lands,
        CASE WHEN json_type(p.value,'$.mulliganCount')='integer' AND json_extract(p.value,'$.mulliganCount')>=0
            THEN json_extract(p.value,'$.mulliganCount') END AS mulligans
    FROM resolved e,json_each(e.payload,'$.players') p
    WHERE coalesce(json_extract(p.value,'$.systemSeatNumber'),json_extract(p.value,'$.controllerSeatId'))=e.local_seat
)
SELECT json_object('found',json(CASE WHEN EXISTS(SELECT 1 FROM games WHERE id=?1) THEN 'true' ELSE 'false' END),
    'game_id',?1,'through_ordinal',(SELECT high FROM boundary),
    'unassigned_events',(SELECT count(*) FROM resolved WHERE known_turn IS NULL),
    'items',json((SELECT json_group_array(json(item)) FROM (
        SELECT json_object('number',number,'events',events,'states',states,'actions',actions,
            'first_ordinal',first_ordinal,'last_ordinal',last_ordinal,'active_player',active_player,
            'is_local_turn',json(CASE is_local_turn WHEN 1 THEN 'true' WHEN 0 THEN 'false' ELSE 'null' END),
            'phase_samples',phase_samples,'phases',json((SELECT json_group_array(phase) FROM (
                SELECT phase FROM resolved r WHERE r.known_turn=page.number AND phase IS NOT NULL GROUP BY phase ORDER BY min(ordinal)
            ))),'land_drops_observed',(SELECT max(lands) FROM players p WHERE p.known_turn=page.number),
            'land_counter_samples',(SELECT count(lands) FROM players p WHERE p.known_turn=page.number),
            'local_life_total',(SELECT life FROM players p WHERE p.known_turn=page.number AND life IS NOT NULL ORDER BY state_id DESC,ordinal DESC LIMIT 1),
            'mulligans',(SELECT max(mulligans) FROM players p WHERE p.known_turn=page.number)) AS item
        FROM page ORDER BY number
    ))),'next_cursor',CASE WHEN (SELECT count(*) FROM grouped)>?4
        THEN printf('t1:%016x:%016x',(SELECT high FROM boundary),(SELECT max(number) FROM page)) END)
