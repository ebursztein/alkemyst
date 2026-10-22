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
        "practiceCreate" => {
            let c: crate::api::CreatePracticeCommand = decode(args)?;
            encode(
                app.practice
                    .create(&c.idempotency_key, c.input, Origin::Agent)
                    .await?,
            )
        }
        "practiceList" => encode(app.practice.list(decode(args)?).await?),
        "practiceGet" => encode(
            app.practice
                .get(&string(&args, "id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "practiceInputsGet" => encode(
            app.practice
                .inputs(&string(&args, "id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "practiceAct" => {
            let id = string(&args, "id")?;
            let mut command = args;
            command
                .as_object_mut()
                .expect("object arguments")
                .remove("id");
            let c: crate::api::PracticeActionCommand = decode(command)?;
            encode(
                app.practice
                    .act(
                        &c.idempotency_key,
                        &id,
                        c.expected_revision,
                        c.action,
                        Origin::Agent,
                    )
                    .await?,
            )
        }

        "deckVersionsCompare" => encode(
            app.analysis
                .compare(&string(&args, "id")?, &string(&args, "candidate_id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "simulationsCreate" => {
            let c: crate::api::CreateSimulationCommand = decode(args)?;
            encode(
                app.analysis
                    .submit_simulation(&c.idempotency_key, c.input, Origin::Agent)
                    .await?,
            )
        }
        "simulationsList" => encode(app.analysis.simulations(decode(args)?).await?),
        "simulationGet" => encode(
            app.analysis
                .simulation(&string(&args, "id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "simulationInputsGet" => encode(
            app.analysis
                .simulation_inputs(&string(&args, "id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "simulationCancel" => encode(
            app.analysis
                .cancel_simulation(&string(&args, "id")?, Origin::Agent)
                .await?,
        ),
        "probabilityDraw" => encode(crate::analysis::probabilities::draw(decode(args)?)?),
        "probabilityJoint" => encode(crate::analysis::probabilities::joint(decode(args)?)?),
        "analysisProfileGet" => encode(
            app.analysis
                .profile(&string(&args, "id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "gamesList" => encode(app.history.games(decode(args)?).await?),
        "gameGet" => encode(
            app.history
                .game(&string(&args, "id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "gameTimeline" => {
            let id = string(&args, "id")?;
            encode(
                app.history
                    .timeline(&id, decode(args)?)
                    .await?
                    .ok_or_else(ApiError::missing)?,
            )
        }
        "gameTurns" => {
            let id = string(&args, "id")?;
            encode(
                app.history
                    .turns(&id, decode(args)?)
                    .await?
                    .ok_or_else(ApiError::missing)?,
            )
        }
        "matchesList" => encode(app.history.matches(decode(args)?).await?),
        "matchGet" => encode(
            app.history
                .match_summary(&string(&args, "id")?)
                .await?
                .ok_or_else(ApiError::missing)?,
        ),
        "statisticsGet" => encode(app.history.statistics(decode(args)?).await?),
        "captureStatusGet" => encode(app.ingest.status().await?),
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
        "deckVersionsList" => {
            let page: PageQuery = decode(args.clone())?;
            encode(
                app.decks
                    .versions(&string(&args, "id")?, page.limit, page.cursor)
                    .await?
                    .ok_or_else(ApiError::missing)?,
            )
        }
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
