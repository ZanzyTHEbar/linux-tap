//! Same-artifact conversion lookup/execution; no key loading, discovery or rebuilding.
use super::{ENDPOINT, SelectedCoin, epoch_of, verify_effects, verify_template};
use crate::{Prepared, client, probe, settlement::Outcome};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sui_rpc::{
    client::ExecuteAndWaitError,
    field::{FieldMask, FieldMaskUtil},
    proto::sui::rpc::v2::{ExecuteTransactionRequest, ExecutedTransaction, GetTransactionRequest},
};
use sui_sdk_types::{Digest, Transaction, TransactionEffects, UserSignature};
use tap_core::SUI_TESTNET_CHAIN;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub endpoint: String,
    pub chain: Option<Digest>,
    pub outcome: Outcome,
    pub execution: Option<ExecutedTransaction>,
    pub error: Option<String>,
}

impl Observation {
    fn unknown(error: impl ToString) -> Self {
        Self {
            endpoint: ENDPOINT.into(),
            chain: None,
            outcome: Outcome::Unknown,
            execution: None,
            error: Some(error.to_string().chars().take(2048).collect()),
        }
    }
    pub fn validate(&self, coin: &SelectedCoin, prepared: &Prepared) -> Result<()> {
        verify_template(coin, &prepared.transaction()?)?;
        ensure!(self.endpoint == ENDPOINT, "observation endpoint mismatch");
        if let Some(chain) = self.chain {
            ensure!(
                chain == SUI_TESTNET_CHAIN.parse()?,
                "observation genesis mismatch"
            );
        }
        match self.outcome {
            Outcome::VerifiedSuccess { .. } | Outcome::VerifiedFailure { .. } => {
                ensure!(
                    self.chain == Some(SUI_TESTNET_CHAIN.parse()?) && self.error.is_none(),
                    "invalid verified observation"
                );
                ensure!(
                    self.outcome
                        == classify(
                            coin,
                            prepared,
                            self.execution.as_ref().context("missing execution")?
                        )?,
                    "observation outcome mismatch"
                );
            }
            Outcome::NotFound => ensure!(
                self.chain == Some(SUI_TESTNET_CHAIN.parse()?)
                    && self.execution.is_none()
                    && self.error.is_none(),
                "invalid NotFound observation"
            ),
            Outcome::Unknown => ensure!(
                self.error
                    .as_ref()
                    .is_some_and(|e| !e.is_empty() && e.len() <= 8192),
                "unknown observation requires bounded error"
            ),
        }
        Ok(())
    }
}

fn classify(
    coin: &SelectedCoin,
    prepared: &Prepared,
    executed: &ExecutedTransaction,
) -> Result<Outcome> {
    let tx = prepared.transaction()?;
    verify_template(coin, &tx)?;
    let checkpoint = executed
        .checkpoint
        .context("execution lacks checkpoint inclusion")?;
    let returned = executed
        .transaction
        .as_ref()
        .context("missing transaction")?;
    let observed: Transaction = returned.try_into()?;
    ensure!(
        observed == tx
            && returned.digest.as_deref() == Some(prepared.digest.as_str())
            && executed.digest.as_deref() == Some(prepared.digest.as_str()),
        "observed transaction/digest mismatch"
    );
    ensure!(
        executed.signatures.len() == 1
            && UserSignature::try_from(&executed.signatures[0])? == prepared.signature,
        "observed signature differs from immutable artifact"
    );
    let raw = executed.effects.as_ref().context("missing effects")?;
    ensure!(
        raw.transaction_digest.as_deref() == Some(prepared.digest.as_str()),
        "effects digest mismatch"
    );
    let effects: TransactionEffects = raw.try_into()?;
    let TransactionEffects::V2(v2) = &effects else {
        anyhow::bail!("V2 effects required")
    };
    ensure!(
        v2.transaction_digest == tx.digest()
            && v2.epoch == epoch_of(&tx)?
            && v2.gas_object_index.is_none(),
        "effects identity/epoch/gas mismatch"
    );
    match &v2.status {
        sui_sdk_types::ExecutionStatus::Success => {
            ensure!(
                raw.status.as_ref().and_then(|s| s.success) == Some(true),
                "status mismatch"
            );
            verify_effects(&tx, coin, &effects)?;
            Ok(Outcome::VerifiedSuccess { checkpoint })
        }
        sui_sdk_types::ExecutionStatus::Failure { .. } => {
            ensure!(
                raw.status.as_ref().and_then(|s| s.success) == Some(false),
                "status mismatch"
            );
            Ok(Outcome::VerifiedFailure { checkpoint })
        }
    }
}

fn observed(
    coin: &SelectedCoin,
    prepared: &Prepared,
    execution: Option<ExecutedTransaction>,
) -> Observation {
    let result = execution
        .as_ref()
        .context("missing execution")
        .and_then(|e| classify(coin, prepared, e));
    let (outcome, error) = match result {
        Ok(outcome) => (outcome, None),
        Err(e) => (
            Outcome::Unknown,
            Some(
                format!("unverified execution: {e:#}")
                    .chars()
                    .take(2048)
                    .collect(),
            ),
        ),
    };
    Observation {
        endpoint: ENDPOINT.into(),
        chain: SUI_TESTNET_CHAIN.parse().ok(),
        outcome,
        execution,
        error,
    }
}

async fn check_node(coin: &SelectedCoin, prepared: &Prepared) -> Result<()> {
    verify_template(coin, &prepared.transaction()?)?;
    let node = probe(ENDPOINT).await?;
    ensure!(
        node.chain.as_deref() == Some("testnet") && node.chain_id == SUI_TESTNET_CHAIN,
        "pinned testnet required"
    );
    Ok(())
}

/// Digest lookup always precedes current coin/epoch checks, including after coin consumption.
pub async fn query(coin: &SelectedCoin, prepared: &Prepared) -> Observation {
    let operation = async {
        check_node(coin, prepared).await?;
        let mut c = client(ENDPOINT)?;
        let mut req = GetTransactionRequest::default();
        req.digest = Some(prepared.digest.clone());
        req.read_mask = Some(FieldMask::from_paths(["*"]));
        match c.ledger_client().get_transaction(req).await {
            Ok(response) => Ok(observed(coin, prepared, response.into_inner().transaction)),
            Err(e) if e.code() as i32 == 5 => Ok(Observation {
                endpoint: ENDPOINT.into(),
                chain: Some(SUI_TESTNET_CHAIN.parse()?),
                outcome: Outcome::NotFound,
                execution: None,
                error: None,
            }),
            Err(e) => Err(anyhow::anyhow!("lookup failed: {e}")),
        }
    };
    match tokio::time::timeout(Duration::from_secs(30), operation).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => Observation::unknown(format!("{e:#}")),
        Err(_) => Observation::unknown("lookup timed out; retain saved bytes"),
    }
}

/// Only call AFTER a durable approved dispatch intent/attempt is committed by the CLI.
pub async fn execute_saved(coin: &SelectedCoin, prepared: &Prepared) -> Observation {
    let operation = async {
        check_node(coin, prepared).await?;
        let mut c = client(ENDPOINT)?;
        let mut req = ExecuteTransactionRequest::new(prepared.transaction()?.into());
        req.signatures = vec![prepared.signature.clone().into()];
        req.read_mask = Some(FieldMask::from_paths(["*"]));
        match c
            .execute_transaction_and_wait_for_checkpoint(req, Duration::from_secs(40))
            .await
        {
            Ok(response) => {
                Ok::<_, anyhow::Error>(observed(coin, prepared, response.into_inner().transaction))
            }
            Err(e) => {
                let mut observation = Observation::unknown(&e);
                observation.execution = match e {
                    ExecuteAndWaitError::CheckpointTimeout(response)
                    | ExecuteAndWaitError::CheckpointStreamError { response, .. } => {
                        response.into_inner().transaction
                    }
                    _ => None,
                };
                Ok(observation)
            }
        }
    };
    match tokio::time::timeout(Duration::from_secs(60), operation).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => Observation::unknown(format!("{e:#}")),
        Err(_) => Observation::unknown(
            "whole execution operation timed out; possibly sent; retain saved bytes",
        ),
    }
}

/// Index availability is separate from checkpoint conversion success; failure is only pending.
pub async fn address_balance(coin: &SelectedCoin) -> Result<u64> {
    use sui_rpc::proto::sui::rpc::v2::GetBalanceRequest;
    let mut c = client(ENDPOINT)?;
    let mut req = GetBalanceRequest::default();
    req.owner = Some(coin.owner.to_string());
    req.coin_type = Some(super::POLICY.coin_type().into());
    let balance = tokio::time::timeout(Duration::from_secs(15), c.state_client().get_balance(req))
        .await??
        .into_inner()
        .balance
        .context("missing indexed balance")?;
    ensure!(
        balance.coin_type.as_deref() == Some(super::POLICY.coin_type()),
        "indexed asset mismatch"
    );
    balance
        .address_balance
        .context("address balance not indexed")
}
