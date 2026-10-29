//! Read-only qualification with explicit asset policy, never automatic substitution or signing.
use super::{build_direct_with_policy, client, probe, verify_template_with_policy};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::time::Duration;
use sui_rpc::{
    field::{FieldMask, FieldMaskUtil},
    proto::sui::rpc::v2::{
        Balance, GetBalanceRequest, GetCoinInfoRequest, GetCoinInfoResponse, GetEpochRequest,
        ProtocolConfig, SimulateTransactionRequest, SimulateTransactionResponse,
        simulate_transaction_request::TransactionChecks,
    },
};
use sui_sdk_types::{Address, Digest, Transaction, TransactionExpiration};
use tap_core::{DirectAssetPolicy, Invoice};

#[derive(Serialize)]
pub struct Qualification {
    pub endpoint: String,
    pub node: super::NodeInfo,
    pub asset: &'static str,
    pub sender: Address,
    pub receiver: Address,
    pub amount: u64,
    pub transaction_digest: String,
    pub transaction_bcs_hex: String,
    pub protocol: Option<ProtocolConfig>,
    pub coin_info: Option<GetCoinInfoResponse>,
    pub balance: Option<Balance>,
    pub simulation: Option<SimulateTransactionResponse>,
    pub blockers: Vec<String>,
    pub qualified: bool,
    pub chain_submission: bool,
}

/// Caller pins the full genesis digest independently of the endpoint. No keys are needed.
/// The terms are diagnostic, not a signed merchant invoice or approval to pay.
pub async fn qualify_testnet(
    endpoint: &str,
    expected_chain: Digest,
    sender: Address,
    receiver: Address,
    amount: u64,
) -> Result<Qualification> {
    qualify_testnet_with_policy(
        endpoint,
        expected_chain,
        sender,
        receiver,
        amount,
        DirectAssetPolicy::PinnedUsdc,
    )
    .await
}

/// The alternate asset is opt-in and bound to the fixed testnet genesis, not a network name.
pub async fn qualify_testnet_with_policy(
    endpoint: &str,
    expected_chain: Digest,
    sender: Address,
    receiver: Address,
    amount: u64,
    policy: DirectAssetPolicy,
) -> Result<Qualification> {
    let asset = policy.coin_type();
    let node = probe(endpoint).await?;
    ensure!(node.chain.as_deref() == Some("testnet"), "testnet required");
    ensure!(
        node.chain_id.parse::<Digest>()? == expected_chain,
        "chain pin mismatch"
    );
    let invoice = Invoice {
        chain: expected_chain.into_inner(),
        id: [0; 16],
        receiver: receiver.into_inner(),
        coin_type: asset.into(),
        amount,
        created_ms: 0,
        expires_ms: 0,
        merchant: "READ-ONLY QUALIFICATION; NOT A PAYMENT REQUEST".into(),
    };
    let tx = build_direct_with_policy(&invoice, sender, node.epoch, 0, policy)?;
    verify_template_with_policy(&invoice, &tx, policy)?;
    qualify_exact(endpoint, &invoice, &tx, policy, node).await
}

/// Qualify the actual frozen payment candidate without rebuilding its nonce, epoch or bytes.
/// This is eligibility only, not authorization or settlement. No key or execution RPC is used.
pub async fn qualify_candidate(
    endpoint: &str,
    invoice: &Invoice,
    tx: &Transaction,
    policy: DirectAssetPolicy,
) -> Result<Qualification> {
    verify_template_with_policy(invoice, tx, policy)?;
    let node = probe(endpoint).await?;
    ensure!(node.chain.as_deref() == Some("testnet"), "testnet required");
    ensure!(
        node.chain_id.parse::<Digest>()? == Digest::new(invoice.chain),
        "chain pin mismatch"
    );
    qualify_exact(endpoint, invoice, tx, policy, node).await
}

async fn qualify_exact(
    endpoint: &str,
    invoice: &Invoice,
    tx: &Transaction,
    policy: DirectAssetPolicy,
    node: super::NodeInfo,
) -> Result<Qualification> {
    let asset = policy.coin_type();
    let sender = tx.sender;
    let receiver = Address::new(invoice.receiver);
    let amount = invoice.amount;
    let mut report = Qualification {
        endpoint: endpoint.into(),
        node,
        asset,
        sender,
        receiver,
        amount,
        transaction_digest: tx.digest().to_string(),
        transaction_bcs_hex: hex::encode(bcs::to_bytes(tx)?),
        protocol: None,
        coin_info: None,
        balance: None,
        simulation: None,
        blockers: vec![],
        qualified: false,
        chain_submission: false,
    };
    if !matches!(tx.expiration, TransactionExpiration::ValidDuring { min_epoch: Some(min), max_epoch: Some(max), .. } if min == report.node.epoch && max == report.node.epoch)
    {
        report
            .blockers
            .push("candidate epoch is not current; saved bytes must not be replaced".into());
    }
    let mut c = client(endpoint)?;
    let mut epoch_req = GetEpochRequest::default();
    epoch_req.epoch = Some(report.node.epoch);
    epoch_req.read_mask = Some(FieldMask::from_paths(["epoch", "protocol_config"]));
    match tokio::time::timeout(
        Duration::from_secs(15),
        c.ledger_client().get_epoch(epoch_req),
    )
    .await
    {
        Ok(Ok(r)) => {
            report.protocol = r.into_inner().epoch.and_then(|e| e.protocol_config);
            if report
                .protocol
                .as_ref()
                .and_then(|p| p.protocol_version)
                .is_none()
            {
                report
                    .blockers
                    .push("node omitted protocol configuration/version".into());
            }
            if let Some(protocol) = &report.protocol {
                // Use the lossless configuration, not the deprecated string attributes.
                let json = serde_json::to_value(protocol)?;
                let configs = &json["configs"];
                for flag in [
                    "enable_gasless",
                    "enable_accumulators",
                    "enable_object_funds_withdraw",
                ] {
                    if configs[flag].as_bool() != Some(true) {
                        report
                            .blockers
                            .push(format!("protocol flag {flag} is disabled or unknown"));
                    }
                }
                let allowed = configs["gasless_allowed_token_types"]
                    .as_array()
                    .is_some_and(|rows| {
                        rows.iter().any(|row| {
                            row[0].as_str() == Some(asset)
                                && row[1]
                                    .as_str()
                                    .and_then(|s| s.parse::<u64>().ok())
                                    .is_some_and(|minimum| amount >= minimum)
                        })
                    });
                if !allowed {
                    report.blockers.push("pinned USDC/amount absent from protocol gasless allowlist or allowlist unknown".into());
                }
            }
        }
        Ok(Err(e)) => report.blockers.push(format!(
            "protocol read failed: {}: {}",
            e.code(),
            e.message()
        )),
        Err(_) => report.blockers.push("protocol read timed out".into()),
    }
    let mut coin_req = GetCoinInfoRequest::default();
    coin_req.coin_type = Some(asset.into());
    match tokio::time::timeout(
        Duration::from_secs(15),
        c.state_client().get_coin_info(coin_req),
    )
    .await
    {
        Ok(Ok(r)) => {
            let coin = r.into_inner();
            if coin.coin_type.as_deref() != Some(asset)
                || coin.metadata.as_ref().and_then(|m| m.decimals) != Some(6)
            {
                report
                    .blockers
                    .push("exact pinned USDC type/6-decimal metadata not confirmed".into());
            }
            report.coin_info = Some(coin);
        }
        Ok(Err(e)) => report.blockers.push(format!(
            "pinned USDC lookup failed: {}: {}",
            e.code(),
            e.message()
        )),
        Err(_) => report.blockers.push("pinned USDC lookup timed out".into()),
    }
    let mut balance_req = GetBalanceRequest::default();
    balance_req.owner = Some(sender.to_string());
    balance_req.coin_type = Some(asset.into());
    match tokio::time::timeout(
        Duration::from_secs(15),
        c.state_client().get_balance(balance_req),
    )
    .await
    {
        Ok(Ok(r)) => {
            report.balance = r.into_inner().balance;
            if !report.balance.as_ref().is_some_and(|b| {
                b.coin_type.as_deref() == Some(asset)
                    && b.address_balance.is_some_and(|n| n >= amount)
            }) {
                report.blockers.push("insufficient or unknown pinned USDC address balance (coin objects do not qualify)".into());
            }
        }
        Ok(Err(e)) => report.blockers.push(format!(
            "address-balance read failed: {}: {}",
            e.code(),
            e.message()
        )),
        Err(_) => report
            .blockers
            .push("address-balance read timed out".into()),
    }
    // Never disable checks, select gas, substitute an asset, or alter the exact template.
    let req = simulation_request(tx);
    match tokio::time::timeout(
        Duration::from_secs(15),
        c.execution_client().simulate_transaction(req),
    )
    .await
    {
        Ok(Ok(r)) => {
            let simulation = r.into_inner();
            if let Err(error) = verify_simulation(tx, &simulation) {
                report
                    .blockers
                    .push(format!("simulation not qualified: {error:#}"));
            }
            report.simulation = Some(simulation);
        }
        Ok(Err(e)) => report.blockers.push(format!(
            "checked exact-template simulation failed: {}: {}",
            e.code(),
            e.message()
        )),
        Err(_) => report
            .blockers
            .push("checked exact-template simulation timed out".into()),
    }
    // Epoch-scoped bytes must not be reported as qualified after an epoch boundary.
    let after = probe(endpoint).await?;
    ensure!(
        after.chain_id == report.node.chain_id,
        "chain changed during qualification"
    );
    ensure!(
        after.chain.as_deref() == Some("testnet"),
        "testnet required"
    );
    if after.epoch != report.node.epoch {
        report
            .blockers
            .push("epoch changed during qualification; repeat read-only qualification".into());
    }
    report.qualified = report.blockers.is_empty();
    Ok(report)
}

pub(crate) fn simulation_request(tx: &Transaction) -> SimulateTransactionRequest {
    let mut request = SimulateTransactionRequest::default();
    request.transaction = Some(tx.clone().into());
    request.read_mask = Some(FieldMask::from_paths(["*"]));
    request.checks = Some(TransactionChecks::Enabled as i32);
    request.do_gas_selection = Some(false);
    request
}

/// Validate a lab preparation request using official protobuf/transaction decoding and native policy.
/// Exact re-encoding rejects unknown fields, unchecked simulation and gas-selection variants.
pub fn validate_lab_simulation(
    invoice: &Invoice,
    expected_sender: Address,
    payload: &[u8],
) -> Result<()> {
    use prost::Message;
    let request = SimulateTransactionRequest::decode(payload)?;
    let tx = Transaction::try_from(
        request
            .transaction
            .as_ref()
            .context("simulation omits transaction")?,
    )?;
    ensure!(
        invoice.amount == 10_000 && tx.sender == expected_sender,
        "lab terms mismatch"
    );
    verify_template_with_policy(invoice, &tx, DirectAssetPolicy::CircleTestnetUsdc)?;
    ensure!(
        request == simulation_request(&tx) && request.encode_to_vec() == payload,
        "lab simulation must be the exact checked native request, without extra fields"
    );
    Ok(())
}

pub(crate) fn verify_simulation(
    tx: &Transaction,
    simulation: &SimulateTransactionResponse,
) -> Result<()> {
    let executed = simulation
        .transaction
        .as_ref()
        .context("missing simulated transaction")?;
    let returned = executed
        .transaction
        .as_ref()
        .context("missing simulated bytes")?;
    let observed: Transaction = returned.try_into()?;
    ensure!(observed == *tx, "simulation changed transaction");
    let expected_digest = tx.digest().to_string();
    // Simulation populates Transaction.digest, but may omit ExecutedTransaction.digest.
    // Require the inner digest and effects to match our SDK digest; reject a conflicting wrapper.
    ensure!(
        returned.digest.as_deref() == Some(expected_digest.as_str())
            && executed
                .digest
                .as_deref()
                .is_none_or(|digest| digest == expected_digest),
        "simulation digest mismatch"
    );
    let effects = executed
        .effects
        .as_ref()
        .context("missing simulation effects")?;
    ensure!(
        effects.transaction_digest.as_deref() == Some(expected_digest.as_str()),
        "simulation effects digest mismatch"
    );
    ensure!(
        effects.status.as_ref().and_then(|s| s.success) == Some(true),
        "simulation execution failed or unknown"
    );
    Ok(())
}
