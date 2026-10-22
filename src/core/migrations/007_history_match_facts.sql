CREATE VIEW history_match_results AS
SELECT e.match_id,CASE
    WHEN json_extract(r.value,'$.result')='ResultType_Draw' THEN 'draw'
    WHEN json_extract(r.value,'$.result')='ResultType_WinLoss'
        AND json_extract(r.value,'$.winning_team_id')>0 AND json_extract(e.payload,'$.local_team')>0
    THEN CASE WHEN json_extract(r.value,'$.winning_team_id')=json_extract(e.payload,'$.local_team')
        THEN 'won' ELSE 'lost' END END AS outcome
FROM capture_events e,json_each(e.payload,'$.game_results') r
WHERE e.kind='match_end' AND json_extract(r.value,'$.scope')='MatchScope_Match';
CREATE VIEW history_match_facts AS
SELECT m.id,o.ordinal AS first_ordinal,
    EXISTS(SELECT 1 FROM capture_events e WHERE e.match_id=m.id AND e.kind='match_end') AS completed,
    (SELECT CASE WHEN count(DISTINCT r.outcome)=1 THEN min(r.outcome) END
        FROM history_match_results r WHERE r.match_id=m.id) AS outcome,
    (SELECT CASE WHEN count(DISTINCT json_extract(e.payload,'$.best_of'))=1
        THEN CASE min(json_extract(e.payload,'$.best_of')) WHEN 1 THEN 'bo1' WHEN 3 THEN 'bo3' ELSE 'unknown' END
        ELSE 'unknown' END FROM capture_events e WHERE e.match_id=m.id
        AND json_extract(e.payload,'$.best_of') IN (1,3)) AS mode
FROM matches m JOIN match_capture_order o ON o.match_id=m.id;
