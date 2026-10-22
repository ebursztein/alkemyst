WITH refs AS (
    SELECT version_id,reference,quantity,zone FROM deck_cards WHERE version_id IN (?1,?2)
), resolved AS MATERIALIZED (
    SELECT r.version_id,r.reference,r.quantity,r.zone,p.data,
        CASE WHEN p.data IS NULL THEN r.reference ELSE 'oracle:'||json_extract(p.data,'$.oracle_id') END AS identity,
        coalesce(json_extract(p.data,'$.card_faces[0].type_line'),
            CASE WHEN instr(json_extract(p.data,'$.type_line'),' // ')>0
                THEN substr(json_extract(p.data,'$.type_line'),1,instr(json_extract(p.data,'$.type_line'),' // ')-1)
                ELSE json_extract(p.data,'$.type_line') END) AS front_type,
        json_extract(p.data,'$.cmc') AS mana_value
