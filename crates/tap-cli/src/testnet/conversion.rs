//! Explicit whole-coin conversion approval, durable recovery and keyless exact-artifact dispatch.
use crate::store::{ConversionHistory as History, ConversionRecord as Record, Store};
use anyhow::{Context, Result, ensure};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{BufRead, IsTerminal, Read},
    path::{Path, PathBuf},
};
use sui_crypto::ed25519::Ed25519PrivateKey;
use sui_sdk_types::{Address, Digest, Transaction};
use tap_core::SUI_TESTNET_CHAIN;
use tap_sui::{
    Prepared,
    conversion::{
        self, ENDPOINT, Inspection, SelectedCoin,
        settlement::{self, Observation},
    },
    settlement::Outcome,
};

const PROFILE: &str = "circle-sui-testnet/whole-coin-own-balance/host-ed25519/v1";

#[derive(Args)]
pub struct ConversionArgs {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Inspect one smallest eligible whole coin and simulate its unsigned same-owner conversion.
    Inspect {
        #[arg(long)]
        sender: Address,
    },
    /// Human-private: freeze, approve and sign one exact conversion. Never broadcasts.
    Prepare(PrepareArgs),
    /// Keyless: reconcile first, then explicitly approve dispatch of a saved live artifact.
    Submit(SubmitArgs),
    /// Keyless saved-digest lookup, including after the input coin is consumed.
    Reconcile(WalletArgs),
}
#[derive(Args)]
struct PrepareArgs {
    #[command(flatten)]
    key: super::KeyArgs,
    /// One conversion operation per dedicated private journal, never overwrite it.
    #[arg(long)]
    journal: PathBuf,
    #[arg(long)]
    out: PathBuf,
    /// Must be the fixed official testnet endpoint. No fallback.
    #[arg(long, default_value = ENDPOINT)]
    endpoint: String,
    #[arg(long)]
    recover_saved_candidate: bool,
    /// PUBLIC fixture key [9;32] ONLY. No RPC; never submittable.
    #[arg(long)]
    offline_fixture_epoch: Option<u64>,
}
#[derive(Args)]
struct WalletArgs {
    #[arg(long)]
    journal: PathBuf,
    #[arg(long)]
    expected_sender: Address,
    #[arg(long, default_value = ENDPOINT)]
    endpoint: String,
}
#[derive(Args)]
struct SubmitArgs {
    #[command(flatten)]
    wallet: WalletArgs,
    /// Explicitly replay only the identical artifact after a new NotFound lookup.
    #[arg(long)]
    replay_saved: bool,
}

fn json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}
fn parse<T: DeserializeOwned>(value: &str) -> Result<T> {
    ensure!(value.len() <= 1_048_576, "conversion record exceeds limit");
    Ok(serde_json::from_str(value)?)
}
fn tag<T: Serialize>(value: &T) -> Result<String> {
    Ok(hex::encode(tap_core::hash(&serde_json::to_vec(value)?)))
}
fn open(path: &Path, existing: bool) -> Result<Store> {
    super::private_parent(path)?;
    match std::fs::symlink_metadata(path) {
        Ok(_) => {
            super::private_file(path)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && !existing => {}
        Err(e) => return Err(e.into()),
    }
    Store::open(path)
}
fn fixture_sender() -> Address {
    Ed25519PrivateKey::new([9; 32])
        .public_key()
        .derive_address()
}
fn live_sender(sender: Address) -> Result<()> {
    for seed in [7, 9, 11, 12, 21, 22, 23] {
        ensure!(
            sender
                != Ed25519PrivateKey::new([seed; 32])
                    .public_key()
                    .derive_address(),
            "public fixture keys cannot enter live conversion flow"
        );
    }
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Terms {
    profile: String,
    endpoint: String,
    coin: SelectedCoin,
    epoch: u64,
    created_ms: u64,
    offline_fixture_epoch: Option<u64>,
    bcs_hex: String,
    digest: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    operation_id: String,
    terms: Terms,
}
impl Candidate {
    fn new(coin: SelectedCoin, tx: &Transaction, fixture: Option<u64>) -> Result<Self> {
        let terms = Terms {
            profile: PROFILE.into(),
            endpoint: ENDPOINT.into(),
            coin,
            epoch: conversion::epoch_of(tx)?,
            created_ms: crate::now()?,
            offline_fixture_epoch: fixture,
            bcs_hex: hex::encode(bcs::to_bytes(tx)?),
            digest: tx.digest().to_string(),
        };
        Ok(Self {
            operation_id: tag(&terms)?,
            terms,
        })
    }
    fn transaction(&self) -> Result<Transaction> {
        let t = &self.terms;
        ensure!(
            t.profile == PROFILE
                && t.endpoint == ENDPOINT
                && self.operation_id == tag(t)?
                && t.created_ms > 0,
            "candidate profile/identity mismatch"
        );
        ensure!(t.bcs_hex.len() <= 32_768, "candidate too large");
        let tx: Transaction = bcs::from_bytes(&hex::decode(&t.bcs_hex)?)?;
        conversion::verify_template(&t.coin, &tx)?;
        ensure!(
            hex::encode(bcs::to_bytes(&tx)?) == t.bcs_hex
                && tx.digest().to_string() == t.digest
                && conversion::epoch_of(&tx)? == t.epoch,
            "candidate bytes/digest/epoch mismatch"
        );
        match t.offline_fixture_epoch {
            Some(epoch) => {
                ensure!(
                    t.coin == fixture_coin() && epoch == t.epoch,
                    "invalid offline fixture candidate"
                );
            }
            None => live_sender(t.coin.owner)?,
        }
        Ok(tx)
    }
}
fn fixture_coin() -> SelectedCoin {
    SelectedCoin {
        id: Address::new([0x44; 32]),
        version: 1,
        digest: Digest::new([0x55; 32]),
        owner: fixture_sender(),
        value: 20_000_000,
        storage_rebate: 0,
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Qualification {
    genesis: Digest,
    epoch: u64,
    digest: String,
    asset: String,
    protocol_version: u64,
    minimum: u64,
    qualified: bool,
    chain_submission: bool,
    checks_enabled: bool,
    gas_selection: bool,
}
impl Qualification {
    fn from_report(report: &Inspection) -> Result<Self> {
        ensure!(
            report.qualified && !report.chain_submission && report.blockers.is_empty(),
            "conversion not qualified: {:?}",
            report.blockers
        );
        Ok(Self {
            genesis: report.node.chain_id.parse()?,
            epoch: report.node.epoch,
            digest: report.transaction_digest.clone(),
            asset: report.asset.into(),
            protocol_version: report.protocol_version,
            minimum: report.protocol_minimum,
            qualified: true,
            chain_submission: false,
            checks_enabled: true,
            gas_selection: false,
        })
    }
    fn validate(&self, c: &Candidate) -> Result<()> {
        ensure!(
            c.terms.offline_fixture_epoch.is_none()
                && self.genesis == SUI_TESTNET_CHAIN.parse()?
                && self.epoch == c.terms.epoch
                && self.digest == c.terms.digest
                && self.asset == super::POLICY.coin_type()
                && self.protocol_version > 0
                && c.terms.coin.value >= self.minimum.max(10_000)
                && self.qualified
                && !self.chain_submission
                && self.checks_enabled
                && !self.gas_selection,
            "qualification summary contradicts candidate/policy"
        );
        Ok(())
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Action {
    Prepare,
    Recover,
    Submit,
    Replay,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Approval {
    operation_id: String,
    digest: String,
    artifact_tag: Option<String>,
    action: Action,
    approved_ms: u64,
    qualification: Option<Qualification>,
}
impl Approval {
    fn validate(&self, c: &Candidate, action: Action, artifact: Option<&Artifact>) -> Result<()> {
        ensure!(
            self.operation_id == c.operation_id
                && self.digest == c.terms.digest
                && self.action == action
                && self.approved_ms >= c.terms.created_ms
                && self.artifact_tag == artifact.map(tag).transpose()?,
            "approval terms/action/artifact mismatch"
        );
        match (&self.qualification, c.terms.offline_fixture_epoch) {
            (Some(q), None) => q.validate(c)?,
            (None, Some(_)) if matches!(action, Action::Prepare | Action::Recover) => {}
            _ => anyhow::bail!("approval qualification/mode mismatch"),
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    operation_id: String,
    prepared: Prepared,
}
impl Artifact {
    fn validate(&self, c: &Candidate) -> Result<()> {
        ensure!(
            self.operation_id == c.operation_id
                && self.prepared.transaction()? == c.transaction()?
                && self.prepared.bcs_hex == c.terms.bcs_hex,
            "artifact differs from exact reserved candidate"
        );
        Ok(())
    }
}

fn approvals(store: &Store, c: &Candidate) -> Result<bool> {
    let reservation = store.conversion_record(Record::Reservation)?;
    if let Some(value) = &reservation {
        parse::<Approval>(value)?.validate(c, Action::Prepare, None)?;
    }
    store.conversion_history(History::Recovery, |_, value| {
        ensure!(reservation.is_some(), "recovery lacks reservation");
        parse::<Approval>(value)?.validate(c, Action::Recover, None)
    })?;
    Ok(reservation.is_some())
}
fn display(c: &Candidate) -> Result<()> {
    let tx = c.transaction()?;
    let coin = &c.terms.coin;
    eprintln!(
        "\nTESTNET WHOLE-COIN CONVERSION / NOT A MERCHANT PAYMENT / {PROFILE}\nWhole value: {}.{:06} USDC ({} units)\nCoin: {}\nVersion: {}\nObject digest: {}\nSender AND recipient: {}\nAsset: {}\nGenesis: {}\nEndpoint: {}\nExpiration/nonce: {:?}\nTransaction digest: {}\nOperation ID: {}\nGas: no objects, price=0, budget=0. Storage rebate FOREGONE (object metadata {} MIST; not a rebate promise).\nOffline PUBLIC fixture: {} / software host approval only",
        coin.value / 1_000_000,
        coin.value % 1_000_000,
        coin.value,
        coin.id,
        coin.version,
        coin.digest,
        coin.owner,
        super::POLICY.coin_type(),
        SUI_TESTNET_CHAIN,
        ENDPOINT,
        tx.expiration,
        c.terms.digest,
        c.operation_id,
        coin.storage_rebate,
        c.terms.offline_fixture_epoch.is_some()
    );
    Ok(())
}
fn approve(c: &Candidate, phrase: &str) -> Result<()> {
    ensure!(
        std::io::stdin().is_terminal(),
        "conversion approval requires terminal input"
    );
    display(c)?;
    eprintln!("Type {phrase}:");
    let mut line = String::new();
    std::io::stdin().lock().take(128).read_line(&mut line)?;
    ensure!(line.trim() == phrase, "not approved");
    Ok(())
}
async fn qualification(c: &Candidate) -> Result<Option<Qualification>> {
    if c.terms.offline_fixture_epoch.is_some() {
        return Ok(None);
    }
    let report = conversion::qualify(&c.terms.coin, &c.transaction()?).await?;
    let q = Qualification::from_report(&report)?;
    q.validate(c)?;
    Ok(Some(q))
}

async fn prepare(args: PrepareArgs) -> Result<()> {
    ensure!(
        args.endpoint == ENDPOINT,
        "fixed official conversion endpoint required"
    );
    match args.offline_fixture_epoch {
        Some(_) => ensure!(
            args.key.expected_sender == fixture_sender(),
            "offline mode requires public fixture [9;32]"
        ),
        None => live_sender(args.key.expected_sender)?,
    }
    super::private_parent(&args.out)?;
    let store = open(&args.journal, false)?;
    let c = match store.conversion_record(Record::Candidate)? {
        Some(value) => parse::<Candidate>(&value)?,
        None => {
            ensure!(
                !args.recover_saved_candidate,
                "no saved candidate to recover"
            );
            let (coin, tx, q) = if let Some(epoch) = args.offline_fixture_epoch {
                let coin = fixture_coin();
                let tx = conversion::build(&coin, epoch)?;
                (coin, tx, None)
            } else {
                let report = conversion::inspect(args.key.expected_sender).await?;
                let q = Qualification::from_report(&report)?;
                let tx = bcs::from_bytes(&hex::decode(&report.transaction_bcs_hex)?)?;
                (report.selected_coin, tx, Some(q))
            };
            let c = Candidate::new(coin, &tx, args.offline_fixture_epoch)?;
            c.transaction()?;
            if let Some(q) = &q {
                q.validate(&c)?;
            }
            store.save_conversion_record(Record::Candidate, &json(&c)?)?;
            c
        }
    };
    let tx = c.transaction()?;
    ensure!(
        c.terms.coin.owner == args.key.expected_sender
            && c.terms.offline_fixture_epoch == args.offline_fixture_epoch,
        "saved sender/mode mismatch"
    );
    let reserved = approvals(&store, &c)?;
    if let Some(value) = store.conversion_record(Record::Artifact)? {
        ensure!(reserved, "artifact lacks reservation");
        let artifact: Artifact = parse(&value)?;
        artifact.validate(&c)?;
        super::export(&args.out, &serde_json::to_vec_pretty(&artifact)?)?;
        println!(
            "SAVED_CONVERSION_ARTIFACT digest={} no_new_signing=true chain_submission=false converted=false",
            c.terms.digest
        );
        return Ok(());
    }
    ensure!(
        reserved == args.recover_saved_candidate,
        "UNRESOLVED_ISSUANCE: reserved candidate needs --recover-saved-candidate; unreserved candidate needs normal preparation"
    );
    let phrase = if reserved {
        "RECOVER TESTNET CONVERSION"
    } else {
        "PREPARE TESTNET CONVERSION"
    };
    approve(&c, phrase)?;
    // Recheck saved bytes after human input. Recovery never refreshes a reference/epoch.
    let q = qualification(&c).await?;
    let action = if reserved {
        Action::Recover
    } else {
        Action::Prepare
    };
    let approval = Approval {
        operation_id: c.operation_id.clone(),
        digest: c.terms.digest.clone(),
        artifact_tag: None,
        action: action.clone(),
        approved_ms: crate::now()?,
        qualification: q,
    };
    approval.validate(&c, action, None)?;
    if reserved {
        store.append_conversion(History::Recovery, &json(&approval)?)?;
    } else {
        store.save_conversion_record(Record::Reservation, &json(&approval)?)?;
    }
    // Open/check metadata before PR_SET_DUMPABLE changes /proc/self ownership. No byte
    // reads until dump protection succeeds; issuance is already durably reserved.
    let file = super::private_file(&args.key.keystore)?;
    super::init::disable_dumpability()?;
    let key = super::key_from_file(file, args.key.expected_sender, 32)?;
    conversion::verify_template(&c.terms.coin, &tx)?;
    let artifact = Artifact {
        operation_id: c.operation_id.clone(),
        prepared: Prepared::sign(&tx, &key)?,
    };
    artifact.validate(&c)?;
    store.save_conversion_record(Record::Artifact, &json(&artifact)?)?;
    super::export(&args.out, &serde_json::to_vec_pretty(&artifact)?)?;
    println!(
        "CONVERSION_PREPARED digest={} chain_submission=false converted=false offline_fixture={}",
        c.terms.digest,
        c.terms.offline_fixture_epoch.is_some()
    );
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedObservation {
    operation_id: String,
    artifact_tag: String,
    observed_ms: u64,
    attempt: Option<i64>,
    observation: Observation,
}
struct Wallet {
    store: Store,
    candidate: Candidate,
    artifact: Artifact,
    dispatched: bool,
    terminal: bool,
}
impl Wallet {
    fn load(args: &WalletArgs) -> Result<Self> {
        ensure!(
            args.endpoint == ENDPOINT,
            "fixed official conversion endpoint required"
        );
        let store = open(&args.journal, true)?;
        let candidate: Candidate = parse(
            &store
                .conversion_record(Record::Candidate)?
                .context("no saved candidate")?,
        )?;
        candidate.transaction()?;
        ensure!(
            candidate.terms.offline_fixture_epoch.is_none(),
            "offline fixture is NEVER submittable or remotely reconcilable"
        );
        ensure!(
            candidate.terms.coin.owner == args.expected_sender,
            "expected sender mismatch"
        );
        ensure!(
            approvals(&store, &candidate)?,
            "artifact lacks validated reservation"
        );
        let artifact: Artifact = parse(
            &store
                .conversion_record(Record::Artifact)?
                .context("no artifact; recover preparation separately")?,
        )?;
        artifact.validate(&candidate)?;
        let intent = store.conversion_record(Record::Dispatch)?;
        if let Some(value) = &intent {
            parse::<Approval>(value)?.validate(&candidate, Action::Submit, Some(&artifact))?;
        }
        let mut attempt_ids = std::collections::BTreeSet::new();
        store.conversion_history(History::Attempts, |id, value| {
            let a: Approval = parse(value)?;
            ensure!(intent.is_some(), "attempt lacks dispatch intent");
            let action = if attempt_ids.is_empty() {
                Action::Submit
            } else {
                Action::Replay
            };
            a.validate(&candidate, action, Some(&artifact))?;
            if attempt_ids.is_empty() {
                ensure!(
                    Some(value) == intent.as_deref(),
                    "first attempt/intent mismatch"
                );
            }
            attempt_ids.insert(id);
            Ok(())
        })?;
        ensure!(
            intent.is_none() || !attempt_ids.is_empty(),
            "dispatch lacks attempt"
        );
        let mut terminal = false;
        store.conversion_history(History::Observations, |_, value| {
            let o: SavedObservation = parse(value)?;
            ensure!(
                o.operation_id == candidate.operation_id
                    && o.artifact_tag == tag(&artifact)?
                    && o.observed_ms >= candidate.terms.created_ms,
                "observation binding mismatch"
            );
            if let Some(id) = o.attempt {
                ensure!(
                    attempt_ids.contains(&id),
                    "observation lacks matching attempt"
                );
            }
            o.observation
                .validate(&candidate.terms.coin, &artifact.prepared)?;
            terminal |= checkpoint_seen(&o.observation);
            Ok(())
        })?;
        Ok(Self {
            store,
            candidate,
            artifact,
            dispatched: intent.is_some(),
            terminal,
        })
    }
    fn save(&mut self, observation: Observation, attempt: Option<i64>) -> Result<()> {
        observation.validate(&self.candidate.terms.coin, &self.artifact.prepared)?;
        let terminal = checkpoint_seen(&observation);
        let saved = SavedObservation {
            operation_id: self.candidate.operation_id.clone(),
            artifact_tag: tag(&self.artifact)?,
            observed_ms: crate::now()?,
            attempt,
            observation,
        };
        self.store
            .append_conversion(History::Observations, &json(&saved)?)?;
        self.terminal |= terminal;
        report(&saved.observation, self.dispatched)?;
        Ok(())
    }
}
fn checkpoint_seen(o: &Observation) -> bool {
    matches!(
        o.outcome,
        Outcome::VerifiedSuccess { .. } | Outcome::VerifiedFailure { .. }
    ) || o.execution.as_ref().and_then(|e| e.checkpoint).is_some()
}
fn report(o: &Observation, dispatched: bool) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "outcome":o.outcome, "error":o.error, "endpoint":o.endpoint,
            "dispatch_intent_recorded":dispatched, "converted":matches!(o.outcome, Outcome::VerifiedSuccess { .. }),
            "notice":"Checkpoint observations trust this chosen node, not a light-client proof. Conversion is NOT merchant payment. Address-balance indexing may lag; never convert again on indexing lag."
        }))?
    );
    Ok(())
}
async fn reconcile(args: WalletArgs) -> Result<()> {
    let mut w = Wallet::load(&args)?;
    let observation = settlement::query(&w.candidate.terms.coin, &w.artifact.prepared).await;
    let known = matches!(
        observation.outcome,
        Outcome::VerifiedSuccess { .. } | Outcome::VerifiedFailure { .. }
    );
    let success = matches!(observation.outcome, Outcome::VerifiedSuccess { .. });
    w.save(observation, None)?;
    if success {
        indexed_balance(&w.candidate).await;
    }
    ensure!(
        known,
        "outcome not checkpoint-verified; retain saved artifact and reconcile; no new conversion"
    );
    Ok(())
}
async fn submit(args: SubmitArgs) -> Result<()> {
    let mut w = Wallet::load(&args.wallet)?;
    // Saved digest FIRST: spent input coins and old epochs do not imply failure.
    let observation = settlement::query(&w.candidate.terms.coin, &w.artifact.prepared).await;
    let not_found = observation.outcome == Outcome::NotFound;
    let success = matches!(observation.outcome, Outcome::VerifiedSuccess { .. });
    w.save(observation, None)?;
    if success {
        indexed_balance(&w.candidate).await;
        return Ok(());
    }
    ensure!(
        !w.terminal,
        "checkpoint result already seen; resend is forbidden even if later RPC degrades"
    );
    ensure!(
        not_found,
        "lookup uncertain; reconcile saved digest, never build another conversion"
    );
    ensure!(
        w.dispatched == args.replay_saved,
        "prior dispatch requires explicit --replay-saved; first dispatch must not request replay"
    );
    let action = if w.dispatched {
        Action::Replay
    } else {
        Action::Submit
    };
    let phrase = if w.dispatched {
        "REPLAY SAVED TESTNET CONVERSION"
    } else {
        "CONVERT TESTNET WHOLE COIN"
    };
    approve(&w.candidate, phrase)?;
    let q = qualification(&w.candidate)
        .await?
        .context("live qualification required")?;
    let a = Approval {
        operation_id: w.candidate.operation_id.clone(),
        digest: w.candidate.terms.digest.clone(),
        artifact_tag: Some(tag(&w.artifact)?),
        action: action.clone(),
        approved_ms: crate::now()?,
        qualification: Some(q),
    };
    a.validate(&w.candidate, action, Some(&w.artifact))?;
    let value = json(&a)?;
    let attempt = w
        .store
        .begin_conversion_dispatch((!w.dispatched).then_some(value.as_str()), &value)?;
    w.dispatched = true;
    let observation =
        settlement::execute_saved(&w.candidate.terms.coin, &w.artifact.prepared).await;
    let success = matches!(observation.outcome, Outcome::VerifiedSuccess { .. });
    w.save(observation, Some(attempt))?;
    if success {
        indexed_balance(&w.candidate).await;
    }
    ensure!(
        success,
        "conversion not verified successful; possibly sent; retain identical artifact and reconcile"
    );
    Ok(())
}

async fn indexed_balance(c: &Candidate) {
    match settlement::address_balance(&c.terms.coin).await {
        Ok(balance) => println!(
            "CONVERSION_CHECKPOINT_VERIFIED address_balance={} indexing_status={} merchant_payment_qualified=false never_reconvert_for_indexing_lag=true",
            balance,
            if balance >= 10_000 {
                "AVAILABLE_REQUALIFY_PAYMENT"
            } else {
                "PENDING_OR_SPENT"
            }
        ),
        Err(_) => println!(
            "CONVERSION_CHECKPOINT_VERIFIED indexing_status=PENDING_OR_UNAVAILABLE merchant_payment_qualified=false never_reconvert_for_indexing_lag=true"
        ),
    }
}

pub async fn run(args: ConversionArgs) -> Result<()> {
    match args.command {
        Command::Inspect { sender } => {
            let report = conversion::inspect(sender).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            ensure!(
                report.qualified,
                "conversion inspection not qualified; nothing signed or submitted"
            );
            Ok(())
        }
        Command::Prepare(args) => prepare(args).await,
        Command::Submit(args) => submit(args).await,
        Command::Reconcile(args) => reconcile(args).await,
    }
}
