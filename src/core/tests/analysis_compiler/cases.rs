use super::*;
use alkemyst_core::analysis::{ResolvedCard, turns::Kind};

#[tokio::test]
async fn artwork_and_printing_changes_do_not_change_the_rules_fingerprint() {
    let mut raw = snapshot().await;
    raw.cards[0].data.as_mut().unwrap()["artist"] = json!("Different public artwork metadata");
    raw.cards[0].data.as_mut().unwrap()["id"] = json!("different-printing");
    let saved = raw.clone();
    let compiled = compile(raw);
    assert!(compiled.profile.can_simulate);
    assert_eq!(compiled.snapshot(), &saved);
}

#[tokio::test]
async fn generic_basics_and_simple_combat_keywords_compile_without_name_shortcuts() {
    for (name, type_line, symbol, color) in [
        ("Island", "Basic Land — Island", "U", "U"),
        ("Swamp", "Basic Land — Swamp", "B", "B"),
        ("Forest", "Basic Land — Forest", "G", "G"),
        ("Snow-Covered Plains", "Basic Snow Land — Plains", "W", "W"),
    ] {
        let mut raw = snapshot().await;
        let baseline = compile(raw.clone());
        let row = raw
            .cards
            .iter_mut()
            .find(|r| r.data.as_ref().unwrap()["name"] == "Plains")
            .unwrap();
        let quantity = u32::from(row.quantity);
        let card = row.data.as_mut().unwrap();
        card["name"] = json!(name);
        card["oracle_id"] = json!("custom-basic");
        card["type_line"] = json!(type_line);
        card["oracle_text"] = json!(format!("({{T}}: Add {{{symbol}}}.)"));
        let compiled = compile(raw);
        assert!(compiled.profile.can_simulate);
        let before = baseline
            .profile
            .sources
            .iter()
            .find(|s| s.color == color)
            .unwrap()
            .unrestricted_lands;
        let after = compiled
            .profile
            .sources
            .iter()
            .find(|s| s.color == color)
            .unwrap()
            .unrestricted_lands;
        assert_eq!(after, before + if color == "W" { 0 } else { quantity });
    }
    let mut raw = snapshot().await;
    let row = raw
        .cards
        .iter_mut()
        .find(|r| r.data.as_ref().unwrap()["name"] == "Dwarven Mauler")
        .unwrap();
    let card = row.data.as_mut().unwrap();
    card["oracle_id"] = json!("generic-haste");
    card["name"] = json!("Vanilla haste test");
    card["oracle_text"] = json!("Flying, haste");
    card["keywords"] = json!(["Flying", "Haste"]);
    let compiled = compile(raw);
    assert!(compiled.profile.can_simulate);
    let scenario = compiled
        .scenario(2, true, London::never(), Goal::default(), true)
        .unwrap();
    assert!(
        scenario
            .deck
            .iter()
            .any(|c| matches!(&c.kind,Kind::Spell(s) if s.haste))
    );
}

#[tokio::test]
async fn missing_sideboard_cards_do_not_block_the_immutable_main_deck() {
    let mut raw = snapshot().await;
    raw.counts.sideboard += 1;
    raw.cards.push(ResolvedCard {
        reference: "oracle:sideboard-only".into(),
        quantity: 1,
        zone: Zone::Sideboard,
        data: None,
    });
    let compiled = compile(raw);
    assert!(compiled.profile.can_simulate);
    assert!(
        compiled
            .scenario(2, true, London::never(), Goal::default(), true)
            .is_ok()
    );
}

#[tokio::test]
async fn invalid_metadata_layouts_and_inconsistent_snapshot_counts_block() {
    let mut raw = snapshot().await;
    raw.counts.main += 1;
    let compiled = compile(raw);
    assert!(!compiled.profile.can_simulate);
    assert!(
        compiled
            .scenario(2, true, London::never(), Goal::default(), true)
            .is_err()
    );
    let mut raw = snapshot().await;
    raw.cards[0].data.as_mut().unwrap()["name"] = json!("");
    let compiled = compile(raw);
    assert!(
        compiled.profile.cards[0]
            .issues
            .iter()
            .any(|i| i.code == IssueCode::InvalidMetadata)
    );
    let mut raw = snapshot().await;
    let row = raw
        .cards
        .iter_mut()
        .find(|r| r.data.as_ref().unwrap()["card_faces"].is_array())
        .unwrap();
    row.data.as_mut().unwrap()["oracle_id"] = json!("unknown-layout");
    let compiled = compile(raw);
    assert!(compiled.profile.cards.iter().any(|c| {
        c.issues
            .iter()
            .any(|i| i.code == IssueCode::UnsupportedLayout)
    }));
    let compiled = compile(snapshot().await);
    assert!(
        compiled
            .scenario(0, true, London::never(), Goal::default(), true)
            .is_err()
    );
}
