//! Local deck calculations and explicit, reproducible simulation models.
mod estimates;
pub mod simulation;
pub mod turns;
pub use estimates::Estimate;
mod snapshot;
pub use snapshot::{Analysis, Counts, CurveBin, DeckSnapshot, Error, ResolvedCard};
mod comparison;
pub mod compiler;
pub mod jobs;
pub mod probabilities;
pub(crate) mod seed;
pub use comparison::{CardDelta, ComparisonSide, CurveDelta, DeckComparison};

pub(crate) use snapshot::read_snapshot;
