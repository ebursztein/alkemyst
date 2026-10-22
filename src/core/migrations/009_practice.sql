CREATE TABLE practice_sessions (
    ordinal INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,version_id TEXT NOT NULL REFERENCES deck_versions(id),
    origin TEXT NOT NULL CHECK(origin IN ('user','agent','arena','import')),
    created_at INTEGER NOT NULL,seed TEXT NOT NULL CHECK(length(seed)=16 AND seed NOT GLOB '*[^0-9a-f]*'),
    practice_version TEXT NOT NULL,replay_version TEXT NOT NULL,
    on_the_play INTEGER NOT NULL CHECK(on_the_play IN (0,1)),snapshot TEXT NOT NULL CHECK(json_valid(snapshot))
) STRICT;
CREATE INDEX practice_version_order ON practice_sessions(version_id,ordinal DESC);
CREATE TABLE practice_steps (
    id TEXT PRIMARY KEY NOT NULL,session_id TEXT NOT NULL REFERENCES practice_sessions(id),
    revision INTEGER NOT NULL CHECK(revision BETWEEN 0 AND 1000),created_at INTEGER NOT NULL,
    origin TEXT NOT NULL CHECK(origin IN ('user','agent','arena','import')),
    action TEXT CHECK(action IS NULL OR json_valid(action)),state TEXT NOT NULL CHECK(json_valid(state)),
    UNIQUE(session_id,revision),CHECK((revision=0 AND action IS NULL) OR (revision>0 AND action IS NOT NULL))
) STRICT;
CREATE TRIGGER practice_session_no_update BEFORE UPDATE ON practice_sessions
BEGIN SELECT RAISE(ABORT,'practice inputs are immutable'); END;
CREATE TRIGGER practice_session_no_delete BEFORE DELETE ON practice_sessions
BEGIN SELECT RAISE(ABORT,'practice inputs are immutable'); END;
CREATE TRIGGER practice_step_no_update BEFORE UPDATE ON practice_steps
BEGIN SELECT RAISE(ABORT,'practice steps are immutable'); END;
CREATE TRIGGER practice_step_no_delete BEFORE DELETE ON practice_steps
BEGIN SELECT RAISE(ABORT,'practice steps are immutable'); END;
CREATE TRIGGER practice_step_contiguous BEFORE INSERT ON practice_steps
WHEN new.revision!=coalesce((SELECT max(revision) FROM practice_steps WHERE session_id=new.session_id),-1)+1
BEGIN SELECT RAISE(ABORT,'practice revisions must be contiguous'); END;
CREATE VIEW practice_details AS SELECT s.id,s.version_id,p.id AS step_id,p.revision,
    json_object('id',s.id,'version_id',s.version_id,'origin',s.origin,'last_action_origin',p.origin,
        'created_at',s.created_at,'updated_at',p.created_at,'seed',s.seed,
        'practice_version',s.practice_version,'replay_version',s.replay_version,
        'on_the_play',json(CASE WHEN s.on_the_play=1 THEN 'true' ELSE 'false' END),
        'snapshot',json(s.snapshot),'revision',p.revision,'state',json(p.state)) AS data
FROM practice_sessions s JOIN practice_steps p ON p.session_id=s.id;
PRAGMA user_version=9;
