use crate::domain::{CreateDeck, ImportDeck, NewNote, NewVersion};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateDeckCommand {
    pub idempotency_key: String,
    pub input: CreateDeck,
}
#[derive(Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateVersionCommand {
    pub idempotency_key: String,
    pub input: NewVersion,
}
#[derive(Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportDeckCommand {
    pub idempotency_key: String,
    pub input: ImportDeck,
}
#[derive(Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateNoteCommand {
    pub idempotency_key: String,
    pub input: NewNote,
}
#[derive(Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ArenaIds {
    pub ids: Vec<u32>,
}
#[derive(Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct References {
    pub references: Vec<String>,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ArenaExport {
    pub text: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct PageQuery {
    #[serde(default = "page_limit")]
    #[param(minimum = 1, maximum = 100, default = 40)]
    pub limit: usize,
    pub cursor: Option<String>,
}
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct NotesQuery {
    pub deck_id: String,
    #[serde(default = "page_limit")]
    #[param(minimum = 1, maximum = 100, default = 40)]
    pub limit: usize,
    pub cursor: Option<String>,
}
fn page_limit() -> usize {
    40
}
