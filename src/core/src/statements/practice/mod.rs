use crate::db::{Mutation, Statement};
const READS: &[&str] = &["practice_sessions", "practice_steps"];
pub const GET: Statement = Statement {
    name: "practice.get",
    sql: "SELECT data FROM practice_details WHERE id=?1 AND revision=(SELECT max(revision) FROM practice_steps WHERE session_id=?1)",
    reads: READS,
    cacheable: true,
};
pub const INPUTS: Statement = Statement {
    name: "practice.inputs",
    sql: "SELECT snapshot FROM practice_sessions WHERE id=?1",
    reads: &["practice_sessions"],
    cacheable: true,
};
pub const RECEIPT: Statement = Statement {
    name: "practice.receipt",
    sql: "SELECT json_object('operation',r.operation,'body',r.body,'value',json(d.data)) FROM mutation_receipts r LEFT JOIN practice_details d ON d.step_id=r.result_id WHERE r.key=?1",
    reads: &["mutation_receipts", "practice_sessions", "practice_steps"],
    cacheable: true,
};
pub const CREATE: Mutation = Mutation {
    sql: "INSERT INTO practice_sessions(id,version_id,origin,created_at,seed,practice_version,replay_version,on_the_play,snapshot)
        SELECT ?1,?2,?3,unixepoch(),?4,?5,?6,?7,?8
        WHERE EXISTS(SELECT 1 FROM mutation_receipts WHERE key=?9 AND result_id=?10)",
    writes: &["practice_sessions"],
};
pub const STEP: Mutation = Mutation {
    sql: "INSERT INTO practice_steps(id,session_id,revision,created_at,origin,action,state) SELECT ?1,?2,?3,unixepoch(),?4,?5,?6 WHERE coalesce((SELECT max(revision) FROM practice_steps WHERE session_id=?2),-1)=?7",
    writes: &["practice_steps"],
};
pub const LIST: Statement = Statement {
    name: "practice.list",
    sql: include_str!("list.sql"),
    reads: READS,
    cacheable: true,
};
