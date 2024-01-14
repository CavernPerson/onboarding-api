use crate::entities::{events_info::EventsInfo, events_tx::EventsTx};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EventsTx::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EventsTx::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EventsTx::Address).string().not_null())
                    .col(ColumnDef::new(EventsTx::TxHash).string().not_null())
                    .col(ColumnDef::new(EventsTx::TxEvents).json().not_null())
                    .col(ColumnDef::new(EventsTx::Timestamp).string().not_null())
                    .col(ColumnDef::new(EventsTx::KadoAmount).string())
                    .col(
                        ColumnDef::new(EventsTx::HasFeeGrant)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(EventsTx::Executed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(EventsInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EventsInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EventsInfo::Events).string().not_null())
                    .col(
                        ColumnDef::new(EventsInfo::CurrentPage)
                            .big_unsigned()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EventsTx::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(EventsInfo::Table).to_owned())
            .await
    }
}
