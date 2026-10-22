use super::Error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use utoipa::ToSchema;
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DrawProbability {
    pub population: u16,
    pub successes: u16,
    pub draws: u16,
    pub min: u16,
    pub max: u16,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub label: String,
    pub population: u16,
    pub min: u16,
    pub max: u16,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct JointProbability {
    pub draws: u16,
    /// Mutually exclusive categories covering the whole deck, including other cards.
    /// Split overlapping groups into explicit intersections before using this model.
    pub categories: Vec<Category>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProbabilityMethod {
    Hypergeometric,
    MultivariateHypergeometric,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ExactProbability {
    pub probability: f64,
    pub population: u16,
    pub draws: u16,
    pub method: ProbabilityMethod,
}
pub fn draw(input: DrawProbability) -> Result<ExactProbability, Error> {
    let probability = landlord::probability::hypergeometric(
        input.population,
        input.successes,
        input.draws,
        input.min,
        input.max,
    )
    .map_err(|_| Error::Invalid("invalid exact draw parameters"))?;
    Ok(ExactProbability {
        probability,
        population: input.population,
        draws: input.draws,
        method: ProbabilityMethod::Hypergeometric,
    })
}
pub fn joint(input: JointProbability) -> Result<ExactProbability, Error> {
    let mut labels = BTreeSet::new();
    if input.categories.len() > 16
        || input.categories.iter().any(|c| {
            c.label.trim().is_empty()
                || c.label.chars().count() > 80
                || !labels.insert(c.label.clone())
        })
    {
        return Err(Error::Invalid(
            "joint categories need distinct labels of 1..=80 characters, at most 16 categories",
        ));
    }
    let categories = input
        .categories
        .iter()
        .map(|c| landlord::probability::Category {
            population: c.population,
            min: c.min,
            max: c.max,
        })
        .collect::<Vec<_>>();
    let probability = landlord::probability::joint_probability(input.draws, &categories)
        .map_err(|_| Error::Invalid("invalid disjoint-category draw parameters"))?;
    Ok(ExactProbability {
        probability,
        population: input.categories.iter().map(|c| c.population).sum(),
        draws: input.draws,
        method: ProbabilityMethod::MultivariateHypergeometric,
    })
}
