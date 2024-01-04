use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxResponse;
use entities::{events_info, events_tx, prelude::*};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::error::ApiError;

pub fn events_key(events: Vec<String>) -> String {
    events.concat()
}

pub async fn add_txs_to_db(
    events: Vec<String>,
    new_txs: Vec<TxResponse>,
    db: &DatabaseConnection,
) -> Result<(), ApiError> {
    if new_txs.is_empty() {
        return Ok(());
    }
    let txhashes = new_txs
        .iter()
        .map(|tx| tx.txhash.clone())
        .collect::<Vec<_>>();
    log::info!("to save {:?}", txhashes);
    let this_events_key = events_key(events.clone());

    let txs: Vec<_> = new_txs
        .into_iter()
        .map(|tx| events_tx::ActiveModel {
            source_events: Set(this_events_key.clone()),
            tx_hash: Set(tx.txhash),
            tx_events: Set(tx.logs.into()),
            ..Default::default()
        })
        .collect();

    EventsTx::insert_many(txs).exec(db).await?;

    Ok(())
}

pub async fn set_events_info(
    events: Vec<String>,
    old_object: Option<events_info::Model>,
    page: u64,
    db: &DatabaseConnection,
) -> Result<(), ApiError> {
    let is_update = old_object.is_some();
    // This line just creates the model to insert in the db
    let mut object = old_object
        .map(Into::into)
        .unwrap_or(events_info::ActiveModel {
            events: Set(events_key(events)),
            ..Default::default()
        });

    object.current_page = Set(page);

    if is_update {
        object.update(db).await?;
    } else {
        object.insert(db).await?;
    }

    Ok(())
}
