INSERT INTO games
SELECT json_extract(value,'$.game_id'),json_extract(value,'$.match_id'),
    json_extract(value,'$.game_number'),json_extract(value,'$.deck_version')
FROM (
    SELECT value,row_number() OVER (
        PARTITION BY json_extract(value,'$.game_id') ORDER BY CAST(key AS INTEGER)
    ) AS first_observation
    FROM json_each(?1) WHERE json_extract(value,'$.game_id') IS NOT NULL
)
WHERE first_observation=1
ON CONFLICT(id) DO NOTHING
