//! Exact finite-deck sampling probabilities (no replacement).
//! The 1000-card bound keeps binomial coefficients finite in f64, even at N/2.
use crate::Error;

#[derive(Debug, Clone, Copy)]
pub struct Category {
    pub population: u16,
    pub min: u16,
    pub max: u16,
}

fn choose(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    let k = k.min(n - k);
    (1..=k).fold(1.0, |value, i| value * (n - k + i) as f64 / i as f64)
}

pub fn hypergeometric(
    population: u16,
    successes: u16,
    draws: u16,
    min: u16,
    max: u16,
) -> Result<f64, Error> {
    if population > 1000 || successes > population || draws > population || min > max {
        return Err(Error::Configuration(
            "invalid hypergeometric parameters".into(),
        ));
    }
    let (n, k, d) = (
        usize::from(population),
        usize::from(successes),
        usize::from(draws),
    );
    let lower = usize::from(min).max(d.saturating_sub(n - k));
    let upper = usize::from(max).min(k).min(d);
    let numerator: f64 = (lower..=upper)
        .map(|x| choose(k, x) * choose(n - k, d - x))
        .sum();
    Ok((numerator / choose(n, d)).clamp(0.0, 1.0))
}

/// Categories must be disjoint and cover the whole deck, including an
/// unconstrained "other" category when needed. Evaluates all bounds jointly.
pub fn joint_probability(draws: u16, categories: &[Category]) -> Result<f64, Error> {
    let population: usize = categories.iter().map(|c| usize::from(c.population)).sum();
    if population > 1000
        || usize::from(draws) > population
        || categories.len() > 16
        || categories.iter().any(|c| c.min > c.max)
    {
        return Err(Error::Configuration("invalid joint categories".into()));
    }
    let d = usize::from(draws);
    let mut coefficients = vec![0.0; d + 1];
    coefficients[0] = 1.0;
    for category in categories {
        let mut next = vec![0.0; d + 1];
        let max = usize::from(category.max)
            .min(usize::from(category.population))
            .min(d);
        for count in usize::from(category.min)..=max {
            let ways = choose(usize::from(category.population), count);
            for before in 0..=d - count {
                next[before + count] += coefficients[before] * ways;
            }
        }
        coefficients = next;
    }
    Ok((coefficients[d] / choose(population, d)).clamp(0.0, 1.0))
}
