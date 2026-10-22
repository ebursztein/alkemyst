WITH refs AS (
    SELECT reference, quantity, zone FROM deck_cards WHERE version_id=?1
), resolved AS MATERIALIZED (
    SELECT r.reference, r.quantity, r.zone, p.data,
        coalesce(json_extract(p.data,'$.card_faces[0].type_line'),json_extract(p.data,'$.type_line')) AS front_type,
        json_extract(p.data,'$.cmc') AS mana_value
