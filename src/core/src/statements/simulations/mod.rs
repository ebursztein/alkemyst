use crate::db::{Mutation, Statement};
pub const GET: Statement = Statement {
    name: "simulations.get",
    sql: "SELECT data FROM simulation_details WHERE id=?1",
    reads: &["simulation_runs"],
    cacheable: true,
};
pub const INPUTS: Statement = Statement {
    name: "simulations.inputs",
    sql: "SELECT json_object('snapshot',json(snapshot),'profile',json(profile),'parameters',json(parameters)) FROM simulation_runs WHERE id=?1",
    reads: &["simulation_runs"],
    cacheable: true,
};
pub const LIST: Statement = Statement {
    name: "simulations.list",
    sql: include_str!("list.sql"),
    reads: &["simulation_runs"],
    cacheable: true,
};
pub const RECEIPT: Statement = Statement {
    name: "simulations.receipt",
    sql: "SELECT json_object('operation',r.operation,'body',r.body,'job',json(d.data)) FROM mutation_receipts r LEFT JOIN simulation_details d ON d.id=r.result_id WHERE r.key=?1",
    reads: &["mutation_receipts", "simulation_runs"],
    cacheable: true,
};
pub const CREATE: Mutation = Mutation {
    sql: include_str!("create.sql"),
    writes: &["simulation_runs"],
};
pub const CANCEL: Mutation = Mutation {
    sql: "UPDATE simulation_runs SET cancel_requested=1,cancellation_origin=?2,status=CASE WHEN status='queued' THEN 'cancelled' ELSE status END,updated_at=unixepoch() WHERE id=?1 AND status IN ('queued','running') AND cancel_requested=0",
    writes: &["simulation_runs"],
};
pub const CLAIM: Mutation = Mutation {
    sql: "UPDATE simulation_runs SET status='running',owner=?1,updated_at=unixepoch() WHERE ordinal=(SELECT ordinal FROM simulation_runs WHERE status='queued' AND cancel_requested=0 ORDER BY ordinal LIMIT 1) AND status='queued'",
    writes: &["simulation_runs"],
};
pub const OWNED: Statement = Statement {
    name: "simulations.owned",
    sql: "SELECT json_object('id',id,'compiler_version',compiler_version,'model_version',model_version,'policy_version',policy_version,'replay_version',replay_version,'inputs',json_object('snapshot',json(snapshot),'profile',json(profile),'parameters',json(parameters))) FROM simulation_runs WHERE owner=?1 AND status='running'",
    reads: &["simulation_runs"],
    cacheable: false,
};
pub const CONTROL: Statement = Statement {
    name: "simulations.control",
    sql: "SELECT json_object('cancel',json(CASE WHEN cancel_requested=1 THEN 'true' ELSE 'false' END)) FROM simulation_runs WHERE id=?1 AND owner=?2 AND status='running'",
    reads: &["simulation_runs"],
    cacheable: false,
};
pub const PROGRESS: Mutation = Mutation {
    sql: "UPDATE simulation_runs SET completed_runs=?3,report=?4,status=CASE WHEN cancel_requested=1 THEN 'cancelled' ELSE ?5 END,error_code=CASE WHEN cancel_requested=1 THEN NULL ELSE ?6 END,updated_at=unixepoch() WHERE id=?1 AND owner=?2 AND status='running'",
    writes: &["simulation_runs"],
};
pub const HEARTBEAT: Mutation = Mutation {
    sql: "UPDATE simulation_runs SET updated_at=unixepoch() WHERE id=?1 AND owner=?2 AND status='running'",
    writes: &["simulation_runs"],
};
pub const RECOVER: Mutation = Mutation {
    sql: "UPDATE simulation_runs SET status=CASE WHEN cancel_requested=1 THEN 'cancelled' ELSE 'interrupted' END,error_code=CASE WHEN cancel_requested=1 THEN NULL ELSE 'interrupted' END WHERE status='running' AND updated_at<unixepoch()-30",
    writes: &["simulation_runs"],
};
