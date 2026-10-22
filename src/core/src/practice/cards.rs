use super::*;
use crate::{analysis::DeckSnapshot, domain::Zone};
use serde_json::Value;
fn string(data: &Value, key: &str) -> Option<String> {
    data[key].as_str().map(str::to_owned)
}
fn face(data: &Value) -> PracticeFace {
    PracticeFace {
        name: string(data, "name"),
        mana_cost: string(data, "mana_cost"),
        type_line: string(data, "type_line"),
        oracle_text: string(data, "oracle_text"),
        image_uri: data["image_uris"]["normal"].as_str().map(str::to_owned),
    }
}
pub(super) fn physical(snapshot: &DeckSnapshot) -> Vec<PracticeCard> {
    let mut cards = vec![];
    for entry in snapshot.cards.iter().filter(|c| c.zone == Zone::Main) {
        let data = entry.data.as_ref().unwrap_or(&Value::Null);
        let mut front = face(data);
        let mut faces = data["card_faces"]
            .as_array()
            .map(|f| f.iter().map(face).collect::<Vec<_>>())
            .unwrap_or_default();
        for (i, f) in faces.iter_mut().enumerate() {
            if f.type_line.is_none() {
                f.type_line = front
                    .type_line
                    .as_deref()
                    .and_then(|s| s.split(" // ").nth(i))
                    .map(str::to_owned);
            }
        }
        if let Some(first) = faces.first() {
            front.image_uri = front.image_uri.or_else(|| first.image_uri.clone());
        }
        if faces.is_empty() {
            faces.push(front.clone());
        }
        for _ in 0..entry.quantity {
            cards.push(PracticeCard {
                id: cards.len() as u16,
                reference: entry.reference.clone(),
                name: front.name.clone(),
                mana_cost: front.mana_cost.clone(),
                type_line: front.type_line.clone(),
                oracle_text: front.oracle_text.clone(),
                image_uri: front.image_uri.clone(),
                faces: faces.clone(),
            });
        }
    }
    cards
}
pub(super) fn is_land(card: &PracticeCard, index: u8) -> bool {
    card.faces
        .get(usize::from(index))
        .and_then(|f| f.type_line.as_ref())
        .is_some_and(|t| t.contains("Land"))
}
pub(super) fn view(row: Stored) -> Result<PracticeView, Error> {
    let cards = physical(&row.snapshot);
    let get = |id: u16| {
        cards
            .get(usize::from(id))
            .cloned()
            .ok_or(Error::Incompatible)
    };
    let collect = |ids: &[u16]| ids.iter().map(|id| get(*id)).collect::<Result<Vec<_>, _>>();
    let warnings=vec!["Manual sandbox: mana payments, abilities, combat and state-based actions are not enforced.".into(),
        "Play Land tracks one land drop per turn. Move is an explicit manual override; it cannot reveal the library.".into()];
    Ok(PracticeView {
        id: row.id,
        version_id: row.version_id,
        revision: row.revision,
        origin: row.origin,
        last_action_origin: row.last_action_origin,
        created_at: row.created_at,
        updated_at: row.updated_at,
        seed: row.seed,
        replay_version: row.replay_version,
        practice_version: row.practice_version,
        rules_mode: "manual_sandbox".into(),
        on_the_play: row.on_the_play,
        phase: row.state.phase.clone(),
        mulligans: row.state.mulligans,
        bottom_count: if row.state.phase == PracticePhase::Opening {
            row.state.mulligans
        } else {
            0
        },
        turn: row.state.turn,
        library_count: row.state.library.len() as u16,
        land_played: row.state.land_played,
        decked: row.state.decked,
        hand: collect(&row.state.hand)?,
        graveyard: collect(&row.state.graveyard)?,
        exile: collect(&row.state.exile)?,
        battlefield: row
            .state
            .battlefield
            .iter()
            .map(|p| {
                Ok(PracticePermanent {
                    card: get(p.card)?,
                    face: p.face,
                    tapped: p.tapped,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?,
        warnings,
    })
}
