use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxResponse;
use entities::{events_tx, prelude::*};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::str::from_utf8;

use crate::{error::ApiError, AXL_USDC_DENOM};

pub fn events_key(events: Vec<String>) -> String {
    events.concat()
}

pub async fn add_txs_to_db(
    address: String,
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

    let txs = new_txs
        .into_iter()
        .map(|tx| {
            // We look for events related to incoming USDC-ibc transactions to set the tx flags

            // The incoming denom should be USDC
            let is_usdc = tx
                .events
                .iter()
                .find(|e| e.r#type == "denomination_trace")
                .map(|e| {
                    println!(
                        "{:?}",
                        e.attributes
                            .iter()
                            .find(|a| a.key == "denom")
                            .map(|a| from_utf8(a.value.as_ref()))
                    );
                    e.attributes
                        .iter()
                        .find(|a| a.key == "denom")
                        .map(|a| from_utf8(a.value.as_ref()))
                        == Some(Ok(AXL_USDC_DENOM))
                })
                == Some(true);

            println!("is_usdc, {}", is_usdc);

            let amount = if is_usdc {
                // An amount has to be set (it's safe because we are querying events based on this event type)
                tx.events
                    .iter()
                    .find(|e| e.r#type == "fungible_token_packet")
                    .unwrap()
                    .attributes
                    .iter()
                    .find(|a| a.key == "amount")
                    .map(|a| from_utf8(a.value.as_ref()))
                    .transpose()?
                    .map(|s| s.to_string())
            } else {
                None
            };

            Ok(events_tx::ActiveModel {
                address: Set(address.clone()),
                tx_hash: Set(tx.txhash),
                tx_events: Set(tx.logs.into()),
                timestamp: Set(tx.timestamp),
                kado_amount: Set(amount),
                ..Default::default()
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    EventsTx::insert_many(txs).exec(db).await?;

    Ok(())
}

pub async fn has_had_fee_grant(
    grantee: String,
    txhash: String,
    db: &DatabaseConnection,
) -> Result<(), ApiError> {
    let existing_tx = events_tx::Entity::find()
        .filter(events_tx::Column::Address.eq(grantee))
        .filter(events_tx::Column::TxHash.eq(txhash))
        .one(db)
        .await?;

    // Into ActiveModel
    let mut existing_tx: events_tx::ActiveModel = existing_tx.unwrap().into();

    // Update name attribute
    existing_tx.has_fee_grant = Set(1);
    existing_tx.update(db).await?;

    Ok(())
}

pub async fn tx_was_deposited(
    grantee: String,
    txhash: String,
    db: &DatabaseConnection,
) -> Result<(), ApiError> {
    let existing_tx = events_tx::Entity::find()
        .filter(events_tx::Column::Address.eq(grantee))
        .filter(events_tx::Column::TxHash.eq(txhash))
        .one(db)
        .await?;

    // Into ActiveModel
    let mut existing_tx: events_tx::ActiveModel = existing_tx.unwrap().into();

    // Update name attribute
    existing_tx.executed = Set(1);
    existing_tx.update(db).await?;

    Ok(())
}
