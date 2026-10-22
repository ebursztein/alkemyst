use super::{
    AppState,
    extract::{ApiJson, ApiQuery},
    models::{CreateNoteCommand, NotesQuery},
    problem::{ApiError, Problem},
};
use crate::domain::{Note, NotePage, Origin};
use axum::{Json, extract::State};

/// List notes without changing captured history.
#[utoipa::path(get,path="/v1/notes",operation_id="notesList",params(NotesQuery),
    responses((status=200,body=NotePage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn list(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<NotesQuery>,
) -> Result<Json<NotePage>, ApiError> {
    Ok(Json(
        state
            .app
            .decks
            .notes(&query.deck_id, query.limit, query.cursor)
            .await?,
    ))
}

/// Add an idempotent local note alongside a deck or version.
#[utoipa::path(post,path="/v1/notes",operation_id="notesCreate",request_body=CreateNoteCommand,
    responses((status=200,body=Note),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn create(
    State(state): State<AppState>,
    ApiJson(command): ApiJson<CreateNoteCommand>,
) -> Result<Json<Note>, ApiError> {
    Ok(Json(
        state
            .app
            .decks
            .create_note(&command.idempotency_key, command.input, Origin::User)
            .await?,
    ))
}
