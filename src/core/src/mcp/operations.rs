use crate::{
    Application,
    api::{
        ArenaExport, ArenaIds, CreateDeckCommand, CreateNoteCommand, CreateVersionCommand,
        ImportDeckCommand, NotesQuery, PageQuery, References, problem::ApiError,
    },
    domain::Origin,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

pub async fn invoke(app: &Application, name: &str, args: Value) -> Result<Value, ApiError> {
    match name {
        "systemDiscovery" => encode(crate::api::discovery::discover()),
        "cardsSearch" => encode(app.catalog.search(decode(args)?).await?),
        "cardsGet" => encode(
            app.catalog
                .card(&string(&args, "oracle_id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "cardsResolveArena" => {
            let input: ArenaIds = decode(args)?;
            encode(app.catalog.resolve_arena(&input.ids).await?)
        }
        "cardsResolveReferences" => {
            let input: References = decode(args)?;
            encode(app.catalog.resolve_references(&input.references).await?)
        }
        "catalogStatusGet" => encode(app.catalog.status().await?),
        "catalogRefresh" => encode(app.refresh_catalog().await?),
        "decksList" => {
            let page: PageQuery = decode(args)?;
            encode(app.decks.list(page.limit, page.cursor).await?)
        }
        "decksCreate" => {
            let command: CreateDeckCommand = decode(args)?;
            encode(
                app.decks
                    .create(&command.idempotency_key, command.input, Origin::Agent)
                    .await?,
            )
        }
        "deckVersionCreate" => {
            let command: CreateVersionCommand = decode(args)?;
            encode(
                app.decks
                    .create_version(&command.idempotency_key, command.input, Origin::Agent)
                    .await?,
            )
        }
        "deckVersionGet" => encode(
            app.decks
                .version(&string(&args, "id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "deckArenaExport" => encode(ArenaExport {
            text: app.decks.export_arena(&string(&args, "id")?).await?,
        }),
        "deckArenaImport" => {
            let command: ImportDeckCommand = decode(args)?;
            encode(
                app.decks
                    .import_arena(&command.idempotency_key, command.input, Origin::Agent)
                    .await?,
            )
        }
        "notesList" => {
            let page: NotesQuery = decode(args)?;
            encode(
                app.decks
                    .notes(&page.deck_id, page.limit, page.cursor)
                    .await?,
            )
        }
        "notesCreate" => {
            let command: CreateNoteCommand = decode(args)?;
            encode(
                app.decks
                    .create_note(&command.idempotency_key, command.input, Origin::Agent)
                    .await?,
            )
        }
        _ => Err(ApiError::missing()),
    }
}
fn decode<T: DeserializeOwned>(value: Value) -> Result<T, ApiError> {
    serde_json::from_value(value)
        .map_err(|_| ApiError::invalid("Arguments do not match the tool input schema"))
}
fn string(value: &Value, key: &str) -> Result<String, ApiError> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| ApiError::invalid("Required string argument missing"))
}
fn encode(value: impl Serialize) -> Result<Value, ApiError> {
    Ok(json!({"data":serde_json::to_value(value).expect("Application result serialization")}))
}
