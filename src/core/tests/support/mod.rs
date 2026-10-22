use alkemyst_core::catalog::CatalogSource;

pub fn source() -> CatalogSource {
    CatalogSource {
        url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
        updated_at: "2026-10-21T21:00:00Z".into(),
    }
}

pub fn cards() -> String {
    include_str!("../fixtures/cards.jsonl").to_owned()
}
