use super::*;
use crate::{
    db::{Engine, Mutation, Write},
    statements::simulations,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::sync::{Notify, oneshot};
use tokio_util::sync::CancellationToken;
#[derive(Deserialize)]
pub(super) struct Owned {
    pub id: String,
    pub compiler_version: String,
    pub model_version: String,
    pub policy_version: String,
    pub replay_version: String,
    pub inputs: Value,
}
pub(super) async fn mutate(
    db: &Engine,
    statement: Mutation,
    params: Vec<Value>,
) -> Result<(), Error> {
    db.write(vec![Write {
        statement,
        params,
        expected_rows: None,
    }])
    .await?;
    Ok(())
}
pub(super) async fn run(
    db: Engine,
    stop: CancellationToken,
    notify: Arc<Notify>,
    ready: oneshot::Sender<()>,
) {
    let owner = uuid::Uuid::new_v4().to_string();
    let mut ready = Some(ready);
    loop {
        if stop.is_cancelled() {
            break;
        }
        let next = async {
            mutate(&db, simulations::RECOVER, vec![]).await?;
            mutate(&db, simulations::CLAIM, vec![json!(owner)]).await?;
            store::row::<Owned>(db.read(simulations::OWNED, vec![json!(owner)]).await?)
        }
        .await;
        if let Some(ready) = ready.take() {
            let _ = ready.send(());
        }
        match next {
            Ok(Some(job)) => {
                super::execution::execute(&db, job, &owner, &stop).await;
                continue;
            }
            Ok(None) => {}
            Err(_) => {
                tokio::select! {_=stop.cancelled()=>break,_=notify.notified()=>{},_=tokio::time::sleep(Duration::from_secs(1))=>{}};
                continue;
            }
        }
        tokio::select! {_=stop.cancelled()=>break,_=notify.notified()=>{},_=tokio::time::sleep(Duration::from_secs(30))=>{}};
    }
}
