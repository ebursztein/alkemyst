//! Native-only ingestion utility. Emits counts/health, never raw events or identities.
use alkemyst_core::{db::Engine, ingest::Ingestor};
use serde_json::json;
use std::{io, path::PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let operation = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    let database = PathBuf::from(args.next().ok_or_else(usage)?);
    let paths = args.map(PathBuf::from).collect::<Vec<_>>();
    let db = Engine::open(&database, 8 * 1024 * 1024)?;
    let ingest = Ingestor::new(db.clone());
    match operation.as_str() {
        "import" if !paths.is_empty() => {
            for path in paths {
                ingest.import_file(&path).await?;
            }
            println!("{}", serde_json::to_string(&ingest.status().await?)?);
        }
        "status" if paths.is_empty() => {
            println!("{}", serde_json::to_string(&ingest.status().await?)?)
        }
        "watch" if paths.is_empty() || paths.len() == 2 => {
            let tracking = if paths.is_empty() {
                ingest.discover_and_watch()?
            } else {
                ingest.watch(paths[0].clone(), paths[1].clone())
            };
            let mut health = tracking.subscribe();
            println!("{}", json!({"pid":std::process::id(),"state":"starting"}));
            loop {
                tokio::select! {
                    result=tokio::signal::ctrl_c()=>{result?;break;},
                    result=health.changed()=>{
                        if result.is_err(){break;}
                        println!("{}",serde_json::to_string(&*health.borrow())?);
                    }
                }
            }
            tracking.stop().await?;
            println!("{}", serde_json::to_string(&ingest.status().await?)?);
        }
        _ => return Err(usage().into()),
    }
    db.close().await?;
    Ok(())
}
fn usage() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "usage: capture <import|status|watch> <database> [log paths]; watch accepts previous/current paths or native discovery",
    )
}
