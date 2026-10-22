//! Read-only qualification of the existing template, never asset substitution or signing.
use super::{build_direct, client, probe, verify_template};
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
use sui_sdk_types::{Address, Digest, Transaction};
use tap_core::{Invoice, USDC};

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
        coin_type: USDC.into(),
        amount,
        created_ms: 0,
        expires_ms: 0,
        merchant: "READ-ONLY QUALIFICATION; NOT A PAYMENT REQUEST".into(),
    };
    let tx = build_direct(&invoice, sender, node.epoch, 0)?;
    verify_template(&invoice, &tx)?;
    let mut report = Qualification {
        endpoint: endpoint.into(),
        node,
        asset: USDC,
        sender,
        receiver,
        amount,
        transaction_digest: tx.digest().to_string(),
        transaction_bcs_hex: hex::encode(bcs::to_bytes(&tx)?),
        protocol: None,
        coin_info: None,
        balance: None,
        simulation: None,
        blockers: vec![],
        qualified: false,
        chain_submission: false,
    };
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
                            row[0].as_str() == Some(USDC)
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
    coin_req.coin_type = Some(USDC.into());
    match tokio::time::timeout(
        Duration::from_secs(15),
        c.state_client().get_coin_info(coin_req),
    )
    .await
    {
        Ok(Ok(r)) => {
            let coin = r.into_inner();
            if coin.coin_type.as_deref() != Some(USDC)
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
    balance_req.coin_type = Some(USDC.into());
    match tokio::time::timeout(
        Duration::from_secs(15),
        c.state_client().get_balance(balance_req),
    )
    .await
    {
        Ok(Ok(r)) => {
            report.balance = r.into_inner().balance;
            if !report.balance.as_ref().is_some_and(|b| {
                b.coin_type.as_deref() == Some(USDC)
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
    let mut req = SimulateTransactionRequest::default();
    req.transaction = Some(tx.clone().into());
    req.read_mask = Some(FieldMask::from_paths(["*"]));
    req.checks = Some(TransactionChecks::Enabled as i32);
    req.do_gas_selection = Some(false);
    match tokio::time::timeout(
        Duration::from_secs(15),
        c.execution_client().simulate_transaction(req),
    )
    .await
    {
        Ok(Ok(r)) => {
            let simulation = r.into_inner();
            if let Err(error) = verify_simulation(&tx, &simulation) {
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
    if after.epoch != report.node.epoch {
        report
            .blockers
            .push("epoch changed during qualification; repeat read-only qualification".into());
    }
    report.qualified = report.blockers.is_empty();
    Ok(report)
}

fn verify_simulation(tx: &Transaction, simulation: &SimulateTransactionResponse) -> Result<()> {
    let executed = simulation
        .transaction
        .as_ref()
        .context("missing simulated transaction")?;
    let observed: Transaction = executed
        .transaction
        .as_ref()
        .context("missing simulated bytes")?
        .try_into()?;
    ensure!(observed == *tx, "simulation changed transaction");
    ensure!(
        executed.digest.as_deref() == Some(tx.digest().to_string().as_str()),
        "simulation digest mismatch"
    );
    let effects = executed
        .effects
        .as_ref()
        .context("missing simulation effects")?;
    ensure!(
        effects.transaction_digest == executed.digest,
        "simulation effects digest mismatch"
    );
    ensure!(
        effects.status.as_ref().and_then(|s| s.success) == Some(true),
        "simulation execution failed or unknown"
    );
    Ok(())
}
