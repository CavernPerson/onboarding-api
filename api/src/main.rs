use std::{env, sync::Arc};

use crate::{
    db_helpers::{has_had_fee_grant, tx_was_deposited},
    fee_grants::get_current_fee_grants,
    tx_indexer::fetch_new_txs,
    types::grants::GrantSimulationResult,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use cw_orch::daemon::{networks::PHOENIX_1, DaemonAsync, DaemonAsyncBuilder};
use entities::events_tx;
use error::ApiError;
use fee_grants::grant;
use sea_orm::{
    ColumnTrait, Database, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tower_http::cors::CorsLayer;
use tx_indexer::{get_tx_count, get_tx_total, get_txs};
pub struct AppState {
    sender: String,
    daemon: Mutex<DaemonAsync>,
    channel: Channel,
    db: DatabaseConnection,
}

pub mod db_helpers;
pub mod error;
pub mod fee_grants;
pub mod tx_indexer;
pub mod types;

pub const AXL_USDC_DENOM: &str =
    "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
const PAGINATION_LIMIT: u64 = 100;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    dotenv::dotenv()?;
    pretty_env_logger::init();

    let mut chain = PHOENIX_1;
    // chain.grpc_urls = &["http://terra-grpc.polkachu.com:11790"];
    // chain.grpc_urls = &["https://phoenix-grpc.terra.dev/"];
    chain.grpc_urls = &["https://terra2-grpc.lavenderfive.com:443"];
    let daemon = DaemonAsyncBuilder::default()
        .chain(chain.clone())
        .build()
        .await?;

    let sender = daemon.sender().to_string();
    let db = Database::connect(env::var("DATABASE_URL")?).await?;

    let shared_state = Arc::new(AppState {
        channel: daemon.channel(),
        daemon: Mutex::new(daemon),
        sender,
        db,
    });

    // Build our application with a route
    let app = Router::new()
        .route("/fee-grant/:address/:txhash", post(grant_fee_to))
        .route("/fee-grant/:address", get(simulate_grant_fee_to))
        .route("/index/:address", get(index_address))
        .route("/txs/:address", get(get_txs))
        .route("/tx-total/:address", get(get_tx_total))
        .route("/tx-count/:address", get(get_tx_count))
        .route("/executed/:address/:txhash", post(is_tx_executed))
        .with_state(shared_state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[axum_macros::debug_handler]
async fn grant_fee_to(
    Path((address, txhash)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<String, ApiError> {
    // We verify the tx hash exists
    let tx_spotted = events_tx::Entity::find()
        .filter(events_tx::Column::Address.eq(address.clone()))
        .filter(events_tx::Column::TxHash.eq(txhash.clone()))
        .filter(events_tx::Column::HasFeeGrant.eq(false))
        .count(&state.db)
        .await?;

    println!("Tx Spotted {:?}", tx_spotted);

    // if tx_spotted == 0 {
    //     return Ok("No transaction allowed to get a fee grant".to_string());
    // }

    // We grant if it doesn't exist
    let grant_response = grant(&state.daemon.lock().await, address.clone()).await?;

    // When the grant function is done, and doesn't error, it means the user has enough grant for depositing
    // We can save that in the database
    has_had_fee_grant(address, txhash, &state.db).await?;

    Ok(grant_response)
}

#[axum_macros::debug_handler]
async fn simulate_grant_fee_to(
    Path(address): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<GrantSimulationResult>, ApiError> {
    // Check the existing fee grants this address has
    let existing_grants =
        get_current_fee_grants(state.channel.clone(), state.sender.clone(), address.clone())
            .await?;

    // We don't set a grant if there's already a grant and if it's sufficient
    if let Some(grant) = existing_grants {
        return Ok(GrantSimulationResult::Present(grant).into());
    }

    Ok(GrantSimulationResult::None.into())
}

#[axum_macros::debug_handler]
async fn index_address(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<(), ApiError> {
    // We fetch new transactions
    let state = state;
    fetch_new_txs(address, state.channel.clone(), &state.db).await
}

#[axum_macros::debug_handler]
async fn is_tx_executed(
    Path((address, txhash)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<(), ApiError> {
    tx_was_deposited(address, txhash, &state.db).await
}
