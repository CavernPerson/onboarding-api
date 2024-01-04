use crate::error::ApiError;
use crate::types::grants::{BasicAllowanceGrant, QuerierGrant};
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use cosmos_sdk_proto::cosmos::feegrant::v1beta1::{BasicAllowance, MsgGrantAllowance};
use cosmos_sdk_proto::traits::{Message, Name};
use cosmos_sdk_proto::Any;
use cw_orch::daemon::queriers::{DaemonQuerier, Feegrant};
use cw_orch::daemon::DaemonAsync;
use tokio::sync::MutexGuard;
use tonic::transport::Channel;

const GRANTER: &str = "terra1v7k3k4qw9y9juwma2d3rw6psmnd3vyg8sehwx6";
const FEE_DENOM: &str = "uluna";
const FEE_GRANT_AMOUNT: u128 = 100_000;

pub async fn get_current_fee_grants(
    chain: Channel,
    grantee: String,
) -> Result<Option<QuerierGrant>, ApiError> {
    let fee_grant = Feegrant::new(chain.clone());
    let grant = fee_grant.allowance(GRANTER, grantee).await.ok();

    // We try to decode the grant basic allowance
    let basic_allowance = grant
        .map(|g| {
            match g
                .allowance
                .as_ref()
                .map(Any::to_msg::<BasicAllowance>)
                .transpose()
            {
                Ok(basic_allowance) => {
                    Ok::<_, ApiError>(QuerierGrant::BasicAllowance(BasicAllowanceGrant {
                        granter: g.granter,
                        grantee: g.grantee,
                        allowance: basic_allowance.map(TryInto::try_into).transpose()?,
                    }))
                }
                Err(_) => Ok(QuerierGrant::AnyAllowance(g.into())),
            }
        })
        .transpose()?;

    Ok(basic_allowance)
}

pub async fn grant(
    daemon: &MutexGuard<'_, DaemonAsync>,
    grantee: String,
) -> Result<String, ApiError> {
    let allowance_type = BasicAllowance {
        spend_limit: vec![Coin {
            amount: FEE_GRANT_AMOUNT.to_string(),
            denom: FEE_DENOM.to_string(),
        }],
        expiration: None,
    };

    let fee_grant = MsgGrantAllowance {
        granter: GRANTER.to_string(),
        grantee,
        allowance: Some(Any {
            type_url: BasicAllowance::type_url(),
            value: allowance_type.encode_to_vec(),
        }),
    };

    daemon
        .sender
        .commit_tx_any(
            vec![Any {
                type_url: MsgGrantAllowance::type_url(),
                value: fee_grant.encode_to_vec(),
            }],
            None,
        )
        .await?;

    Ok("Tx succesfully submitted".to_string())
}
