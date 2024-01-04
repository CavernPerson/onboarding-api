//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;

use crate::log::TxLogs;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "events_tx")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub source_events: String,
    pub tx_hash: String,
    pub tx_events: TxLogs,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}