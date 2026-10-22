//! Developer CLI for the same catalog operations used by REST and MCP.
use alkemyst_core::{
    catalog::{Catalog, CatalogSource, Search},
    db::Engine,
};
use std::io::BufReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = args.first().map(String::as_str).unwrap_or("");
    let path = args
        .get(1)
        .ok_or("usage: catalog <import|refresh|search|resolve|status> <db> [arguments]")?;
    let catalog = Catalog::new(Engine::open(std::path::Path::new(path), 8 * 1024 * 1024)?);
    let result = match command {
        "import" => {
            let file = std::fs::File::open(args.get(2).ok_or("gzip path required")?)?;
            let source = CatalogSource {
                url: args.get(3).ok_or("source URL required")?.clone(),
                updated_at: args.get(4).ok_or("bulk timestamp required")?.clone(),
            };
            serde_json::to_value(
                catalog
                    .import_jsonl(
                        BufReader::new(flate2::read::MultiGzDecoder::new(file)),
                        source,
                    )
                    .await?,
            )?
        }
        "arena" => {
            serde_json::json!({"aliases":catalog.import_arena_database(std::path::Path::new(args.get(2).ok_or("Arena database path required")?)).await?})
        }
        "refresh" => serde_json::to_value(catalog.refresh().await?)?,
        "search" => serde_json::to_value(
            catalog
                .search(Search {
                    text: args.get(2).cloned().unwrap_or_default(),
                    limit: 10,
                    ..Search::default()
                })
                .await?,
        )?,
        "resolve" => {
            let ids = args
                .iter()
                .skip(2)
                .map(|id| id.parse())
                .collect::<Result<Vec<u32>, _>>()?;
            let resolved = catalog.resolve_arena(&ids).await?;
            serde_json::Value::Array(
                resolved
                    .into_iter()
                    .map(|r| {
                        serde_json::json!({
                            "arena_id":r.arena_id,"resolved_by":r.resolved_by, "name":r.card.as_ref().map(|c|&c["name"]),
                        })
                    })
                    .collect(),
            )
        }
        "status" => serde_json::to_value(catalog.status().await?)?,
        _ => return Err("unknown catalog command".into()),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
