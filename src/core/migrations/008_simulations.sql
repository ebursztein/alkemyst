CREATE TABLE simulation_runs (
    ordinal INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    version_id TEXT NOT NULL REFERENCES deck_versions(id),
    origin TEXT NOT NULL CHECK(origin IN ('user','agent','arena','import')),
    created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('queued','running','completed','cancelled','failed','interrupted')),
    requested_runs INTEGER NOT NULL CHECK(requested_runs BETWEEN 1 AND 1000000),
    completed_runs INTEGER NOT NULL DEFAULT 0 CHECK(completed_runs BETWEEN 0 AND requested_runs),
    seed TEXT NOT NULL CHECK(length(seed)=16 AND seed NOT GLOB '*[^0-9a-f]*'),
    compiler_version TEXT NOT NULL, model_version TEXT NOT NULL, policy_version TEXT NOT NULL,replay_version TEXT NOT NULL,
    parameters TEXT NOT NULL CHECK(json_valid(parameters)),
    snapshot TEXT NOT NULL CHECK(json_valid(snapshot)),profile TEXT NOT NULL CHECK(json_valid(profile)),
    report TEXT CHECK(report IS NULL OR json_valid(report)),error_code TEXT,
    cancel_requested INTEGER NOT NULL DEFAULT 0 CHECK(cancel_requested IN (0,1)),cancellation_origin TEXT,
    owner TEXT,
    CHECK(status!='running' OR owner IS NOT NULL),
    CHECK(status!='completed' OR (completed_runs=requested_runs AND report IS NOT NULL)),
    CHECK((completed_runs=0 AND report IS NULL) OR (completed_runs>0 AND report IS NOT NULL
        AND json_extract(report,'$.completed_runs')=completed_runs
        AND json_extract(report,'$.requested_runs')=requested_runs
        AND json_extract(report,'$.seed')=seed))
) STRICT;
CREATE INDEX simulation_version_order ON simulation_runs(version_id,ordinal DESC);
CREATE INDEX simulation_queue ON simulation_runs(status,ordinal);
CREATE UNIQUE INDEX simulation_owner ON simulation_runs(owner) WHERE status='running';
CREATE TRIGGER simulation_inputs_immutable BEFORE UPDATE ON simulation_runs
WHEN new.id IS NOT old.id OR new.ordinal IS NOT old.ordinal OR new.version_id IS NOT old.version_id
    OR new.origin IS NOT old.origin OR new.created_at IS NOT old.created_at
    OR new.requested_runs IS NOT old.requested_runs OR new.seed IS NOT old.seed
    OR new.compiler_version IS NOT old.compiler_version OR new.model_version IS NOT old.model_version
    OR new.policy_version IS NOT old.policy_version OR new.replay_version IS NOT old.replay_version
    OR new.parameters IS NOT old.parameters OR new.snapshot IS NOT old.snapshot OR new.profile IS NOT old.profile
    OR new.completed_runs<old.completed_runs
BEGIN SELECT RAISE(ABORT,'simulation inputs are immutable and progress is monotone'); END;
CREATE TRIGGER simulation_terminal_immutable BEFORE UPDATE ON simulation_runs
WHEN old.status IN ('completed','cancelled','failed','interrupted') AND (
    new.status IS NOT old.status OR new.completed_runs IS NOT old.completed_runs
    OR new.report IS NOT old.report OR new.error_code IS NOT old.error_code
    OR new.cancel_requested IS NOT old.cancel_requested OR new.cancellation_origin IS NOT old.cancellation_origin
    OR new.updated_at IS NOT old.updated_at OR new.owner IS NOT old.owner)
BEGIN SELECT RAISE(ABORT,'terminal simulation results are immutable'); END;
