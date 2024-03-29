//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;
use serde::Serialize;

use crate::log::TxLogs;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "events_tx")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub address: String,
    pub tx_hash: String,
    pub tx_events: TxLogs,
    pub timestamp: String,
    pub kado_amount: Option<String>,
    pub has_fee_grant: i8,
    pub executed: i8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
