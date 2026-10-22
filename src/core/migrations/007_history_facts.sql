CREATE VIEW history_game_facts AS
SELECT g.id,g.match_id,g.number,g.deck_version,v.deck_id,v.name AS deck_name,v.format,
    o.ordinal AS first_ordinal,
    (SELECT count(*) FROM capture_events e WHERE e.game_id=g.id) AS events,
    (SELECT CASE WHEN count(DISTINCT json_extract(e.payload,'$.outcome'))=1
        THEN min(json_extract(e.payload,'$.outcome')) END
        FROM capture_events e WHERE e.game_id=g.id AND e.kind='result') AS outcome,
    (SELECT count(DISTINCT json_extract(e.payload,'$.outcome'))>1
        FROM capture_events e WHERE e.game_id=g.id AND e.kind='result') AS conflicting_result,
    EXISTS(SELECT 1 FROM capture_events e WHERE e.game_id=g.id AND e.kind='result') AS terminal,
    EXISTS(SELECT 1 FROM capture_events e WHERE e.game_id=g.id AND e.kind='state'
        AND e.state_id=1 AND json_extract(e.payload,'$.game_state_type')='GameStateType_Full') AS initial_state,
    EXISTS(SELECT 1 FROM capture_events e WHERE e.game_id=g.id AND e.kind IN ('state','result')
        AND json_extract(e.payload,'$.prev_game_state_id')>0
        AND (json_extract(e.payload,'$.prev_game_state_id')>=e.state_id OR NOT EXISTS(
            SELECT 1 FROM capture_events p WHERE p.game_id=g.id
            AND p.kind IN ('state','result') AND p.state_id=json_extract(e.payload,'$.prev_game_state_id')
        ))) AS state_gap,
    EXISTS(SELECT 1 FROM capture_events e WHERE e.game_id=g.id AND e.kind IN ('state','result')
        AND (e.kind='result' OR json_extract(e.payload,'$.type') IN ('game_state_message','queued_game_state_message'))
        AND (e.state_id IS NULL OR (e.state_id>1 AND coalesce(json_extract(e.payload,'$.prev_game_state_id'),0)<=0))) AS unverifiable_chain,
    EXISTS(SELECT 1 FROM capture_events e WHERE e.match_id=g.match_id AND e.kind='truncated'
        AND (e.game_id=g.id OR e.game_id IS NULL)) AS truncated,
    EXISTS(SELECT 1 FROM capture_events e WHERE e.match_id=g.match_id AND e.kind='connection'
        AND (e.game_id=g.id OR e.game_id IS NULL) AND json_extract(e.payload,'$.capture_interrupted')=1
        AND e.ordinal<coalesce((SELECT min(t.ordinal) FROM capture_events t
            WHERE t.game_id=g.id AND t.kind='result'),9223372036854775807)) AS interrupted,
    (SELECT CASE WHEN count(DISTINCT json_extract(e.payload,'$.local_seat'))=1
        THEN min(json_extract(e.payload,'$.local_seat')) END FROM capture_events e
        WHERE e.game_id=g.id AND json_extract(e.payload,'$.local_seat')>0) AS local_seat,
    (SELECT CASE WHEN count(DISTINCT (json_extract(e.payload,'$.turn_info.active_player')=json_extract(e.payload,'$.local_seat')))=1
        THEN min(json_extract(e.payload,'$.turn_info.active_player')=json_extract(e.payload,'$.local_seat')) END
        FROM capture_events e WHERE e.game_id=g.id AND json_extract(e.payload,'$.turn_info.turn_number')=1
        AND json_extract(e.payload,'$.local_seat')>0 AND json_extract(e.payload,'$.turn_info.active_player')>0) AS on_play,
    (SELECT CASE WHEN count(DISTINCT json_extract(e.payload,'$.best_of'))=1
        THEN CASE min(json_extract(e.payload,'$.best_of')) WHEN 1 THEN 'bo1' WHEN 3 THEN 'bo3' ELSE 'unknown' END
        ELSE 'unknown' END FROM capture_events e WHERE e.match_id=g.match_id
        AND json_extract(e.payload,'$.best_of') IN (1,3)) AS mode,
    (SELECT count(DISTINCT json_extract(e.payload,'$.turn_info.turn_number')) FROM capture_events e
        WHERE e.game_id=g.id AND json_extract(e.payload,'$.turn_info.turn_number')>0) AS turns_observed,
    (SELECT max(json_extract(e.payload,'$.turn_info.turn_number')) FROM capture_events e
        WHERE e.game_id=g.id AND json_extract(e.payload,'$.turn_info.turn_number')>0) AS last_turn,
    (SELECT max(json_extract(p.value,'$.mulliganCount')) FROM capture_events e,json_each(e.payload,'$.players') p
        WHERE e.game_id=g.id AND coalesce(json_extract(p.value,'$.systemSeatNumber'),json_extract(p.value,'$.controllerSeatId'))=json_extract(e.payload,'$.local_seat')
        AND json_type(p.value,'$.mulliganCount')='integer' AND json_extract(p.value,'$.mulliganCount')>=0) AS mulligans,
    (SELECT count(*) FROM capture_events e WHERE e.game_id=g.id AND e.kind='action'
        AND json_extract(e.payload,'$.decision')='mulligan') AS mulligan_actions_observed,
    (SELECT sum(drops) FROM (
        SELECT max(json_extract(p.value,'$.landsPlayedThisTurn')) AS drops
        FROM capture_events e,json_each(e.payload,'$.players') p WHERE e.game_id=g.id
        AND coalesce(json_extract(p.value,'$.systemSeatNumber'),json_extract(p.value,'$.controllerSeatId'))=json_extract(e.payload,'$.local_seat')
        AND json_extract(e.payload,'$.turn_info.active_player')=json_extract(e.payload,'$.local_seat')
        AND json_extract(e.payload,'$.turn_info.turn_number')>0
        AND json_type(p.value,'$.landsPlayedThisTurn')='integer' AND json_extract(p.value,'$.landsPlayedThisTurn')>=0
        GROUP BY json_extract(e.payload,'$.turn_info.turn_number')
    )) AS land_drops_observed,
    (SELECT count(DISTINCT json_extract(e.payload,'$.turn_info.turn_number'))
        FROM capture_events e,json_each(e.payload,'$.players') p WHERE e.game_id=g.id
        AND coalesce(json_extract(p.value,'$.systemSeatNumber'),json_extract(p.value,'$.controllerSeatId'))=json_extract(e.payload,'$.local_seat')
        AND json_extract(e.payload,'$.turn_info.active_player')=json_extract(e.payload,'$.local_seat')
        AND json_extract(e.payload,'$.turn_info.turn_number')>0
        AND json_type(p.value,'$.landsPlayedThisTurn')='integer' AND json_extract(p.value,'$.landsPlayedThisTurn')>=0) AS land_turn_samples,
    (SELECT e.local_time FROM capture_events e WHERE e.game_id=g.id AND e.local_time IS NOT NULL ORDER BY e.ordinal LIMIT 1) AS local_time,
    (SELECT e.instant_utc FROM capture_events e WHERE e.match_id=g.match_id AND e.instant_utc IS NOT NULL ORDER BY e.ordinal LIMIT 1) AS match_instant_utc
FROM games g JOIN game_capture_order o ON o.game_id=g.id
LEFT JOIN deck_versions v ON v.id=g.deck_version;
