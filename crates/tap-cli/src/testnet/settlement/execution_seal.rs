//! Keyless operator-lab export. A private seal is a binding record, not broadcast authority.
use super::{Registration, Wallet, WalletArgs, read_record};
use anyhow::{Context, Result, ensure};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sui_sdk_types::Address;

const UPSTREAM: &str = "https://fullnode.testnet.sui.io:443";

#[derive(Args)]
pub struct SealArgs {
    #[command(flatten)]
    wallet: WalletArgs,
    #[arg(long)]
    registration: PathBuf,
    /// Independently expected NEW controlled receiver; this does not certify global freshness.
    #[arg(long)]
    expected_receiver: Address,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Seal {
    profile: String,
    upstream: String,
    endpoint: String,
    genesis: String,
    sender: Address,
    receiver: Address,
    merchant_pin: String,
    invoice_hash: String,
    digest: String,
    artifact_tag: String,
    asset: String,
    amount_base_units: u64,
    execution_protobuf_hex: String,
    simulation_protobuf_hex: String,
    authority: String,
    wallet_journal: PathBuf,
    invoice_inbox: PathBuf,
    registration: PathBuf,
}

pub(crate) fn export(args: SealArgs) -> Result<()> {
    let wallet = Wallet::read_only(&args.wallet)?;
    ensure!(
        wallet.intent.is_none() && wallet.known.is_none(),
        "SEAL_REQUIRES_PRE_DISPATCH_ARTIFACT: never seal a settled/possibly-sent operation"
    );
    let seal = make_seal(&args, &wallet)?;
    let bytes = serde_json::to_vec_pretty(&seal)?;
    ensure!(bytes.len() <= 65_536, "seal exceeds limit");
    super::super::export(&args.out, &bytes)?;
    println!("EXECUTION_SEAL_EXPORTED_NOT_ARMED_NOT_SUBMITTED");
    Ok(())
}

fn make_seal(args: &SealArgs, wallet: &Wallet) -> Result<Seal> {
    ensure!(
        wallet.invoice.amount == 10_000
            && Address::new(wallet.invoice.receiver) == args.expected_receiver,
        "seal requires exact 0.01 USDC and independently expected receiver"
    );
    ensure!(
        wallet.merchant.endpoint == UPSTREAM,
        "fixed lab upstream required"
    );
    let port: u16 = wallet
        .endpoint
        .strip_prefix("http://127.0.0.1:")
        .context("seal requires explicit loopback lab endpoint")?
        .parse()?;
    ensure!(
        port >= 1024 && wallet.endpoint == format!("http://127.0.0.1:{port}"),
        "seal endpoint must be a literal loopback port without path/userinfo"
    );
    let registration: Registration = read_record(&args.registration)?;
    registration.validate(&wallet.merchant, &wallet.envelope, &wallet.invoice)?;
    if let Some(intent) = &wallet.intent {
        ensure!(
            super::tag(&intent.registration)? == super::tag(&registration)?,
            "registration changed since approval"
        );
    }
    let payloads = tap_sui::settlement::execution_lab_payloads(&wallet.invoice, &wallet.prepared)?;
    let seal = Seal {
        profile: "circle-sui-testnet/execution-response-loss-seal/v1".into(),
        upstream: UPSTREAM.into(),
        endpoint: wallet.endpoint.clone(),
        genesis: wallet.identity.chain.to_string(),
        sender: wallet.identity.sender,
        receiver: args.expected_receiver,
        merchant_pin: wallet.identity.merchant_pin.clone(),
        invoice_hash: wallet.identity.invoice_hash.clone(),
        digest: wallet.identity.digest.clone(),
        artifact_tag: wallet.identity.artifact_tag.clone(),
        asset: wallet.invoice.coin_type.clone(),
        amount_base_units: wallet.invoice.amount,
        execution_protobuf_hex: hex::encode(payloads.execution),
        simulation_protobuf_hex: hex::encode(payloads.simulation),
        authority: "BINDING_ONLY_NOT_SUBMISSION_APPROVAL; native TESTNET SUBMIT and separate human proxy arming required".into(),
        wallet_journal: args.wallet.journal.clone(),
        invoice_inbox: args.wallet.invoice_inbox.clone(),
        registration: args.registration.clone(),
    };
    Ok(seal)
}

#[derive(Args)]
pub struct CheckSealArgs {
    #[arg(long)]
    seal: PathBuf,
}

pub(crate) fn check_dispatch(args: CheckSealArgs) -> Result<()> {
    super::super::private_parent(&args.seal)?;
    super::super::private_file(&args.seal)?;
    let seal: Seal = serde_json::from_slice(&crate::read_bounded(&args.seal, 65_536)?)?;
    let wallet_args = WalletArgs {
        invoice_inbox: seal.invoice_inbox.clone(),
        merchant_key: seal.merchant_pin.clone(),
        expected_sender: seal.sender,
        journal: seal.wallet_journal.clone(),
        endpoint: seal.endpoint.clone(),
        merchant_endpoint: seal.upstream.clone(),
    };
    // Submit retains its cooperative owner lock. This separate guard uses only a read snapshot.
    let wallet = Wallet::read_only(&wallet_args)?;
    ensure!(
        wallet.intent.is_some() && wallet.known.is_none(),
        "first native dispatch barrier required"
    );
    let mut attempts = 0;
    wallet.store.live_history(
        &wallet.identity.invoice_hash,
        super::History::Attempts,
        |_, _| {
            attempts += 1;
            Ok(())
        },
    )?;
    ensure!(
        attempts == 1,
        "exactly one approved first dispatch attempt required"
    );
    wallet.store.live_history(
        &wallet.identity.invoice_hash,
        super::History::Observations,
        |_, value| {
            let observation: super::WalletObservation = super::parse(value)?;
            ensure!(
                matches!(observation.source, super::Source::Query),
                "prior Dispatch/confirmation cannot be rearmed"
            );
            Ok(())
        },
    )?;
    let expected = make_seal(
        &SealArgs {
            wallet: wallet_args,
            registration: seal.registration.clone(),
            expected_receiver: seal.receiver,
            out: args.seal,
        },
        &wallet,
    )?;
    ensure!(
        seal == expected,
        "seal differs from approved native records or canonical request"
    );
    println!("EXECUTION_SEAL_FIRST_DISPATCH_VALID_NO_RPC");
    Ok(())
}

#[derive(Args)]
pub struct SimulationArgs {
    #[arg(long)]
    invoice_inbox: PathBuf,
    #[arg(long)]
    merchant_key: String,
    #[arg(long)]
    expected_sender: Address,
    /// Official protobuf bytes only, excluding the standard gRPC frame header. No RPC occurs.
    #[arg(long)]
    request_hex: String,
}

pub(crate) fn check_simulation(args: SimulationArgs) -> Result<()> {
    ensure!(
        args.request_hex.len() <= 65_536,
        "simulation request exceeds limit"
    );
    let (_, invoice, _) = super::invoice(&args.invoice_inbox, &args.merchant_key)?;
    tap_sui::qualification::validate_lab_simulation(
        &invoice,
        args.expected_sender,
        &hex::decode(&args.request_hex)?,
    )?;
    println!("LAB_CHECKED_SIMULATION_VALID_NOT_SIGNED_NOT_SUBMITTED");
    Ok(())
}
