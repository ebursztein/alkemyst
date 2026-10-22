use super::{
    AppState,
    problem::{ApiError, ErrorCode},
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use subtle::ConstantTimeEq;

#[derive(Clone)]
pub struct AuthConfig {
    token: Arc<str>,
    development: bool,
}

impl AuthConfig {
    pub fn new(token: String, development: bool) -> Result<Self, std::io::Error> {
        if token.len() < 32 || token.len() > 256 || !token.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "local credential must have 32..256 printable ASCII characters",
            ));
        }
        Ok(Self {
            token: token.into(),
            development,
        })
    }

    pub fn generate() -> Self {
        Self {
            token: format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            )
            .into(),
            development: false,
        }
    }

    /// Native bootstrap uses this value; it must never be placed in logs or URLs.
    pub fn credential(&self) -> &str {
        &self.token
    }

    fn origin_allowed(&self, origin: &str) -> bool {
        [
            "tauri://localhost",
            "http://tauri.localhost",
            "https://tauri.localhost",
        ]
        .contains(&origin)
            || self.development
                && ["http://localhost:1420", "http://127.0.0.1:1420"].contains(&origin)
    }
}

fn host_allowed(headers: &HeaderMap) -> bool {
    headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(|host| host.parse::<axum::http::uri::Authority>().ok())
        .is_some_and(|authority| ["127.0.0.1", "localhost", "[::1]"].contains(&authority.host()))
}

pub(super) async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if !host_allowed(request.headers()) {
        return ApiError::new(
            StatusCode::FORBIDDEN,
            ErrorCode::Forbidden,
            "Loopback host required",
        )
        .into_response();
    }
    let origin = request.headers().get(header::ORIGIN).cloned();
    if origin.as_ref().is_some_and(|origin| {
        !origin
            .to_str()
            .is_ok_and(|origin| state.auth.origin_allowed(origin))
    }) {
        return ApiError::new(
            StatusCode::FORBIDDEN,
            ErrorCode::Forbidden,
            "Origin not allowed",
        )
        .into_response();
    }
    if request.method() == Method::OPTIONS {
        if origin.is_none() {
            return ApiError::invalid("Preflight requires an origin").into_response();
        }
        return cors(StatusCode::NO_CONTENT.into_response(), origin);
    }
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_once(' '))
        .is_some_and(|(scheme, credential)| {
            scheme.eq_ignore_ascii_case("Bearer")
                && bool::from(credential.as_bytes().ct_eq(state.auth.token.as_bytes()))
        });
    if !authorized {
        return cors(
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Local bearer credential required",
            )
            .into_response(),
            origin,
        );
    }
    cors(next.run(request).await, origin)
}

fn cors(mut response: Response, origin: Option<HeaderValue>) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    if response.status() == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Bearer realm=\"Alkemyst\""),
        );
    }
    if let Some(origin) = origin {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Origin"));
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Authorization, Content-Type"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
    }
    response
}
