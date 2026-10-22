use super::*;
use crate::analysis::{compiler::CompiledDeck, seed};
use landlord::London;
use std::collections::BTreeSet;
pub(super) fn identifier(id: &str) -> Result<(), Error> {
    if id.is_empty() || id.len() > 256 {
        Err(Error::Invalid("identifier must contain 1..=256 bytes"))
    } else {
        Ok(())
    }
}
pub(super) fn request(key: &str, p: &mut SimulationParameters) -> Result<(), Error> {
    if key.is_empty() || key.len() > 128 || key.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "idempotency key must contain 1..=128 bytes without control characters",
        ));
    }
    identifier(&p.version_id)?;
    if p.runs == 0
        || p.runs > 1_000_000
        || p.turns == 0
        || p.turns > 30
        || [p.goal.equipment, p.goal.attached, p.goal.attack_ready]
            .iter()
            .any(|v| *v > 1000)
    {
        return Err(Error::Invalid("invalid simulation bounds"));
    }
    if let Some(value) = &mut p.seed {
        seed::parse(value).map_err(Error::Invalid)?;
        *value = value.to_ascii_lowercase();
    }
    let m = &mut p.mulligan;
    if m.minimum_keep > 7
        || m.force_mulligans > 7 - m.minimum_keep
        || m.min_lands > m.max_lands
        || m.max_lands > 7
        || m.required_references.len() > 7
        || m.required_references
            .iter()
            .any(|r| r.is_empty() || r.len() > 200)
    {
        return Err(Error::Invalid("invalid London mulligan parameters"));
    }
    m.required_references.sort();
    m.required_references.dedup();
    Ok(())
}
pub(super) fn mulligan(p: &SimulationParameters, compiled: &CompiledDeck) -> Result<London, Error> {
    let ids = p
        .mulligan
        .required_references
        .iter()
        .map(|r| {
            compiled.identity(r).ok_or(Error::Invalid(
                "required mulligan card is not a modeled main-deck reference",
            ))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    Ok(London {
        minimum_keep: p.mulligan.minimum_keep,
        force_mulligans: p.mulligan.force_mulligans,
        min_lands: p.mulligan.min_lands,
        max_lands: p.mulligan.max_lands,
        required_ids: ids.into_iter().collect(),
    })
}
