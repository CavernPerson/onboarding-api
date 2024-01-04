use std::{env, sync::Arc};

use crate::{
    fee_grants::get_current_fee_grants, tx_indexer::fetch_new_txs,
    types::grants::GrantSimulationResult,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use cw_orch::daemon::{
    networks::{PHOENIX_1, PISCO_1},
    DaemonAsync, DaemonAsyncBuilder,
};
use error::ApiError;
use fee_grants::grant;
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::Mutex;
use tonic::transport::Channel;

struct AppState {
    daemon: Mutex<DaemonAsync>,
    channel: Channel,
    db: DatabaseConnection,
}

pub mod db_helpers;
pub mod error;
pub mod fee_grants;
pub mod tx_indexer;
pub mod types;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    dotenv::dotenv()?;
    pretty_env_logger::init();

    let chain = PISCO_1;
    // let mut chain = PHOENIX_1;
    // chain.grpc_urls = &["https://phoenix-grpc.terra.dev/"];
    let daemon = DaemonAsyncBuilder::default()
        .chain(chain.clone())
        .build()
        .await?;
    let db = Database::connect(env::var("DATABASE_URL")?).await?;

    let shared_state = Arc::new(AppState {
        channel: daemon.channel(),
        daemon: Mutex::new(daemon),
        db,
    });

    // Build our application with a route
    let app = Router::new()
        .route("/", get(|| async { "Hello World" }))
        .route("/fee-grant/:address", post(grant_fee_to))
        .route("/fee-grant/:address", get(simulate_grant_fee_to))
        .route("/index/:address", get(index_address))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[axum_macros::debug_handler]
async fn grant_fee_to(
    Path(address): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<String, ApiError> {
    // Check the existing fee grants this address has
    let existing_grants = get_current_fee_grants(state.channel.clone(), address.clone()).await?;

    // We don't set a grant if there's already a grant and if it's sufficient
    if existing_grants.is_some() {
        return Ok("Grant already set".to_string());
    }

    // We check in our database that this address hasn't already received a grant for this specific transaction
    grant(&state.daemon.lock().await, address).await
}

#[axum_macros::debug_handler]
async fn simulate_grant_fee_to(
    Path(address): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<GrantSimulationResult>, ApiError> {
    // Check the existing fee grants this address has
    let existing_grants = get_current_fee_grants(state.channel.clone(), address.clone()).await?;

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
    fetch_new_txs(
        vec![format!("message.sender='{}'", address)],
        state.channel.clone(),
        &state.db,
    )
    .await
}
