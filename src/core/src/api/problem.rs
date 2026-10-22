use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Unauthorized,
    Forbidden,
    InvalidRequest,
    NotFound,
    Conflict,
    UnresolvedCards,
    UnsupportedModel,
    Busy,
    Internal,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Problem {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub problem: Problem,
}

impl ApiError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            problem: Problem {
                code,
                message: message.into(),
                unresolved: None,
            },
        }
    }
    pub fn missing() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            ErrorCode::NotFound,
            "Object not found",
        )
    }
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::InvalidRequest,
            message,
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.problem)).into_response()
    }
}

impl From<crate::chat::Error> for ApiError {
    fn from(error: crate::chat::Error) -> Self {
        use crate::chat::Error;
        match error {
            Error::Database(error) => error.into(),
            Error::Invalid(message) => Self::invalid(message),
            Error::Conflict => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::Conflict,
                "Idempotency key already used for different conversation content",
            ),
            Error::StaleRevision => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::Conflict,
                "Conversation revision changed; reload before appending",
            ),
            Error::NotFound => Self::missing(),
            Error::Json(_) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "Invalid stored conversation data",
            ),
        }
    }
}

impl From<crate::domain::Error> for ApiError {
    fn from(error: crate::domain::Error) -> Self {
        use crate::domain::Error;
        match error {
            Error::Invalid(message) => Self::invalid(message),
            Error::Conflict => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::Conflict,
                "Idempotency key already used for a different request",
            ),
            Error::NotFound => Self::missing(),
            Error::UnknownCards(cards) => {
                let mut error = Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ErrorCode::UnresolvedCards,
                    "Some cards could not be resolved",
                );
                error.problem.unresolved = Some(cards);
                error
            }
            Error::Database(error) => error.into(),
            Error::Json(_) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "Invalid stored data",
            ),
        }
    }
}

impl From<crate::db::Error> for ApiError {
    fn from(error: crate::db::Error) -> Self {
        use crate::db::Error;
        match error {
            Error::Closed => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::Busy,
                "Local storage unavailable",
            ),
            Error::Sql(rusqlite::Error::SqliteFailure(error, _))
                if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY =>
            {
                Self::invalid("Invalid deck or version reference")
            }
            _ => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "Local storage operation failed",
            ),
        }
    }
}

impl From<crate::catalog::Error> for ApiError {
    fn from(error: crate::catalog::Error) -> Self {
        use crate::catalog::Error;
        match error {
            Error::Invalid(message) => Self::invalid(message),
            Error::Busy => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::Busy,
                "A catalog refresh is already running",
            ),
            Error::Database(error) => error.into(),
            _ => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::Busy,
                "Card refresh failed; the local catalog remains available",
            ),
        }
    }
}

impl From<crate::ingest::Error> for ApiError {
    fn from(error: crate::ingest::Error) -> Self {
        match error {
            crate::ingest::Error::Database(error) => error.into(),
            _ => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "Capture status unavailable",
            ),
        }
    }
}

impl From<crate::analysis::Error> for ApiError {
    fn from(error: crate::analysis::Error) -> Self {
        match error {
            crate::analysis::Error::Database(error) => error.into(),
            crate::analysis::Error::Invalid(message) => Self::invalid(message),
            crate::analysis::Error::Conflict => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::Conflict,
                "Idempotency key already used",
            ),
            crate::analysis::Error::QueueFull => Self::new(
                StatusCode::TOO_MANY_REQUESTS,
                ErrorCode::Busy,
                "Simulation queue is full",
            ),
            crate::analysis::Error::NotFound => Self::missing(),
            crate::analysis::Error::Unsupported => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::UnsupportedModel,
                "Deck has missing or unsupported model inputs",
            ),
            crate::analysis::Error::Stopped => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::Busy,
                "Simulation worker is stopping",
            ),
            crate::analysis::Error::Model(_) => Self::invalid("Invalid simulation configuration"),
            crate::analysis::Error::Stored => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "Invalid stored simulation state",
            ),
            crate::analysis::Error::Json(_) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "Invalid stored analysis input",
            ),
        }
    }
}

impl From<crate::practice::Error> for ApiError {
    fn from(error: crate::practice::Error) -> Self {
        use crate::practice::Error;
        match error {
            Error::Database(e) => e.into(),
            Error::Analysis(e) => e.into(),
            Error::Invalid(message) => Self::invalid(message),
            Error::NotFound => Self::missing(),
            Error::Conflict => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::Conflict,
                "Idempotency conflict or stale practice revision",
            ),
            Error::Incompatible => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::UnsupportedModel,
                "Practice version is incompatible",
            ),
            Error::Json(_) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "Invalid stored practice state",
            ),
        }
    }
}
