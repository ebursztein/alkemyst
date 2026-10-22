use super::*;
use crate::{
    db::{Mutation, Write},
    statements::chat,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[derive(Deserialize)]
struct Receipt {
    fingerprint: String,
    value: Option<Conversation>,
}
#[derive(Deserialize)]
struct MessageReceipt {
    value: Option<Message>,
}
fn write(statement: Mutation, params: Vec<Value>) -> Write {
    Write {
        statement,
        params,
        expected_rows: None,
    }
}
impl Chat {
    pub async fn create(&self, key: &str, mut input: NewConversation) -> Result<Conversation> {
        validation::key(key)?;
        input.title = input.title.trim().into();
        validation::text(&input.title, 200)?;
        if let Some(id) = &input.version_id {
            validation::text(id, 256)?;
        }
        if (input.share_deck || input.share_notes || input.share_history)
            && input.version_id.is_none()
        {
            return Err(Error::Invalid(
                "sharing context requires a selected deck version",
            ));
        }
        // Freeze the v1 receipt format independently of DTO field ordering/extensions.
        let canonical = (
            1,
            &input.title,
            &input.version_id,
            input.share_deck,
            input.share_notes,
            input.share_history,
        );
        let fingerprint: String = Sha256::digest(serde_json::to_vec(&canonical)?)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        self.db
            .write(vec![write(
                chat::CREATE,
                vec![
                    json!(uuid::Uuid::new_v4().to_string()),
                    json!(key),
                    json!(fingerprint),
                    json!(input.title),
                    json!(input.version_id),
                    json!(input.share_deck),
                    json!(input.share_notes),
                    json!(input.share_history),
                ],
            )])
            .await?;
        let receipt: Receipt = reads::row(self.db.read(chat::RECEIPT, vec![json!(key)]).await?)?
            .ok_or(Error::NotFound)?;
        if receipt.fingerprint != fingerprint {
            return Err(Error::Conflict);
        }
        receipt.value.ok_or(Error::NotFound)
    }
    pub async fn append_user(&self, id: &str, key: &str, input: NewMessage) -> Result<Message> {
        validation::text(id, 256)?;
        validation::key(key)?;
        validation::text(&input.body, 16384)?;
        if input.expected_revision >= 2000 {
            return Err(Error::Invalid("conversation message limit reached"));
        }
        self.db
            .write(vec![write(
                chat::APPEND_USER,
                vec![
                    json!(uuid::Uuid::new_v4().to_string()),
                    json!(input.expected_revision),
                    json!(key),
                    json!(input.body),
                    json!(id),
                ],
            )])
            .await?;
        let receipt: MessageReceipt = reads::row(
            self.db
                .read(chat::MESSAGE_RECEIPT, vec![json!(id), json!(key)])
                .await?,
        )?
        .ok_or(Error::NotFound)?;
        let message = receipt.value.ok_or(Error::StaleRevision)?;
        if message.body != input.body || message.sequence != input.expected_revision + 1 {
            return Err(Error::Conflict);
        }
        Ok(message)
    }
    pub async fn delete(&self, id: &str) -> Result<()> {
        validation::text(id, 256)?;
        self.db
            .write(vec![
                write(chat::CLEAR_MESSAGES, vec![json!(id)]),
                write(chat::DELETE, vec![json!(id)]),
            ])
            .await?;
        Ok(())
    }
}
