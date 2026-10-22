WITH selected AS MATERIALIZED (
    SELECT id,match_id,outcome,mode,capture_complete,on_play,mulligans,land_drops_observed,land_turn_samples
    FROM history_game_summaries
    WHERE (?1 IS NULL OR deck_id=?1) AND (?2 IS NULL OR deck_version=?2)
        AND (?3 IS NULL OR mode=?3) AND (?4=0 OR capture_complete)
), totals AS (
    SELECT count(*) AS games,count(DISTINCT match_id) AS matches,count(outcome) AS known,
        coalesce(sum(outcome='won'),0) AS wins,coalesce(sum(outcome='lost'),0) AS losses,
        coalesce(sum(outcome='draw'),0) AS draws,coalesce(sum(NOT capture_complete),0) AS partial,
        coalesce(sum(on_play=1),0) AS on_play,coalesce(sum(on_play=0),0) AS on_draw,count(on_play) AS play_draw_samples,
        avg(mulligans) AS mulligans_average,count(mulligans) AS mulligan_samples,
        coalesce(sum(land_drops_observed),0) AS lands,coalesce(sum(land_turn_samples),0) AS land_samples
    FROM selected
)
SELECT json_object('games',games,'matches',matches,'known_results',known,'wins',wins,'losses',losses,'draws',draws,
    'unknown_results',games-known,'partial_games',partial,'decisive_games',wins+losses,
    'decisive_win_rate',CASE WHEN wins+losses>0 THEN 1.0*wins/(wins+losses) END,
    'on_play_samples',on_play,'on_draw_samples',on_draw,'unknown_play_draw',games-play_draw_samples,
    'observed_mulligans_average',mulligans_average,'mulligan_samples',mulligan_samples,
    'recorded_land_drops',lands,'land_turn_samples',land_samples,
    'by_mode',json((SELECT json_group_array(json(item)) FROM (
        SELECT json_object('mode',mode,'games',count(*),'known_results',count(outcome),
            'wins',coalesce(sum(outcome='won'),0),'losses',coalesce(sum(outcome='lost'),0),
            'draws',coalesce(sum(outcome='draw'),0)) AS item FROM selected GROUP BY mode ORDER BY mode
    ))))
FROM totals
