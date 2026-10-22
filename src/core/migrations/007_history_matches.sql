CREATE VIEW history_match_summaries AS
SELECT f.*,json_object('id',id,'first_ordinal',first_ordinal,'mode',mode,'outcome',outcome,
    'completed',json(CASE WHEN completed THEN 'true' ELSE 'false' END),
    'game_count',(SELECT count(*) FROM games g WHERE g.match_id=f.id),
    'games',json((SELECT json_group_array(json(summary)) FROM (
        SELECT g.summary FROM history_game_summaries g WHERE g.match_id=f.id ORDER BY g.number
    )))) AS summary
FROM history_match_facts f;
