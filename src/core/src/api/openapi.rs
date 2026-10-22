use super::{cards, decks, discovery, history, notes};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(crate::health::build_info, discovery::get, history::games,history::game,history::timeline,history::turns,history::matches,history::match_detail,history::statistics,history::capture,
        decks::list,decks::create,decks::version,decks::create_version,decks::import,decks::export,
        notes::list,notes::create,cards::search,cards::get,cards::arena,cards::references,cards::status,cards::refresh),
    modifiers(&Security), security(("localBearer"=[])),
    info(title="Alkemyst API"),
    servers((url="http://127.0.0.1:47831",description="Local Alkemyst service"))
)]
pub struct ApiDoc;

struct Security;
impl utoipa::Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = &mut openapi.components {
            components.add_security_scheme(
                "localBearer",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}
