# Persisted simulation jobs

Simulation creation atomically stores an idempotency receipt, exact resolved card
snapshot, profile, parameters, seed and compiler/model/policy/RNG versions. Retries
return the original job, even after a catalog refresh. Agent requests preserve
Agent provenance; replaying a user's receipt does not rewrite that provenance.

Seeds are 16 hexadecimal digits in JSON, including reports. Rust retains the full
u64 internally, and JavaScript never has to represent it as a number. Generated
seeds are committed before execution. Required mulligan references resolve to
stable model identities in a sorted set, so replay does not depend on hash order.

All job/status/input/list reads are prepared SQL statements using the one engine
cache. A read never polls an in-memory result copy. The bounded worker runs native
calculation off the async/UI thread in batches of eight complete trials. Each
batch commits its completed count and report together; zero completed trials
produce no report. Intervals quantify sampling uncertainty conditional on the
selected model, not uncertainty about the model's game assumptions.

The queue holds at most 16 queued/running jobs, with one calculation worker per
application coordinator. Atomic claiming and owner predicates prevent duplicate
execution or stale result writes. Heartbeats protect a long batch's ownership.
After an unexpected process stop, a running owner with no heartbeat for 30 seconds
becomes interrupted (or cancelled when cancellation was already requested). No
interrupted job is silently rerun using today's catalog. Queued jobs survive
restart and use their frozen inputs; incompatible model versions fail explicitly.

The HTTP service starts the coordinator and stops it on shutdown. Native Quit must
await service shutdown; hiding a window must keep the service alive. Cancellation
is idempotent and checked between bounded batches. Shutdown preserves committed
partial samples as interrupted. Callers can read terminal results after shutdown;
a new application instance is required to start a stopped coordinator.

## Operations

- `POST /v1/simulations`: idempotently enqueue and start; returns HTTP 202.
- `GET /v1/simulations`: bounded creation-order cursor pages, optional version filter.
- `GET /v1/simulations/{id}`: durable status, parameters and optional sampled report.
- `GET /v1/simulations/{id}/inputs`: frozen metadata/profile used by this run.
- `POST /v1/simulations/{id}/cancel`: request cancellation or return a terminal job.
- `POST /v1/probabilities/draw`: exact finite-deck hypergeometric draw event.
- `POST /v1/probabilities/joint`: exact joint counts over disjoint whole-deck categories.

The generated SDK and allowlisted MCP tools call the same operations. Exact draw
endpoints do not model casting, tutoring or Arena smoothing. For overlapping
classes, explicitly partition their intersections before using the joint endpoint;
the battlefield simulator instead evaluates joint goals within each trial.
