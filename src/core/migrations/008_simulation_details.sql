CREATE VIEW simulation_details AS SELECT ordinal,id,version_id,
    json_object('id',id,'version_id',version_id,
        'status',CASE WHEN status='running' AND cancel_requested=1 THEN 'cancelling' ELSE status END,
        'origin',origin,'created_at',created_at,'updated_at',updated_at,
        'requested_runs',requested_runs,'completed_runs',completed_runs,'seed',seed,
        'compiler_version',compiler_version,'model_version',model_version,
        'policy_version',policy_version,'replay_version',replay_version,
        'catalog_snapshot',json_extract(snapshot,'$.catalog_snapshot'),
        'parameters',json(parameters),'report',json(report),'error_code',error_code,
        'cancellation_origin',cancellation_origin) AS data
FROM simulation_runs;
PRAGMA user_version=8;
