use crate::error::ApiError;
use crate::types::grants::{BasicAllowanceGrant, QuerierGrant};
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use cosmos_sdk_proto::cosmos::feegrant::v1beta1::{
    BasicAllowance, MsgGrantAllowance, MsgRevokeAllowance,
};
use cosmos_sdk_proto::traits::{Message, Name};
use cosmos_sdk_proto::Any;
use cw_orch::daemon::queriers::{DaemonQuerier, Feegrant};
use cw_orch::daemon::DaemonAsync;
use tokio::sync::MutexGuard;
use tonic::transport::Channel;

const FEE_DENOM: &str = "uluna";
const FEE_GRANT_AMOUNT: u128 = 100_000;
const MIN_FEE_GRANT_AMOUNT: u128 = 20_000;

pub async fn get_current_fee_grants(
    chain: Channel,
    granter: String,
    grantee: String,
) -> Result<Option<QuerierGrant>, ApiError> {
    let fee_grant = Feegrant::new(chain.clone());
    let grant = fee_grant.allowance(granter, grantee).await.ok();

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
    let granter = daemon.sender().to_string();
    // Check the existing fee grants this address has
    let existing_grants =
        get_current_fee_grants(daemon.channel().clone(), granter.clone(), grantee.clone()).await?;

    // We don't set a grant if there's already a grant and if it's sufficient
    if let Some(QuerierGrant::BasicAllowance(allowance)) = &existing_grants {
        if let Some(allowance) = &allowance.allowance {
            let amount = allowance
                .spend_limit
                .iter()
                .find(|c| c.denom == FEE_DENOM)
                .map(|c| c.amount)
                .unwrap_or_default();
            if amount.u128() >= MIN_FEE_GRANT_AMOUNT {
                return Ok("Already enough allowance for this address".to_string());
            }
        }
    }
    let mut msgs = vec![];
    if existing_grants.is_some() {
        msgs.push(Any {
            type_url: MsgRevokeAllowance::type_url(),
            value: MsgRevokeAllowance {
                granter: granter.clone(),
                grantee: grantee.clone(),
            }
            .encode_to_vec(),
        })
    }

    let allowance_type = BasicAllowance {
        spend_limit: vec![Coin {
            amount: FEE_GRANT_AMOUNT.to_string(),
            denom: FEE_DENOM.to_string(),
        }],
        expiration: None,
    };

    let fee_grant = MsgGrantAllowance {
        granter: granter.clone(),
        grantee: grantee.clone(),
        allowance: Some(Any {
            type_url: BasicAllowance::type_url(),
            value: allowance_type.encode_to_vec(),
        }),
    };

    msgs.push(Any {
        type_url: MsgGrantAllowance::type_url(),
        value: fee_grant.encode_to_vec(),
    });

    daemon.sender.commit_tx_any(msgs, None).await?;

    Ok("Tx succesfully submitted".to_string())
}
