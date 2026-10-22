INSERT INTO simulation_runs(id,version_id,origin,created_at,updated_at,status,requested_runs,seed,
    compiler_version,model_version,policy_version,replay_version,parameters,snapshot,profile)
SELECT receipt.result_id,?2,?3,unixepoch(),unixepoch(),'queued',?4,?5,?6,?7,?8,?9,?10,?11,?12
FROM mutation_receipts receipt WHERE receipt.key=?1 AND (
    EXISTS(SELECT 1 FROM simulation_runs WHERE id=receipt.result_id)
    OR (SELECT count(*) FROM simulation_runs WHERE status IN ('queued','running'))<16)
ON CONFLICT(id) DO UPDATE SET id=excluded.id
