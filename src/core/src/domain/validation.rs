use super::{DeckCard, Error, Result, Zone};
use std::collections::BTreeMap;

pub(super) fn request_key(key: &str) -> Result<()> {
    if key.is_empty() || key.len() > 128 || key.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "idempotency key must be 1..128 bytes without control characters",
        ));
    }
    Ok(())
}

pub(super) fn deck(name: &str, format: &str, cards: &mut Vec<DeckCard>) -> Result<()> {
    if name.trim().is_empty() || name.len() > 200 || format.is_empty() || format.len() > 64 {
        return Err(Error::Invalid(
            "deck name and format are required and bounded",
        ));
    }
    if cards.len() > 500 {
        return Err(Error::Invalid("too many card lines"));
    }
    let mut normalized = BTreeMap::<(Zone, String), u16>::new();
    let mut total = 0_u32;
    for card in cards.iter() {
        reference(&card.reference)?;
        if !(1..=250).contains(&card.quantity) {
            return Err(Error::Invalid("card quantity must be 1..250"));
        }
        let quantity = normalized
            .entry((card.zone, card.reference.clone()))
            .or_default();
        *quantity = quantity
            .checked_add(card.quantity)
            .filter(|n| *n <= 250)
            .ok_or(Error::Invalid("combined card quantity exceeds 250"))?;
        total += u32::from(card.quantity);
    }
    if total > 1000 {
        return Err(Error::Invalid("deck exceeds 1000 cards"));
    }
    *cards = normalized
        .into_iter()
        .map(|((zone, reference), quantity)| DeckCard {
            zone,
            reference,
            quantity,
        })
        .collect();
    Ok(())
}

fn reference(reference: &str) -> Result<()> {
    let (kind, id) = reference
        .split_once(':')
        .ok_or(Error::Invalid("card reference needs a namespace"))?;
    let valid = match kind {
        "arena" => id.parse::<u32>().is_ok_and(|id| id > 0),
        "oracle" | "scryfall" => {
            !id.is_empty()
                && id.len() <= 128
                && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        }
        _ => false,
    };
    if !valid {
        return Err(Error::Invalid("invalid card reference"));
    }
    Ok(())
}
