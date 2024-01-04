use sea_orm_migration::sea_orm::DeriveIden;

#[derive(DeriveIden)]
pub enum EventsTx {
    Table,
    Id,
    SourceEvents,
    TxHash,
    TxEvents,
}
