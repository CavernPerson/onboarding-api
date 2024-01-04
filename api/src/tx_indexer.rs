use crate::db_helpers::{add_txs_to_db, events_key, set_events_info};
use crate::error::ApiError;
use cosmos_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{GetTxsEventRequest, GetTxsEventResponse, OrderBy};

use entities::prelude::*;
use entities::{events_info, events_tx};
use futures::future::try_join_all;
use redis_serde_json::RedisJsonValue;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
#[derive(Default, Serialize, Deserialize, RedisJsonValue)]
pub struct EventsStatus {
    current_page: u64,
}

const PAGINATION_LIMIT: u64 = 1;

/// Gets the transactions on the events page
async fn get_last_txs(
    channel: Channel,
    events: Vec<String>,
    page: u64,
) -> Result<GetTxsEventResponse, ApiError> {
    let mut client = ServiceClient::new(channel);
    log::info!("Fetching page {}", page);

    #[allow(deprecated)]
    let request = GetTxsEventRequest {
        events: events.clone(),
        page,
        limit: PAGINATION_LIMIT,
        pagination: None, // This is not used, so good.
        order_by: OrderBy::Asc.into(),
    };

    let tx_result = client.get_txs_event(request.clone()).await?.into_inner();

    Ok(tx_result)
}

pub async fn fetch_new_txs(
    events: Vec<String>,
    channel: Channel,
    db: &DatabaseConnection,
) -> Result<(), ApiError> {
    // First we query the existing transactions
    let current_events_card = events_tx::Entity::find()
        .filter(events_tx::Column::SourceEvents.eq(events_key(events.clone())))
        .count(db)
        .await?;

    let mut current_page = current_events_card / PAGINATION_LIMIT + 1;

    let mut local_count = current_events_card;
    // Now we get all new txs until there is no more transactions
    loop {
        let tx_result = get_last_txs(channel.clone(), events.clone(), current_page).await?;
        let fetched_txs = tx_result.tx_responses;

        log::debug!(
            "Fetched tx hashes : {:?} - {events:?}",
            fetched_txs
                .iter()
                .map(|tx| tx.txhash.clone())
                .collect::<Vec<_>>(),
        );

        // Select only new txs (not in db)
        let new_txs_filter = try_join_all(fetched_txs.iter().map(|tx| async {
            Ok::<_, ApiError>(
                events_tx::Entity::find()
                    .filter(events_tx::Column::SourceEvents.eq(events_key(events.clone())))
                    .filter(events_tx::Column::TxHash.eq(tx.txhash.clone()))
                    .count(db)
                    .await?
                    == 0,
            )
        }))
        .await?;

        let new_txs: Vec<_> = fetched_txs
            .into_iter()
            .zip(new_txs_filter)
            .filter_map(|(tx, filter)| if filter { Some(tx) } else { None })
            .collect();

        log::debug!(
            "New tx hashes : {:?}",
            new_txs
                .iter()
                .map(|tx| tx.txhash.clone())
                .collect::<Vec<_>>(),
        );

        let temp_count = local_count + new_txs.len() as u64;
        add_txs_to_db(events.clone(), new_txs, db).await?;

        if temp_count == tx_result.total {
            // If the number of element matches the total :
            // - We don't increment the page number as a page might be partially full
            // - We stop querying new transaction
            return Ok(());
        }

        // In any other case, we update the underlying object and try again for other transactions
        current_page += 1;
        local_count = temp_count;
        set_events_info(
            events.clone(),
            current_events_state.clone(),
            current_page,
            db,
        )
        .await?
    }
}
