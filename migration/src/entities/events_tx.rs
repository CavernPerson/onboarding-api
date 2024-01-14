use sea_orm_migration::sea_orm::DeriveIden;

#[derive(DeriveIden)]
pub enum EventsTx {
    Table,
    Id,
    Address,
    TxHash,
    TxEvents,
    Timestamp,
    KadoAmount,
    HasFeeGrant,
    Executed,
}
