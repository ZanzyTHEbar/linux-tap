//! Explicit testnet-only observation and exact-artifact execution. No key loading or rebuilding.
use super::{
    Prepared, client, probe, verify_execution_identity, verify_execution_with_policy,
    verify_template_with_policy,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sui_rpc::{
    client::ExecuteAndWaitError,
    field::{FieldMask, FieldMaskUtil},
    proto::sui::rpc::v2::{ExecuteTransactionRequest, ExecutedTransaction, GetTransactionRequest},
};
use sui_sdk_types::{Digest, UserSignature};
use tap_core::{DirectAssetPolicy, Invoice};
mod transport_trace;
pub use transport_trace::ExecutionTransportTrace;

const POLICY: DirectAssetPolicy = DirectAssetPolicy::CircleTestnetUsdc;
const MASK: [&str; 6] = [
    "digest",
    "transaction",
    "signatures",
    "effects",
    "checkpoint",
    "timestamp",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Outcome {
    Unknown,
    NotFound,
    VerifiedSuccess { checkpoint: u64 },
    VerifiedFailure { checkpoint: u64 },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub endpoint: String,
    pub chain: Option<Digest>,
    pub outcome: Outcome,
    pub execution: Option<ExecutedTransaction>,
    pub error: Option<String>,
}

impl Observation {
    fn unknown(endpoint: &str, error: impl ToString) -> Self {
        Self {
            endpoint: endpoint.into(),
            chain: None,
            outcome: Outcome::Unknown,
            execution: None,
            error: Some(error.to_string().chars().take(2048).collect()),
        }
    }

    /// Validate historical observation integrity, not current eligibility or RPC authenticity.
    pub fn validate(&self, invoice: &Invoice, prepared: &Prepared) -> Result<()> {
        POLICY.validate(invoice)?;
        super::validate_endpoint(&self.endpoint)?;
        if let Some(chain) = self.chain {
            ensure!(
                chain == Digest::new(invoice.chain),
                "observation chain mismatch"
            );
        }
        match &self.outcome {
            Outcome::VerifiedSuccess { .. } | Outcome::VerifiedFailure { .. } => {
                ensure!(
                    self.chain == Some(Digest::new(invoice.chain)) && self.error.is_none(),
                    "invalid verified observation"
                );
                ensure!(
                    self.outcome
                        == classify(
                            invoice,
                            prepared,
                            self.execution
                                .as_ref()
                                .context("missing observed execution")?
                        )?,
                    "observation outcome mismatch"
                );
            }
            Outcome::NotFound => ensure!(
                self.chain == Some(Digest::new(invoice.chain))
                    && self.execution.is_none()
                    && self.error.is_none(),
                "invalid NotFound observation"
            ),
            Outcome::Unknown => ensure!(
                self.error
                    .as_ref()
                    .is_some_and(|e| !e.is_empty() && e.len() <= 8192),
                "unknown observation lacks bounded error"
            ),
        }
        Ok(())
    }
}

fn classify(
    invoice: &Invoice,
    prepared: &Prepared,
    execution: &ExecutedTransaction,
) -> Result<Outcome> {
    let checkpoint = verify_execution_identity(invoice, prepared, execution, POLICY)?;
    ensure!(
        execution
            .transaction
            .as_ref()
            .and_then(|tx| tx.digest.as_deref())
            == Some(prepared.digest.as_str()),
        "observed inner transaction digest mismatch"
    );
    ensure!(
        execution.signatures.len() == 1
            && UserSignature::try_from(&execution.signatures[0])? == prepared.signature,
        "observed signatures differ from saved artifact"
    );
    match execution
        .effects
        .as_ref()
        .and_then(|e| e.status.as_ref())
        .and_then(|s| s.success)
    {
        Some(true) => {
            verify_execution_with_policy(invoice, prepared, execution, POLICY)?;
            Ok(Outcome::VerifiedSuccess { checkpoint })
        }
        Some(false) => Ok(Outcome::VerifiedFailure { checkpoint }),
        None => anyhow::bail!("unknown execution status"),
    }
}

fn observed(
    endpoint: &str,
    invoice: &Invoice,
    prepared: &Prepared,
    execution: Option<ExecutedTransaction>,
) -> Observation {
    let classification = execution
        .as_ref()
        .context("node omitted execution")
        .and_then(|e| classify(invoice, prepared, e));
    let (outcome, error) = match classification {
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
        endpoint: endpoint.into(),
        chain: Some(Digest::new(invoice.chain)),
        outcome,
        execution,
        error,
    }
}

async fn check_node(endpoint: &str, invoice: &Invoice, prepared: &Prepared) -> Result<()> {
    POLICY.validate(invoice)?;
    verify_template_with_policy(invoice, &prepared.transaction()?, POLICY)?;
    let info = probe(endpoint).await?;
    ensure!(
        info.chain.as_deref() == Some("testnet")
            && info.chain_id.parse::<Digest>()? == Digest::new(invoice.chain),
        "pinned testnet node required"
    );
    Ok(())
}

/// Query the saved digest before considering fresh balance, epoch or simulation eligibility.
pub async fn query(endpoint: &str, invoice: &Invoice, prepared: &Prepared) -> Observation {
    let result = async {
        check_node(endpoint, invoice, prepared).await?;
        let mut c = client(endpoint)?;
        let mut request = GetTransactionRequest::default();
        request.digest = Some(prepared.digest.clone());
        request.read_mask = Some(FieldMask::from_paths(MASK));
        match tokio::time::timeout(
            Duration::from_secs(15),
            c.ledger_client().get_transaction(request),
        )
        .await?
        {
            Ok(response) => Ok(observed(
                endpoint,
                invoice,
                prepared,
                response.into_inner().transaction,
            )),
            // Canonical gRPC status code 5 is NOT_FOUND, not evidence of an unspent transaction.
            Err(e) if e.code() as i32 == 5 => Ok(Observation {
                endpoint: endpoint.into(),
                chain: Some(Digest::new(invoice.chain)),
                outcome: Outcome::NotFound,
                execution: None,
                error: None,
            }),
            Err(e) => Err(anyhow::anyhow!("lookup failed: {e}")),
        }
    }
    .await;
    result.unwrap_or_else(|e| Observation::unknown(endpoint, format!("{e:#}")))
}

/// Execute exact stored SDK bytes/signature only. Caller MUST commit dispatch intent first.
/// Every helper error is conservative unknown, retaining any available raw execution response.
pub async fn execute_saved(endpoint: &str, invoice: &Invoice, prepared: &Prepared) -> Observation {
    execute_inner(endpoint, invoice, prepared, None).await
}

/// Observe the actual Execute transport future without changing SDK recovery or RPC policy.
pub async fn execute_saved_with_trace(
    endpoint: &str,
    invoice: &Invoice,
    prepared: &Prepared,
) -> (Observation, ExecutionTransportTrace) {
    let trace = transport_trace::Trace::default();
    let observation = execute_inner(endpoint, invoice, prepared, Some(trace.clone())).await;
    (observation, trace.result())
}

async fn execute_inner(
    endpoint: &str,
    invoice: &Invoice,
    prepared: &Prepared,
    trace: Option<transport_trace::Trace>,
) -> Observation {
    let operation = async {
        check_node(endpoint, invoice, prepared).await?;
        let mut c = client(endpoint)?;
        if let Some(trace) = trace {
            c = trace.attach(c);
        }
        let request = execution_request(invoice, prepared)?;
        // The SDK's timeout starts at its later checkpoint wait, not subscription/execute.
        match c
            .execute_transaction_and_wait_for_checkpoint(request, Duration::from_secs(40))
            .await
        {
            Ok(response) => Ok::<Observation, anyhow::Error>(observed(
                endpoint,
                invoice,
                prepared,
                response.into_inner().transaction,
            )),
            Err(e) => {
                let mut observation = Observation::unknown(endpoint, &e);
                observation.chain = Some(Digest::new(invoice.chain));
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
        Ok(Ok(observation)) => observation,
        Ok(Err(e)) => Observation::unknown(endpoint, format!("{e:#}")),
        Err(_) => Observation::unknown(
            endpoint,
            "whole execution operation timed out; outcome unknown; saved bytes retained",
        ),
    }
}

/// Construct the actual fixed-policy execution request via official SDK conversions, without RPC.
/// This does not grant submission authority; the caller owns approval and the durable intent barrier.
pub fn execution_request(
    invoice: &Invoice,
    prepared: &Prepared,
) -> Result<ExecuteTransactionRequest> {
    let tx = prepared.transaction()?;
    verify_template_with_policy(invoice, &tx, POLICY)?;
    let mut request = ExecuteTransactionRequest::new(tx.into());
    request.signatures = vec![prepared.signature.clone().into()];
    request.read_mask = Some(FieldMask::from_paths(MASK));
    Ok(request)
}

/// Canonical official-protobuf payloads for a separate, explicitly armed operator lab.
#[derive(Debug)]
pub struct ExecutionLabPayloads {
    pub execution: Vec<u8>,
    pub simulation: Vec<u8>,
}

/// Verify the immutable signed artifact, then encode the same requests used by native dispatch.
/// These are protobuf messages, not gRPC frames, and grant no permission to execute.
pub fn execution_lab_payloads(
    invoice: &Invoice,
    prepared: &Prepared,
) -> Result<ExecutionLabPayloads> {
    use prost::Message;
    let execution = execution_request(invoice, prepared)?.encode_to_vec();
    let simulation =
        crate::qualification::simulation_request(&prepared.transaction()?).encode_to_vec();
    Ok(ExecutionLabPayloads {
        execution,
        simulation,
    })
}
