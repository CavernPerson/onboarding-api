use cosmrs::proto::cosmos::base::abci::v1beta1::AbciMessageLog;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct TxLogs(Vec<TxLog>);

impl From<Vec<AbciMessageLog>> for TxLogs {
    fn from(value: Vec<AbciMessageLog>) -> Self {
        Self(value.into_iter().map(|log| log.into()).collect())
    }
}

// The custom struct must derive `FromJsonQueryResult`, `Serialize` and `Deserialize`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct TxLog {
    pub msg_index: u32,
    pub log: String,
    pub events: Vec<TxEvent>,
}

/// StringEvent defines en Event object wrapper where all the attributes
/// contain key/value pairs that are strings instead of raw bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct TxEvent {
    pub event_type: String,
    pub attributes: Vec<TxAttribute>,
}
/// Attribute defines an attribute wrapper where the key and value are
/// strings instead of raw bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct TxAttribute {
    pub key: String,
    pub value: String,
}

impl From<AbciMessageLog> for TxLog {
    fn from(value: AbciMessageLog) -> Self {
        Self {
            msg_index: value.msg_index,
            log: value.log,
            events: value
                .events
                .into_iter()
                .map(|e| TxEvent {
                    event_type: e.r#type,
                    attributes: e
                        .attributes
                        .into_iter()
                        .map(|a| TxAttribute {
                            key: a.key,
                            value: a.value,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}
