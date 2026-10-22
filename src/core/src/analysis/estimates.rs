/// Wilson score interval, with explicit numerator and denominator.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    pub successes: u32,
    pub samples: u32,
    pub probability: f64,
    pub interval95: [f64; 2],
}
impl Estimate {
    pub(super) fn new(successes: u32, samples: u32) -> Self {
        let n = f64::from(samples);
        let p = f64::from(successes) / n;
        let z = 1.959963984540054;
        let scale = 1.0 + z * z / n;
        let center = (p + z * z / (2.0 * n)) / scale;
        let half = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / scale;
        Self {
            successes,
            samples,
            probability: p,
            interval95: [(center - half).max(0.0), (center + half).min(1.0)],
        }
    }
}
