use std::env::VarError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cosmwasm_std::StdError;
use cw_orch::daemon::DaemonError;
use redis::RedisError;
use sea_orm::DbErr;
use thiserror::Error;
use tonic::Status;
#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    DaemonError(#[from] DaemonError),

    #[error(transparent)]
    TonicError(#[from] Status),

    #[error(transparent)]
    RedisError(#[from] RedisError),

    #[error(transparent)]
    DbErr(#[from] DbErr),

    #[error(transparent)]
    EnvVarError(#[from] VarError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    DotenvError(#[from] dotenv::Error),

    #[error(transparent)]
    ProstError(#[from] cosmos_sdk_proto::prost::DecodeError),

    #[error(transparent)]
    StdError(#[from] StdError),
}

pub type ApiResult<T = ()> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {:?}", self),
        )
            .into_response()
    }
}
