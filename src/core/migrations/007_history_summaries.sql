CREATE VIEW history_game_quality AS
SELECT f.*,
    json((SELECT json_group_array(reason) FROM (
        SELECT 'initial_state' AS reason WHERE NOT initial_state
        UNION ALL SELECT 'terminal_result' WHERE NOT terminal
        UNION ALL SELECT 'state_gap' WHERE state_gap
        UNION ALL SELECT 'unverifiable_chain' WHERE unverifiable_chain
        UNION ALL SELECT 'truncated' WHERE truncated
        UNION ALL SELECT 'connection' WHERE interrupted
        UNION ALL SELECT 'deck_version' WHERE deck_version IS NULL
        UNION ALL SELECT 'local_seat' WHERE local_seat IS NULL
        UNION ALL SELECT 'conflicting_result' WHERE conflicting_result
        UNION ALL SELECT 'outcome' WHERE terminal AND outcome IS NULL AND NOT conflicting_result
    ))) AS missing_data
FROM history_game_facts f;
CREATE VIEW history_game_summaries AS
SELECT q.*,json_array_length(missing_data)=0 AS capture_complete,
    json_object('id',id,'match_id',match_id,'number',number,
        'deck_id',deck_id,'deck_version',deck_version,'deck_name',deck_name,'format',format,
        'first_ordinal',first_ordinal,'events',events,'outcome',outcome,'mode',mode,
        'capture_complete',json(CASE WHEN json_array_length(missing_data)=0 THEN 'true' ELSE 'false' END),
        'missing_data',json(missing_data),'local_seat',local_seat,
        'on_play',json(CASE on_play WHEN 1 THEN 'true' WHEN 0 THEN 'false' ELSE 'null' END),
        'turns_observed',turns_observed,'last_turn',last_turn,'mulligans',mulligans,
        'mulligan_actions_observed',mulligan_actions_observed,'land_drops_observed',land_drops_observed,
        'land_turn_samples',land_turn_samples,'local_time',local_time,'match_instant_utc',match_instant_utc) AS summary
FROM history_game_quality q;
PRAGMA user_version=7;
