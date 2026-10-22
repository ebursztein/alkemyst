//! Stable replay protocol: PCG64-MCG, rejection sampling, Fisher-Yates v1.
use rand_core::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

pub const REPLAY_VERSION: &str = "pcg64-mcg/fisher-yates-v1";

pub struct SimulationRng(Pcg64Mcg);
impl SimulationRng {
    pub fn seeded(seed: u64) -> Self {
        Self(Pcg64Mcg::seed_from_u64(seed))
    }
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() {
            let bound = (i + 1) as u64;
            let threshold = bound.wrapping_neg() % bound;
            let j = loop {
                let value = self.0.next_u64();
                if value >= threshold {
                    break (value % bound) as usize;
                }
            };
            values.swap(i, j);
        }
    }
}
