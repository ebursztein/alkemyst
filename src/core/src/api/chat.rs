use super::{
    AppState,
    extract::{ApiJson, ApiPath, ApiQuery},
    problem::{ApiError, Problem},
};
use crate::chat::{
    ChatQuery, Conversation, ConversationPage, Message, MessagePage, NewConversation, NewMessage,
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateConversationCommand {
    #[schema(min_length = 1, max_length = 128)]
    idempotency_key: String,
    input: NewConversation,
}
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AppendMessageCommand {
    #[schema(min_length = 1, max_length = 128)]
    idempotency_key: String,
    input: NewMessage,
}
/// Create local history with frozen context choices. This does not contact an AI
/// provider. Retrying a deleted creation returns not found instead of recreating it.
#[utoipa::path(post,path="/v1/chat/conversations",operation_id="chatConversationsCreate",request_body=CreateConversationCommand,
    responses((status=200,body=Conversation),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn create(
    State(s): State<AppState>,
    ApiJson(c): ApiJson<CreateConversationCommand>,
) -> Result<Json<Conversation>, ApiError> {
    Ok(Json(s.app.chat.create(&c.idempotency_key, c.input).await?))
}
/// List local conversations newest first. Cursor namespace: c1.
#[utoipa::path(get,path="/v1/chat/conversations",operation_id="chatConversationsList",params(ChatQuery),
    responses((status=200,body=ConversationPage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn list(
    State(s): State<AppState>,
    ApiQuery(q): ApiQuery<ChatQuery>,
) -> Result<Json<ConversationPage>, ApiError> {
    Ok(Json(s.app.chat.list(q).await?))
}
/// Read the pinned local context and current message revision.
#[utoipa::path(get,path="/v1/chat/conversations/{id}",operation_id="chatConversationGet",params(("id"=String,Path)),
    responses((status=200,body=Conversation),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn get(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<Conversation>, ApiError> {
    Ok(Json(
        s.app.chat.get(&id).await?.ok_or_else(ApiError::missing)?,
    ))
}
/// Remove transcript content and context references. Keep only a minimal creation
/// tombstone for retry safety. Existing backups and SQLite recovery files are not erased.
#[utoipa::path(delete,path="/v1/chat/conversations/{id}",operation_id="chatConversationDelete",params(("id"=String,Path)),
    responses((status=204,description="Local conversation removed"),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn delete(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<StatusCode, ApiError> {
    s.app.chat.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
/// Read a bounded chronological message page. Cursor namespace: m1.
#[utoipa::path(get,path="/v1/chat/conversations/{id}/messages",operation_id="chatMessagesList",params(("id"=String,Path),ChatQuery),
    responses((status=200,body=MessagePage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn messages(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
    ApiQuery(q): ApiQuery<ChatQuery>,
) -> Result<Json<MessagePage>, ApiError> {
    Ok(Json(
        s.app
            .chat
            .messages(&id, q)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
/// Append user text at an expected revision. This stores history locally without
/// generating a reply. Clients cannot supply assistant or tool roles.
#[utoipa::path(post,path="/v1/chat/conversations/{id}/messages",operation_id="chatMessagesAppendUser",params(("id"=String,Path)),request_body=AppendMessageCommand,
    responses((status=200,body=Message),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn append(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
    ApiJson(c): ApiJson<AppendMessageCommand>,
) -> Result<Json<Message>, ApiError> {
    Ok(Json(
        s.app
            .chat
            .append_user(&id, &c.idempotency_key, c.input)
            .await?,
    ))
}
