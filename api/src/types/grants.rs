use cosmos_sdk_proto::cosmos::feegrant::v1beta1::{BasicAllowance, Grant};
use cosmwasm_std::{Coin, StdError, Uint128};
use serde::Serialize;
use std::str::FromStr;

#[derive(Serialize)]
pub struct CosmosBasicAllowance {
    pub spend_limit: Vec<cosmwasm_std::Coin>,
}
impl TryInto<CosmosBasicAllowance> for BasicAllowance {
    fn try_into(self) -> Result<CosmosBasicAllowance, Self::Error> {
        Ok(CosmosBasicAllowance {
            spend_limit: self
                .spend_limit
                .into_iter()
                .map(|s| {
                    Ok::<_, StdError>(Coin {
                        denom: s.denom,
                        amount: Uint128::from_str(&s.amount)?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    type Error = StdError;
}

#[derive(Serialize)]
pub struct BasicAllowanceGrant {
    pub granter: String,
    pub grantee: String,
    pub allowance: Option<CosmosBasicAllowance>,
}

#[derive(Serialize)]
pub struct CosmosGrant {
    pub granter: String,
    pub grantee: String,
    pub allowance: Option<(String, Vec<u8>)>,
}

impl From<Grant> for CosmosGrant {
    fn from(val: Grant) -> Self {
        CosmosGrant {
            granter: val.granter,
            grantee: val.grantee,
            allowance: val.allowance.map(|a| (a.type_url, a.value)),
        }
    }
}

#[derive(Serialize)]
pub enum QuerierGrant {
    BasicAllowance(BasicAllowanceGrant),
    AnyAllowance(CosmosGrant),
}

#[derive(Serialize)]
pub enum GrantSimulationResult {
    Present(QuerierGrant),
    None,
}
