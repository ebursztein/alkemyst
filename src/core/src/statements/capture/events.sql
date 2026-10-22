INSERT INTO capture_events(id,match_id,game_id,kind,state_id,payload,local_time,instant_utc)
SELECT json_extract(value,'$.id'),json_extract(value,'$.match_id'),json_extract(value,'$.game_id'),
    json_extract(value,'$.kind'),json_extract(value,'$.state_id'),
    json_extract(value,'$.payload'),json_extract(value,'$.local_time'),json_extract(value,'$.instant_utc')
FROM json_each(?1) WHERE true
ON CONFLICT(id) DO NOTHING
