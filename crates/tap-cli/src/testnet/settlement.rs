//! Controlled keyless dispatch and separately trusted merchant reconciliation, using private files.
use super::{Candidate, POLICY, Prepared, QualificationSummary, SignedInvoice};
use crate::store::{History, Store};
use anyhow::{Context, Result, ensure};
use clap::Args;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{BufRead, IsTerminal, Read},
    path::{Path, PathBuf},
};
use sui_sdk_types::{Address, Digest};
use tap_core::{Invoice, MAX_ENVELOPE, SUI_TESTNET_CHAIN};
use tap_sui::settlement::{Observation, Outcome};

const PROFILE: &str = "circle-sui-testnet/controlled-dispatch-host-approval/v1";
mod execution_seal;
pub(super) use execution_seal::check_dispatch as check_execution_seal;
pub(super) use execution_seal::check_simulation as check_lab_simulation;
pub(super) use execution_seal::export as export_execution_seal;
pub use execution_seal::{CheckSealArgs, SealArgs, SimulationArgs};

#[derive(Args)]
pub struct WalletArgs {
    #[arg(long)]
    pub invoice_inbox: PathBuf,
    /// Independently pinned invoice key, not a display name or receipt-provided identity.
    #[arg(long)]
    pub merchant_key: String,
    #[arg(long)]
    pub expected_sender: Address,
    #[arg(long)]
    pub journal: PathBuf,
    /// Must match the approved candidate endpoint exactly. No fallback.
    #[arg(long)]
    pub endpoint: String,
    /// Separately chosen merchant observer, pinned independently of artifact files.
    #[arg(long)]
    pub merchant_endpoint: String,
}
#[derive(Args)]
pub struct SubmitArgs {
    #[command(flatten)]
    wallet: WalletArgs,
    #[arg(long)]
    registration: PathBuf,
    /// After querying the saved digest, explicitly permit only an identical-artifact resubmission.
    #[arg(long)]
    replay_saved: bool,
    /// TESTNET FAULT TRIAL: exit 86 after a verified first execution response, before observation persistence.
    /// This loses an application-received result, not an RPC packet. Never arms on replay/cached success.
    #[arg(long, conflicts_with = "replay_saved")]
    fault_exit_after_execution_before_observation: bool,
    /// Observe the real Execute transport future for a reviewed operator lab; never arms or approves payment.
    #[arg(long, conflicts_with_all = ["replay_saved", "fault_exit_after_execution_before_observation"])]
    trace_execution_response: bool,
}
#[derive(Args)]
pub struct RegisterArgs {
    #[arg(long)]
    invoice: PathBuf,
    #[arg(long)]
    merchant_key: String,
    #[arg(long)]
    merchant_endpoint: String,
    #[arg(long)]
    ledger: PathBuf,
    #[arg(long)]
    out: PathBuf,
}
#[derive(Args)]
pub struct AcceptArgs {
    #[command(flatten)]
    merchant: RegisterArgs,
    #[arg(long)]
    prepared: PathBuf,
    /// TESTNET FAULT TRIAL: exit 86 after a NEW verified credit commits, before receipt export.
    /// Requires an absent output and an invoice without existing credit.
    #[arg(long)]
    fault_exit_after_credit_before_receipt: bool,
}
#[derive(Args)]
pub struct ConfirmArgs {
    #[command(flatten)]
    wallet: WalletArgs,
    /// Untrusted receipt locator/metadata; verified independently before local association.
    #[arg(long)]
    receipt: PathBuf,
}

fn parse<T: DeserializeOwned>(value: &str) -> Result<T> {
    ensure!(value.len() <= 1_048_576, "live record exceeds limit");
    Ok(serde_json::from_str(value)?)
}
fn json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}
fn tag<T: Serialize>(value: &T) -> Result<String> {
    Ok(hex::encode(tap_core::hash(&serde_json::to_vec(value)?)))
}
fn read_record<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&crate::read_bounded(
        path, 1_048_576,
    )?)?)
}
fn open_store(path: &Path, existing: bool) -> Result<Store> {
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
fn invoice(path: &Path, pin: &str) -> Result<(Vec<u8>, Invoice, String)> {
    let envelope = crate::read_bounded(path, MAX_ENVELOPE)?;
    let signed: SignedInvoice = serde_json::from_slice(&envelope)?;
    let invoice = signed.verify(&crate::pin(pin)?)?;
    POLICY.validate(&invoice)?;
    Ok((envelope, invoice, hex::encode(signed.invoice_hash()?)))
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    profile: String,
    chain: Digest,
    merchant_pin: String,
    invoice_hash: String,
    digest: String,
    artifact_tag: String,
    sender: Address,
}
impl Identity {
    fn new(
        invoice: &Invoice,
        envelope: &[u8],
        pin: &[u8; 32],
        prepared: &Prepared,
    ) -> Result<Self> {
        let signed: SignedInvoice = serde_json::from_slice(envelope)?;
        ensure!(signed.verify(pin)? == *invoice, "invoice trust mismatch");
        let tx = prepared.transaction()?;
        tap_sui::verify_template_with_policy(invoice, &tx, POLICY)?;
        ensure!(
            tx.sender != Address::new(invoice.receiver),
            "self-transfer cannot settle a merchant invoice"
        );
        Ok(Self {
            profile: PROFILE.into(),
            chain: Digest::new(invoice.chain),
            merchant_pin: hex::encode(pin),
            invoice_hash: hex::encode(signed.invoice_hash()?),
            digest: prepared.digest.clone(),
            artifact_tag: tag(prepared)?,
            sender: tx.sender,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MerchantConfig {
    profile: String,
    chain: Digest,
    merchant_pin: String,
    endpoint: String,
}
impl MerchantConfig {
    fn new(pin: &str, endpoint: &str) -> Result<Self> {
        tap_sui::validate_endpoint(endpoint)?;
        Ok(Self {
            profile: PROFILE.into(),
            chain: SUI_TESTNET_CHAIN.parse()?,
            merchant_pin: hex::encode(crate::pin(pin)?),
            endpoint: endpoint.into(),
        })
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registration {
    config: MerchantConfig,
    invoice_hash: String,
    envelope_hex: String,
    receiver: Address,
    registered_ms: u64,
}
impl Registration {
    fn validate(&self, config: &MerchantConfig, envelope: &[u8], invoice: &Invoice) -> Result<()> {
        let signed: SignedInvoice = serde_json::from_slice(envelope)?;
        ensure!(
            self.config == *config
                && self.invoice_hash == hex::encode(signed.invoice_hash()?)
                && self.envelope_hex == hex::encode(envelope)
                && self.receiver == Address::new(invoice.receiver)
                && self.registered_ms >= invoice.created_ms
                && self.registered_ms < invoice.expires_ms,
            "merchant registration trust/terms mismatch"
        );
        ensure!(
            signed.verify(&crate::pin(&config.merchant_pin)?)? == *invoice,
            "registration merchant pin mismatch"
        );
        POLICY.validate(invoice)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
enum DispatchAction {
    SubmitTestnet,
    ReplaySavedTestnet,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DispatchApproval {
    identity: Identity,
    endpoint: String,
    registration: Registration,
    approved_ms: u64,
    action: DispatchAction,
    qualification: QualificationSummary,
}
impl DispatchApproval {
    fn validate(&self, wallet: &Wallet) -> Result<()> {
        ensure!(
            self.identity == wallet.identity
                && self.endpoint == wallet.endpoint
                && self.approved_ms >= wallet.invoice.created_ms,
            "dispatch approval binding mismatch"
        );
        self.registration
            .validate(&wallet.merchant, &wallet.envelope, &wallet.invoice)?;
        if self.action == DispatchAction::SubmitTestnet {
            ensure!(
                self.approved_ms < wallet.invoice.expires_ms,
                "first dispatch approval was expired"
            );
        }
        self.qualification.validate(
            &wallet.candidate,
            &wallet.invoice,
            &wallet.prepared.transaction()?,
        )
    }
}
#[derive(Serialize, Deserialize)]
enum Source {
    Query,
    Dispatch { attempt: i64 },
    MerchantConfirmation,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WalletObservation {
    identity: Identity,
    observed_ms: u64,
    source: Source,
    observation: Observation,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MerchantObservation {
    identity: Identity,
    observed_ms: u64,
    observation: Observation,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    identity: Identity,
    registration: Registration,
    observed_ms: u64,
    observed_late: bool,
    observation: Observation,
}
impl Receipt {
    fn validate(
        &self,
        identity: &Identity,
        config: &MerchantConfig,
        envelope: &[u8],
        invoice: &Invoice,
        prepared: &Prepared,
    ) -> Result<()> {
        ensure!(
            self.identity == *identity
                && self.observation.endpoint == config.endpoint
                && self.observed_ms >= self.registration.registered_ms
                && self.observed_late == (self.observed_ms >= invoice.expires_ms),
            "receipt association mismatch"
        );
        self.registration.validate(config, envelope, invoice)?;
        self.observation.validate(invoice, prepared)?;
        ensure!(
            matches!(self.observation.outcome, Outcome::VerifiedSuccess { .. }),
            "receipt lacks independently verified checkpoint success"
        );
        Ok(())
    }
}

fn terminal(known: &mut Option<Outcome>, outcome: &Outcome) -> Result<()> {
    if matches!(
        outcome,
        Outcome::VerifiedSuccess { .. } | Outcome::VerifiedFailure { .. }
    ) {
        if let Some(previous) = known {
            ensure!(
                *previous == *outcome,
                "contradictory verified observations; reconciliation required"
            );
        }
        *known = Some(outcome.clone());
    }
    Ok(())
}

struct Wallet {
    store: Store,
    envelope: Vec<u8>,
    invoice: Invoice,
    candidate: Candidate,
    prepared: Prepared,
    identity: Identity,
    endpoint: String,
    merchant: MerchantConfig,
    intent: Option<DispatchApproval>,
    known: Option<Outcome>,
}
impl Wallet {
    fn load(args: &WalletArgs) -> Result<Self> {
        let store = open_store(&args.journal, true)?;
        Self::load_from_store(args, store)
    }
    fn read_only(args: &WalletArgs) -> Result<Self> {
        super::private_parent(&args.journal)?;
        super::private_file(&args.journal)?;
        Self::load_from_store(args, Store::open_read_only(&args.journal)?)
    }
    fn load_from_store(args: &WalletArgs, store: Store) -> Result<Self> {
        let envelope = crate::read_bounded(&args.invoice_inbox, MAX_ENVELOPE)?;
        let pin = crate::pin(&args.merchant_key)?;
        let (invoice, candidate, prepared) =
            super::saved_payment(&store, &envelope, &pin, args.expected_sender)?;
        // These checks are local and precede ALL RPCs and dispatch-intent writes.
        ensure!(
            candidate.offline_fixture_epoch.is_none(),
            "OFFLINE_FIXTURE_NEVER_SUBMITTABLE"
        );
        ensure!(
            args.expected_sender
                != sui_crypto::ed25519::Ed25519PrivateKey::new([9; 32])
                    .public_key()
                    .derive_address()
                && pin != crate::fixture_key(),
            "PUBLIC_DEFAULT_FIXTURE_NEVER_SUBMITTABLE"
        );
        ensure!(
            candidate.endpoint.as_deref() == Some(args.endpoint.as_str()),
            "wallet endpoint differs from approved candidate"
        );
        tap_sui::validate_endpoint(&args.endpoint)?;
        let merchant = MerchantConfig::new(&args.merchant_key, &args.merchant_endpoint)?;
        let identity = Identity::new(&invoice, &envelope, &pin, &prepared)?;
        let intent = store
            .live_dispatch(&identity.invoice_hash, &identity.digest)?
            .map(|s| parse(&s))
            .transpose()?;
        let mut wallet = Self {
            store,
            envelope,
            invoice,
            candidate,
            prepared,
            identity,
            endpoint: args.endpoint.clone(),
            merchant,
            intent,
            known: None,
        };
        wallet.validate_history()?;
        Ok(wallet)
    }
    fn validate_history(&mut self) -> Result<()> {
        if let Some(intent) = &self.intent {
            intent.validate(self)?;
            ensure!(
                intent.action == DispatchAction::SubmitTestnet,
                "invalid first dispatch intent action"
            );
        }
        let mut attempts = 0;
        self.store.live_history(
            &self.identity.invoice_hash,
            History::Attempts,
            |_, value| {
                let approval: DispatchApproval = parse(value)?;
                approval.validate(self)?;
                let intent = self
                    .intent
                    .as_ref()
                    .context("attempt without dispatch intent")?;
                ensure!(
                    tag(&approval.registration)? == tag(&intent.registration)?,
                    "attempt changed merchant registration"
                );
                if attempts == 0 {
                    ensure!(
                        tag(&approval)? == tag(intent)?,
                        "first attempt differs from intent"
                    );
                } else {
                    ensure!(
                        approval.action == DispatchAction::ReplaySavedTestnet
                            && approval.approved_ms >= intent.approved_ms,
                        "invalid replay approval"
                    );
                }
                attempts += 1;
                Ok(())
            },
        )?;
        ensure!(
            self.intent.is_some() == (attempts > 0),
            "incomplete dispatch intent/attempt state"
        );
        let mut known = None;
        let mut independently_confirmed = None;
        self.store.live_history(
            &self.identity.invoice_hash,
            History::Observations,
            |_, value| {
                let record: WalletObservation = parse(value)?;
                self.validate_observation(&record)?;
                if matches!(record.source, Source::MerchantConfirmation) {
                    terminal(&mut independently_confirmed, &record.observation.outcome)?;
                }
                terminal(&mut known, &record.observation.outcome)
            },
        )?;
        if let Some(value) = self
            .store
            .wallet_confirmation(&self.identity.invoice_hash)?
        {
            let receipt: Receipt = parse(&value)?;
            receipt.validate(
                &self.identity,
                &self.merchant,
                &self.envelope,
                &self.invoice,
                &self.prepared,
            )?;
            if let Some(intent) = &self.intent {
                ensure!(
                    tag(&receipt.registration)? == tag(&intent.registration)?,
                    "receipt registration differs from dispatch intent"
                );
            }
            ensure!(
                independently_confirmed.as_ref() == Some(&receipt.observation.outcome),
                "wallet association lacks matching durable node observation"
            );
        }
        self.known = known;
        Ok(())
    }
    fn validate_observation(&self, record: &WalletObservation) -> Result<()> {
        ensure!(
            record.identity == self.identity && record.observed_ms >= self.invoice.created_ms,
            "wallet observation binding mismatch"
        );
        let endpoint = match record.source {
            Source::Query => &self.endpoint,
            Source::MerchantConfirmation => &self.merchant.endpoint,
            Source::Dispatch { attempt } => {
                ensure!(
                    self.intent.is_some()
                        && self
                            .store
                            .live_attempt_exists(&self.identity.invoice_hash, attempt)?,
                    "observation references missing dispatch attempt"
                );
                &self.endpoint
            }
        };
        ensure!(
            &record.observation.endpoint == endpoint,
            "observation endpoint mismatch"
        );
        record.observation.validate(&self.invoice, &self.prepared)
    }
    async fn query(&mut self) -> Result<Outcome> {
        let observation =
            tap_sui::settlement::query(&self.endpoint, &self.invoice, &self.prepared).await;
        self.record(Source::Query, observation)
    }
    fn record(&mut self, source: Source, observation: Observation) -> Result<Outcome> {
        let record = WalletObservation {
            identity: self.identity.clone(),
            observed_ms: crate::now()?,
            source,
            observation,
        };
        self.validate_observation(&record)?;
        self.store
            .save_wallet_observation(&self.identity.invoice_hash, &json(&record)?)?;
        terminal(&mut self.known, &record.observation.outcome)?;
        println!(
            "WALLET_NODE_OBSERVATION {:?} merchant_credit=false digest={}",
            record.observation.outcome, self.identity.digest
        );
        Ok(record.observation.outcome)
    }
}

fn approval_prompt(wallet: &Wallet, replay: bool) -> Result<()> {
    ensure!(
        std::io::stdin().is_terminal(),
        "submission requires terminal approval; no unattended approval flag"
    );
    let action = if replay {
        "TESTNET REPLAY SAVED"
    } else {
        "TESTNET SUBMIT"
    };
    eprintln!(
        "\nTESTNET ONLY / {PROFILE}\nNo signing or key access. Sends ONLY this immutable artifact."
    );
    eprintln!(
        "Digest: {}\nSender: {}\nReceiver: {}\nAsset: {}\nAmount: {}.{:06} USDC ({} base units)\nGenesis: {}\nWallet endpoint: {}\nMerchant endpoint: {}\nMerchant pin: {}",
        wallet.identity.digest,
        wallet.identity.sender,
        Address::new(wallet.invoice.receiver),
        wallet.invoice.coin_type,
        wallet.invoice.amount / 1_000_000,
        wallet.invoice.amount % 1_000_000,
        wallet.invoice.amount,
        SUI_TESTNET_CHAIN,
        wallet.endpoint,
        wallet.merchant.endpoint,
        wallet.identity.merchant_pin
    );
    eprintln!("Type {action} to authorize possible broadcast of these exact saved bytes:");
    let mut text = String::new();
    std::io::stdin().lock().take(128).read_line(&mut text)?;
    ensure!(
        text.trim() == action,
        "submission declined; no dispatch intent or execution"
    );
    Ok(())
}

pub(super) async fn submit(args: SubmitArgs) -> Result<()> {
    let mut wallet = Wallet::load(&args.wallet)?;
    ensure!(
        !args.trace_execution_response
            || (wallet.intent.is_none()
                && wallet.known.is_none()
                && wallet.invoice.amount == 10_000),
        "EXECUTION_TRACE_REQUIRES_FIRST_0_01_USDC_DISPATCH"
    );
    ensure!(
        !args.fault_exit_after_execution_before_observation
            || (wallet.intent.is_none() && wallet.known.is_none()),
        "FAULT_TRIAL_REQUIRES_FRESH_DISPATCH: existing intent or historical terminal result; never rearm a settled/possibly-sent operation"
    );
    ensure!(
        !args.fault_exit_after_execution_before_observation || wallet.invoice.amount == 10_000,
        "FAULT_TRIAL_REQUIRES_0_01_USDC"
    );
    let registration: Registration = read_record(&args.registration)?;
    registration.validate(&wallet.merchant, &wallet.envelope, &wallet.invoice)?;
    if let Some(intent) = &wallet.intent {
        ensure!(
            tag(&intent.registration)? == tag(&registration)?,
            "registration differs from dispatch intent"
        );
    }
    let first = wallet.intent.is_none();
    ensure!(
        !first || !args.replay_saved,
        "--replay-saved requires existing dispatch intent"
    );
    // Declining first-send approval has no network or dispatch side effect.
    if first {
        if args.fault_exit_after_execution_before_observation {
            eprintln!(
                "TESTNET FAULT ARMED: 0.01 USDC first dispatch only; exit 86 after received verified success BEFORE observation persistence. Not dropped RPC. Restart reconcile, never resubmit automatically."
            );
        }
        approval_prompt(&wallet, false)?;
    }
    // On EVERY resume, query the saved digest before any fresh eligibility checks.
    let queried = wallet.query().await?;
    if args.fault_exit_after_execution_before_observation && queried != Outcome::NotFound {
        eprintln!(
            "FAULT_NOT_REACHED stage=execution-before-observation digest={} query={queried:?}; normal evidence retained",
            wallet.identity.digest
        );
    }
    match queried {
        Outcome::VerifiedSuccess { .. } => {
            println!("CHECKPOINT_OBSERVED_NOT_MERCHANT_CREDIT");
            return Ok(());
        }
        Outcome::VerifiedFailure { .. } => {
            anyhow::bail!("VERIFIED_FAILED; no replacement or automatic retry")
        }
        Outcome::Unknown => {
            anyhow::bail!("OUTCOME_UNKNOWN; saved bytes retained; reconcile before any retry")
        }
        Outcome::NotFound => {}
    }
    ensure!(
        wallet.known.is_none(),
        "historical verified result exists; NotFound cannot authorize resubmission"
    );
    ensure!(
        first || args.replay_saved,
        "POSSIBLY_SENT; NotFound is not proof of nonexecution; only explicit --replay-saved may resend identical bytes"
    );
    if !first {
        approval_prompt(&wallet, true)?;
    } else {
        super::fresh(&wallet.invoice)?;
    }
    let report = tap_sui::qualification::qualify_candidate(
        &wallet.endpoint,
        &wallet.invoice,
        &wallet.prepared.transaction()?,
        POLICY,
    )
    .await?;
    ensure!(
        report.qualified,
        "saved candidate not eligible: {:?}; never replace nonce/epoch/bytes",
        report.blockers
    );
    if first {
        super::fresh(&wallet.invoice)?;
    }
    let qualification = QualificationSummary {
        qualified: report.qualified,
        chain_submission: report.chain_submission,
        genesis: report.node.chain_id.parse()?,
        epoch: report.node.epoch,
        digest: report.transaction_digest,
        asset: report.asset.into(),
        protocol_version: report
            .protocol
            .and_then(|p| p.protocol_version)
            .context("missing qualified protocol version")?,
        checks_enabled: true,
        gas_selection: false,
    };
    let approval = DispatchApproval {
        identity: wallet.identity.clone(),
        endpoint: wallet.endpoint.clone(),
        registration,
        approved_ms: crate::now()?,
        action: if first {
            DispatchAction::SubmitTestnet
        } else {
            DispatchAction::ReplaySavedTestnet
        },
        qualification,
    };
    approval.validate(&wallet)?;
    let encoded = json(&approval)?;
    let attempt = wallet.store.begin_dispatch(
        &wallet.identity.invoice_hash,
        &wallet.identity.digest,
        first.then_some(encoded.as_str()),
        &encoded,
    )?;
    if first {
        wallet.intent = Some(approval);
    }
    eprintln!(
        "DISPATCH_INTENT_COMMITTED attempt={attempt} digest={} possible_effect=true",
        wallet.identity.digest
    );
    let observation = if args.trace_execution_response {
        let (observation, trace) = tap_sui::settlement::execute_saved_with_trace(
            &wallet.endpoint,
            &wallet.invoice,
            &wallet.prepared,
        )
        .await;
        eprintln!("EXECUTION_RESPONSE_TRANSPORT_TRACE {}", json(&trace)?);
        observation
    } else {
        tap_sui::settlement::execute_saved(&wallet.endpoint, &wallet.invoice, &wallet.prepared)
            .await
    };
    if args.fault_exit_after_execution_before_observation {
        if let Outcome::VerifiedSuccess { checkpoint } = observation.outcome {
            observation.validate(&wallet.invoice, &wallet.prepared)?;
            fault_exit(&format!(
                "FAULT_EXIT_AFTER_EXECUTION_BEFORE_OBSERVATION digest={} invoice_hash={} attempt={attempt} checkpoint={checkpoint} received_result_not_persisted=true exit_code=86",
                wallet.identity.digest, wallet.identity.invoice_hash
            ));
        }
        eprintln!(
            "FAULT_NOT_REACHED stage=execution-before-observation digest={} outcome={:?}; normal evidence retained",
            wallet.identity.digest, observation.outcome
        );
    }
    let outcome = wallet.record(Source::Dispatch { attempt }, observation)?;
    ensure!(
        matches!(outcome, Outcome::VerifiedSuccess { .. }),
        "execution not verified successful: {outcome:?}; saved artifact retained; no replacement"
    );
    println!(
        "EXECUTION_CHECKPOINT_OBSERVED merchant_credit=false; run independent merchant accept"
    );
    Ok(())
}

pub(super) async fn reconcile(args: WalletArgs) -> Result<()> {
    let mut wallet = Wallet::load(&args)?;
    let outcome = wallet.query().await?;
    ensure!(
        matches!(
            outcome,
            Outcome::VerifiedSuccess { .. } | Outcome::VerifiedFailure { .. }
        ),
        "UNRESOLVED {outcome:?}; no execution performed"
    );
    Ok(())
}

pub(super) fn register(args: RegisterArgs) -> Result<()> {
    let (envelope, invoice, hash) = invoice(&args.invoice, &args.merchant_key)?;
    let config = MerchantConfig::new(&args.merchant_key, &args.merchant_endpoint)?;
    let store = open_store(&args.ledger, false)?;
    let registration = if let Some(value) = store.merchant_registration(
        &hash,
        &config.chain.to_string(),
        &Address::new(invoice.receiver).to_string(),
    )? {
        let registration: Registration = parse(&value)?;
        ensure!(
            store.merchant_config()?.as_deref() == Some(json(&config)?.as_str()),
            "merchant namespace mismatch"
        );
        registration.validate(&config, &envelope, &invoice)?;
        registration
    } else {
        super::fresh(&invoice)?;
        let registration = Registration {
            config: config.clone(),
            invoice_hash: hash.clone(),
            envelope_hex: hex::encode(&envelope),
            receiver: Address::new(invoice.receiver),
            registered_ms: crate::now()?,
        };
        registration.validate(&config, &envelope, &invoice)?;
        store.register_merchant(
            &hash,
            &config.chain.to_string(),
            &registration.receiver.to_string(),
            &json(&config)?,
            &json(&registration)?,
        )?;
        registration
    };
    super::export(&args.out, &serde_json::to_vec_pretty(&registration)?)?;
    println!(
        "MERCHANT_RECEIVER_RESERVED invoice_hash={hash} chain_submission=false paid=false local_no_reuse_only=true"
    );
    Ok(())
}

pub(super) async fn accept(args: AcceptArgs) -> Result<()> {
    if args.fault_exit_after_credit_before_receipt {
        super::private_parent(&args.merchant.out)?;
        match std::fs::symlink_metadata(&args.merchant.out) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => anyhow::bail!(
                "FAULT_TRIAL_REQUIRES_ABSENT_RECEIPT: no existing file/link may be removed or replaced"
            ),
        }
    }
    let (envelope, invoice, hash) = invoice(&args.merchant.invoice, &args.merchant.merchant_key)?;
    ensure!(
        !args.fault_exit_after_credit_before_receipt || invoice.amount == 10_000,
        "FAULT_TRIAL_REQUIRES_0_01_USDC"
    );
    let config = MerchantConfig::new(
        &args.merchant.merchant_key,
        &args.merchant.merchant_endpoint,
    )?;
    let prepared: Prepared = read_record(&args.prepared)?;
    let identity = Identity::new(
        &invoice,
        &envelope,
        &crate::pin(&args.merchant.merchant_key)?,
        &prepared,
    )?;
    ensure!(
        identity.sender
            != sui_crypto::ed25519::Ed25519PrivateKey::new([9; 32])
                .public_key()
                .derive_address()
            && crate::pin(&args.merchant.merchant_key)? != crate::fixture_key(),
        "PUBLIC_DEFAULT_FIXTURE_NOT_LIVE_SETTLEMENT"
    );
    let store = open_store(&args.merchant.ledger, true)?;
    let stored_config: MerchantConfig = parse(
        &store
            .merchant_config()?
            .context("merchant not registered")?,
    )?;
    ensure!(stored_config == config, "merchant trust namespace mismatch");
    let registration: Registration = parse(
        &store
            .merchant_registration(
                &hash,
                &config.chain.to_string(),
                &Address::new(invoice.receiver).to_string(),
            )?
            .context("register fresh invoice receiver BEFORE dispatch")?,
    )?;
    registration.validate(&config, &envelope, &invoice)?;
    let mut known = None;
    store.live_history(&hash, History::MerchantObservations, |_, value| {
        let record: MerchantObservation = parse(value)?;
        ensure!(
            record.identity == identity
                && record.observed_ms >= registration.registered_ms
                && record.observation.endpoint == config.endpoint,
            "merchant observation binding mismatch; different artifact requires reconciliation"
        );
        record.observation.validate(&invoice, &prepared)?;
        terminal(&mut known, &record.observation.outcome)
    })?;
    if let Some(value) =
        store.merchant_credit(&hash, &config.chain.to_string(), &identity.digest)?
    {
        ensure!(
            !args.fault_exit_after_credit_before_receipt,
            "FAULT_TRIAL_REQUIRES_NEW_CREDIT: existing credit; never rearm"
        );
        let receipt: Receipt = parse(&value)?;
        receipt.validate(&identity, &config, &envelope, &invoice, &prepared)?;
        ensure!(
            tag(&receipt.registration)? == tag(&registration)?,
            "credited registration differs from receiver reservation"
        );
        let expected_observation = MerchantObservation {
            identity: identity.clone(),
            observed_ms: receipt.observed_ms,
            observation: receipt.observation.clone(),
        };
        ensure!(
            store.merchant_observation_exists(&hash, &json(&expected_observation)?)?,
            "credit lacks exact linked merchant observation"
        );
        ensure!(
            known.as_ref() == Some(&receipt.observation.outcome),
            "credit lacks durable verified observation"
        );
        super::export(&args.merchant.out, &serde_json::to_vec_pretty(&receipt)?)?;
        println!(
            "MERCHANT_EXISTING_IDENTICAL_CREDIT digest={}",
            identity.digest
        );
        return Ok(());
    }
    // No expiry/balance/current-epoch gate: a late or previously executed transfer still matters.
    let observation = tap_sui::settlement::query(&config.endpoint, &invoice, &prepared).await;
    observation.validate(&invoice, &prepared)?;
    let observed_ms = crate::now()?;
    let record = MerchantObservation {
        identity: identity.clone(),
        observed_ms,
        observation: observation.clone(),
    };
    if !matches!(observation.outcome, Outcome::VerifiedSuccess { .. }) {
        store.save_merchant_observation(&hash, &json(&record)?)?;
        if args.fault_exit_after_credit_before_receipt {
            eprintln!(
                "FAULT_NOT_REACHED stage=credit-before-receipt digest={} outcome={:?}; normal evidence retained",
                identity.digest, observation.outcome
            );
        }
        anyhow::bail!(
            "MERCHANT_NOT_CREDITED {:?}; observation retained",
            observation.outcome
        );
    }
    terminal(&mut known, &observation.outcome)?;
    let receipt = Receipt {
        identity,
        registration,
        observed_ms,
        observed_late: observed_ms >= invoice.expires_ms,
        observation,
    };
    receipt.validate(&receipt.identity, &config, &envelope, &invoice, &prepared)?;
    store.credit_merchant(
        &hash,
        &config.chain.to_string(),
        &receipt.identity.digest,
        &json(&record)?,
        &json(&receipt)?,
    )?;
    if args.fault_exit_after_credit_before_receipt {
        fault_exit(&format!(
            "FAULT_EXIT_AFTER_CREDIT_BEFORE_RECEIPT digest={} invoice_hash={} credit_committed=true receipt_exported=false exit_code=86",
            receipt.identity.digest, hash
        ));
    }
    super::export(&args.merchant.out, &serde_json::to_vec_pretty(&receipt)?)?;
    println!(
        "MERCHANT_CHECKPOINT_CREDIT digest={} observed_late={} trusted_fullnode_not_light_client=true",
        receipt.identity.digest, receipt.observed_late
    );
    Ok(())
}

fn fault_exit(marker: &str) -> ! {
    use std::io::Write;
    eprintln!("{marker}");
    let _ = std::io::stderr().flush();
    // Deliberately bypass destructors: all qualifying state is committed before this boundary.
    // There is no persistent arming bit; restart ordinary keyless commands, never the fault flag.
    std::process::exit(86)
}

pub(super) async fn confirm(args: ConfirmArgs) -> Result<()> {
    let mut wallet = Wallet::load(&args.wallet)?;
    let receipt: Receipt = read_record(&args.receipt)?;
    receipt.validate(
        &wallet.identity,
        &wallet.merchant,
        &wallet.envelope,
        &wallet.invoice,
        &wallet.prepared,
    )?;
    if let Some(intent) = &wallet.intent {
        ensure!(
            tag(&receipt.registration)? == tag(&intent.registration)?,
            "receipt registration differs from dispatch intent"
        );
    }
    if let Some(value) = wallet
        .store
        .wallet_confirmation(&wallet.identity.invoice_hash)?
    {
        let existing: Receipt = parse(&value)?;
        ensure!(
            tag(&existing)? == tag(&receipt)?,
            "conflicting wallet receipt association"
        );
        println!("WALLET_EXISTING_IDENTICAL_CHAIN_CONFIRMED_ASSOCIATION");
        return Ok(());
    }
    // Receipt files are NOT cryptographic business acknowledgements. Requery the caller-pinned
    // merchant node independently; never credit merely because a file says paid.
    let observation =
        tap_sui::settlement::query(&wallet.merchant.endpoint, &wallet.invoice, &wallet.prepared)
            .await;
    let record = WalletObservation {
        identity: wallet.identity.clone(),
        observed_ms: crate::now()?,
        source: Source::MerchantConfirmation,
        observation,
    };
    wallet.validate_observation(&record)?;
    if record.observation.outcome != receipt.observation.outcome {
        wallet
            .store
            .save_wallet_observation(&wallet.identity.invoice_hash, &json(&record)?)?;
        anyhow::bail!("receipt not independently confirmed; observation retained, no association");
    }
    terminal(&mut wallet.known, &record.observation.outcome)?;
    wallet.store.confirm_wallet(
        &wallet.identity.invoice_hash,
        &json(&record)?,
        &json(&receipt)?,
    )?;
    println!(
        "WALLET_CHAIN_CONFIRMED_RECEIPT_ASSOCIATED digest={} remote_business_ack_authenticated=false",
        wallet.identity.digest
    );
    Ok(())
}
