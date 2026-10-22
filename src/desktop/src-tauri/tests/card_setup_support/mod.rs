use alkemyst_core::{
    catalog::{Catalog, CatalogSource},
    db::Engine,
};
use alkemyst_desktop::{NativeRuntime, RuntimeOptions, Tracking};
use std::{path::Path, time::Duration};

pub fn options(database: &Path) -> RuntimeOptions {
    RuntimeOptions {
        database: database.to_owned(),
        tracking: Tracking::Disabled,
        port: 0,
        development: false,
    }
}

pub fn asset(path: &Path, tutor: bool) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let db = rusqlite::Connection::open(path).unwrap();
    db.execute_batch("CREATE TABLE Cards(GrpId INTEGER,TitleId INTEGER,ExpansionCode TEXT,CollectorNumber TEXT,CollectorSuffix TEXT);
        CREATE TABLE Localizations_enUS(LocId INTEGER,Formatted INTEGER,Loc TEXT);").unwrap();
    let (name, number) = if tutor {
        ("Find the Blade", "2")
    } else {
        ("Forgeblade", "1")
    };
    db.execute("INSERT INTO Cards VALUES(100,1,'TST',?1,'')", [number])
        .unwrap();
    db.execute("INSERT INTO Localizations_enUS VALUES(1,1,?1)", [name])
        .unwrap();
    db.close().unwrap();
}

pub async fn metadata(path: &Path) {
    let engine = Engine::open(path, 4096).unwrap();
    Catalog::new(engine.clone())
        .import_jsonl(
            std::io::Cursor::new(include_str!("../../../../core/tests/fixtures/cards.jsonl")),
            CatalogSource {
                url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
                updated_at: "2026-10-21T00:00:00Z".into(),
            },
        )
        .await
        .unwrap();
    engine.close().await.unwrap();
}

pub async fn checked(runtime: &NativeRuntime, after: u64) -> alkemyst_desktop::CardCacheHealth {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let state = runtime.card_cache_status().await;
            if state.checks > after {
                return state;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap()
}

pub async fn call(runtime: &NativeRuntime, path: &str, ids: Option<&[u32]>) -> serde_json::Value {
    let connection = runtime.bootstrap().unwrap();
    let client = reqwest::Client::new();
    let request = if let Some(ids) = ids {
        client
            .post(format!("{}{path}", connection.base_url))
            .json(&serde_json::json!({"ids": ids}))
    } else {
        client.get(format!("{}{path}", connection.base_url))
    };
    request
        .bearer_auth(connection.token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap()
}
