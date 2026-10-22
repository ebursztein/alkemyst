mod support;
use alkemyst_core::{
    catalog::{Catalog, Search},
    db::{Engine, Statement},
};
use serde_json::json;
use support::{cards, source};

async fn setup() -> (Engine, Catalog) {
    let db = Engine::memory(1024 * 1024).unwrap();
    let catalog = Catalog::new(db.clone());
    catalog
        .import_jsonl(std::io::Cursor::new(cards()), source())
        .await
        .unwrap();
    (db, catalog)
}

#[tokio::test]
async fn text_filters_search_faces_types_and_rules_without_fts_injection() {
    let (_, c) = setup().await;
    for (text, expected) in [
        ("equipment", 2),
        ("FORGEBL", 1),
        ("enters tapped", 1),
        ("missing", 0),
    ] {
        assert_eq!(
            c.search(Search {
                text: text.into(),
                ..Search::default()
            })
            .await
            .unwrap()
            .items
            .len(),
            expected
        );
    }
    assert!(
        c.search(Search {
            text: "\" OR * NEAR(foo)".into(),
            ..Search::default()
        })
        .await
        .is_ok()
    );
    let page = c
        .search(Search {
            colors: Some("w".into()),
            format: Some("standard".into()),
            mana_min: Some(2.0),
            mana_max: Some(2.0),
            arena_only: true,
            legendary: Some(true),
            type_line: Some("Equipment".into()),
            ..Search::default()
        })
        .await
        .unwrap();
    assert_eq!(page.items[0].name, "Forgeblade");
    let nonlegendary = c
        .search(Search {
            legendary: Some(false),
            ..Search::default()
        })
        .await
        .unwrap();
    assert_eq!(nonlegendary.items.len(), 2);
    let modal = c.card("oracle-modal").await.unwrap().unwrap();
    assert_eq!(
        modal["card_faces"][1]["oracle_text"],
        "This land enters tapped."
    );
}

#[tokio::test]
async fn pagination_is_stable_and_never_duplicates_reprints() {
    let (_, c) = setup().await;
    let mut cursor = None;
    let mut names = Vec::new();
    loop {
        let page = c
            .search(Search {
                limit: 1,
                cursor,
                ..Search::default()
            })
            .await
            .unwrap();
        assert_eq!(page.items.len(), 1);
        names.push(page.items[0].name.clone());
        cursor = page.next_cursor;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(names, ["Ember // Meadow", "Find the Blade", "Forgeblade"]);
}

#[tokio::test]
async fn invalid_filters_are_rejected_before_database_work() {
    let (db, c) = setup().await;
    let before = db.metrics().await.unwrap().read_calls;
    let invalid = [
        Search {
            limit: 0,
            ..Search::default()
        },
        Search {
            limit: 101,
            ..Search::default()
        },
        Search {
            text: "x".repeat(257),
            ..Search::default()
        },
        Search {
            colors: Some("purple".into()),
            ..Search::default()
        },
        Search {
            format: Some("$.bad'".into()),
            ..Search::default()
        },
        Search {
            mana_min: Some(-1.0),
            ..Search::default()
        },
        Search {
            mana_max: Some(f64::NAN),
            ..Search::default()
        },
        Search {
            mana_min: Some(3.0),
            mana_max: Some(2.0),
            ..Search::default()
        },
        Search {
            cursor: Some("broken".into()),
            ..Search::default()
        },
        Search {
            cursor: Some("x".repeat(2049)),
            ..Search::default()
        },
        Search {
            type_line: Some("x".repeat(129)),
            ..Search::default()
        },
    ];
    for request in invalid {
        assert!(c.search(request).await.is_err());
    }
    assert!(c.resolve_arena(&vec![1; 1001]).await.is_err());
    assert_eq!(db.metrics().await.unwrap().read_calls, before);
}

#[tokio::test]
async fn search_and_arena_resolution_use_indexes() {
    let (db, _) = setup().await;
    let search = Statement {
        name: "test.searchPlan",
        sql: include_str!("../src/statements/cards/search.sql"),
        reads: &[],
        cacheable: false,
    };
    let plan = db
        .explain(
            search,
            vec![
                json!("forge*"),
                json!(null),
                json!(null),
                json!(null),
                json!(null),
                json!(null),
                json!(false),
                json!(null),
                json!(null),
                json!(null),
                json!(41),
            ],
        )
        .await
        .unwrap();
    assert!(
        plan.iter().any(|step| step.contains("card_order")),
        "{plan:?}"
    );
    assert!(plan.iter().any(|step| step.contains("VIRTUAL TABLE INDEX")));
    let resolve = Statement {
        sql: include_str!("../src/statements/cards/resolve.sql"),
        ..search
    };
    let plan = db.explain(resolve, vec![json!("[42,43]")]).await.unwrap();
    assert!(plan.iter().any(|step| step.contains("card_arena")));
}
