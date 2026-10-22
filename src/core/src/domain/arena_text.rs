use super::{Error, Result, Zone};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(super) struct Line {
    pub name: String,
    pub set_code: Option<String>,
    pub collector_number: Option<String>,
    pub quantity: u16,
    pub zone: Zone,
}

#[derive(Deserialize)]
pub(super) struct ExportCard {
    pub reference: String,
    pub name: Option<String>,
    pub set_code: Option<String>,
    pub collector_number: Option<String>,
    pub quantity: u16,
    pub zone: Zone,
}

pub(super) fn parse(input: &str) -> Result<Vec<Line>> {
    if input.len() > 100_000 {
        return Err(Error::Invalid("deck text too long"));
    }
    let mut result = Vec::new();
    let mut zone = Zone::Main;
    let mut pending_sideboard = false;
    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() {
            pending_sideboard = zone == Zone::Main && !result.is_empty();
            continue;
        }
        let header = match line {
            "Deck" => Some(Zone::Main),
            "Sideboard" => Some(Zone::Sideboard),
            "Commander" => Some(Zone::Command),
            "Companion" => Some(Zone::Companion),
            _ => None,
        };
        if let Some(header) = header {
            zone = header;
            pending_sideboard = false;
            continue;
        }
        if line == "About" || line.starts_with("Name ") || line.starts_with("//") {
            continue;
        }
        if pending_sideboard {
            zone = Zone::Sideboard;
            pending_sideboard = false;
        }
        let (quantity, name) = line
            .split_once(char::is_whitespace)
            .ok_or(Error::Invalid("expected quantity and card name"))?;
        let quantity = quantity
            .parse::<u16>()
            .ok()
            .filter(|n| (1..=250).contains(n))
            .ok_or(Error::Invalid("invalid card quantity"))?;
        let name = name.trim();
        if name.is_empty() || name.len() > 512 {
            return Err(Error::Invalid("invalid card name"));
        }
        let (name, set_code, collector_number) =
            if let Some((name, suffix)) = name.rsplit_once(" (") {
                if let Some((set, number)) = suffix.split_once(") ") {
                    if set.is_empty()
                        || !set.chars().all(|c| c.is_ascii_alphanumeric())
                        || number.is_empty()
                        || !number
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '/')
                    {
                        return Err(Error::Invalid("invalid set or collector number"));
                    }
                    (
                        name.to_owned(),
                        Some(set.to_ascii_lowercase()),
                        Some(number.into()),
                    )
                } else {
                    (name.to_string() + " (" + suffix, None, None)
                }
            } else {
                (name.into(), None, None)
            };
        result.push(Line {
            name,
            set_code,
            collector_number,
            quantity,
            zone,
        });
        if result.len() > 500 {
            return Err(Error::Invalid("too many card lines"));
        }
    }
    if result.is_empty() {
        return Err(Error::Invalid("no cards in imported deck"));
    }
    Ok(result)
}

pub(super) fn export(cards: Vec<ExportCard>) -> Result<String> {
    let missing = cards
        .iter()
        .filter(|c| c.name.is_none())
        .map(|c| c.reference.clone())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(Error::UnknownCards(missing));
    }
    let mut sections = Vec::new();
    for (zone, header) in [
        (Zone::Companion, "Companion"),
        (Zone::Command, "Commander"),
        (Zone::Main, "Deck"),
        (Zone::Sideboard, "Sideboard"),
    ] {
        let mut section = vec![header.to_string()];
        for card in cards.iter().filter(|card| card.zone == zone) {
            let printing = match (&card.set_code, &card.collector_number) {
                (Some(set), Some(number)) => format!(" ({}) {number}", set.to_ascii_uppercase()),
                _ => String::new(),
            };
            section.push(format!(
                "{} {}{}",
                card.quantity,
                card.name.as_deref().unwrap_or_default(),
                printing
            ));
        }
        if section.len() > 1 || zone == Zone::Main {
            sections.push(section.join("\n"));
        }
    }
    Ok(sections.join("\n\n") + "\n")
}
