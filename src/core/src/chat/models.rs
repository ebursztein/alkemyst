use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewConversation {
    #[schema(min_length = 1, max_length = 200)]
    pub title: String,
    pub version_id: Option<String>,
    #[serde(default)]
    pub share_deck: bool,
    #[serde(default)]
    pub share_notes: bool,
    #[serde(default)]
    pub share_history: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub version_id: Option<String>,
    pub created_at: i64,
    pub revision: u32,
    pub share_deck: bool,
    pub share_notes: bool,
    pub share_history: bool,
}
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewMessage {
    #[schema(min_length = 1, max_length = 16384)]
    pub body: String,
    #[schema(maximum = 1999)]
    pub expected_revision: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    Tool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sequence: u32,
    pub role: MessageRole,
    pub body: String,
    pub created_at: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ConversationPage {
    pub items: Vec<Conversation>,
    pub next_cursor: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MessagePage {
    pub items: Vec<Message>,
    pub next_cursor: Option<String>,
}
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in=Query)]
pub struct ChatQuery {
    #[serde(default = "page_limit")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: usize,
    #[param(min_length = 19, max_length = 19)]
    pub cursor: Option<String>,
}
fn page_limit() -> usize {
    40
}
impl Default for ChatQuery {
    fn default() -> Self {
        Self {
            limit: page_limit(),
            cursor: None,
        }
    }
}
