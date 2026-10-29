//! Explicit fixed-genesis testnet workflows, separate from legacy simulation defaults.
use anyhow::{Context, Result, ensure};
use clap::{Args, Subcommand};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, IsTerminal, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
use sui_crypto::ed25519::Ed25519PrivateKey;
use sui_sdk_types::{Address, Digest, Transaction, TransactionExpiration};
use tap_core::{DirectAssetPolicy, Invoice, MAX_ENVELOPE, SUI_TESTNET_CHAIN, SignedInvoice};
use tap_sui::{DevelopmentSigner, Prepared};

const POLICY: DirectAssetPolicy = DirectAssetPolicy::CircleTestnetUsdc;
const PROFILE: &str = "circle-sui-testnet/software-ed25519-host-approval/v1";
mod conversion;
mod init;
mod settlement;
mod verify_setup;

#[derive(Subcommand)]
pub enum Command {
    /// Explicit whole-coin to same-owner conversion: inspect, human-private prepare, keyless dispatch.
    Conversion(conversion::ConversionArgs),
    /// Human-only creation of three disposable testnet keystores; no RPC or funding.
    Init(init::InitArgs),
    /// Human-private read-only verification of setup keys against public READY metadata.
    VerifySetup(verify_setup::VerifySetupArgs),
    /// Create an explicitly testnet-only merchant invoice using a dedicated legacy Sui keystore.
    Invoice(InvoiceArgs),
    /// Freeze, review and prepare exact APDU inbox terms. Never broadcasts or records paid status.
    Prepare(PrepareArgs),
    /// Keyless exact-request seal for a separately armed operator lab; no RPC or submission.
    ExecutionSeal(settlement::SealArgs),
    /// Keyless read-only verification of a seal against the native first-dispatch approval barrier.
    CheckExecutionSeal(settlement::CheckSealArgs),
    /// Keyless local predicate for the separate operator gateway's unsigned checked simulation.
    CheckLabSimulation(settlement::SimulationArgs),
    /// Submit only an existing approved live-profile artifact, after explicit terminal approval.
    Submit(settlement::SubmitArgs),
    /// Keyless lookup of a saved digest; never signs or submits.
    Reconcile(settlement::WalletArgs),
    /// Reserve one fresh receiving address for one signed invoice in a local merchant ledger.
    Register(settlement::RegisterArgs),
    /// Independently query the merchant's chosen node and transactionally credit once.
    Accept(settlement::AcceptArgs),
    /// Independently verify a receipt's chain observation before wallet association.
    Confirm(settlement::ConfirmArgs),
}

#[derive(Args)]
pub struct KeyArgs {
    /// Absolute path to a dedicated, owned mode-0600 legacy Sui keystore in a private directory.
    #[arg(long)]
    keystore: PathBuf,
    /// Independently expected public Ed25519 address, not a key supplied by the invoice.
    #[arg(long)]
    expected_sender: Address,
}

#[derive(Args)]
pub struct InvoiceArgs {
    #[command(flatten)]
    key: KeyArgs,
    #[arg(long)]
    receiver: Address,
    #[arg(long, default_value_t = 10_000)]
    amount: u64,
    #[arg(long)]
    merchant: String,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Args)]
pub struct PrepareArgs {
    #[command(flatten)]
    key: KeyArgs,
    /// Completed Rust APDU receiver envelope; transport acknowledgement is not approval.
    #[arg(long)]
    invoice_inbox: PathBuf,
    /// Independently pinned merchant public key, never inferred from the envelope.
    #[arg(long)]
    merchant_key: String,
    #[arg(long)]
    journal: PathBuf,
    #[arg(long)]
    out: PathBuf,
    /// Explicitly trusted testnet node. Qualifies these actual candidate bytes before signing.
    #[arg(
        long,
        required_unless_present = "offline_fixture_epoch",
        conflicts_with = "offline_fixture_epoch"
    )]
    endpoint: Option<String>,
    /// Offline laboratory ONLY: restricted to public sender [9;32] and merchant [7;32] fixture keys.
    /// Never qualified or ready to submit. There is no unattended approval mode.
    #[arg(long)]
    offline_fixture_epoch: Option<u64>,
    /// Explicit deterministic software recovery of a reserved candidate lacking its artifact.
    #[arg(long)]
    recover_saved_candidate: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    profile: String,
    envelope_hex: String,
    merchant_pin: String,
    core_hash: String,
    endpoint: Option<String>,
    offline_fixture_epoch: Option<u64>,
    digest: String,
    bcs_hex: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Approval {
    profile: String,
    digest: String,
    approved_ms: u64,
    action: String,
    qualification: Option<QualificationSummary>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QualificationSummary {
    qualified: bool,
    chain_submission: bool,
    genesis: Digest,
    epoch: u64,
    digest: String,
    asset: String,
    protocol_version: u64,
    checks_enabled: bool,
    gas_selection: bool,
}

impl QualificationSummary {
    fn validate(&self, candidate: &Candidate, invoice: &Invoice, tx: &Transaction) -> Result<()> {
        let epoch = match tx.expiration {
            TransactionExpiration::ValidDuring {
                min_epoch: Some(min),
                max_epoch: Some(max),
                ..
            } if min == max => min,
            _ => anyhow::bail!("unsupported candidate expiration"),
        };
        ensure!(
            candidate.offline_fixture_epoch.is_none()
                && candidate.endpoint.is_some()
                && self.qualified
                && !self.chain_submission
                && self.genesis == Digest::new(invoice.chain)
                && self.epoch == epoch
                && self.digest == candidate.digest
                && self.asset == invoice.coin_type
                && self.protocol_version > 0
                && self.checks_enabled
                && !self.gas_selection,
            "qualification summary contradicts saved candidate or required checks"
        );
        Ok(())
    }
}

impl Approval {
    fn validate(
        &self,
        candidate: &Candidate,
        invoice: &Invoice,
        tx: &Transaction,
        action: &str,
    ) -> Result<()> {
        ensure!(
            self.profile == PROFILE && self.digest == candidate.digest && self.action == action,
            "approval profile/digest/action mismatch"
        );
        ensure!(
            self.approved_ms >= invoice.created_ms && self.approved_ms < invoice.expires_ms,
            "approval timestamp outside invoice lifetime"
        );
        // Historical integrity, not a current eligibility gate: saved artifacts may be old.
        let epoch = match tx.expiration {
            TransactionExpiration::ValidDuring {
                min_epoch: Some(min),
                max_epoch: Some(max),
                ..
            } if min == max => min,
            _ => anyhow::bail!("unsupported candidate expiration"),
        };
        match (
            candidate.offline_fixture_epoch,
            &candidate.endpoint,
            &self.qualification,
        ) {
            (Some(fixture_epoch), None, None) => {
                ensure!(fixture_epoch == epoch, "offline candidate epoch mismatch")
            }
            (None, Some(_), Some(q)) => q.validate(candidate, invoice, tx)?,
            _ => anyhow::bail!("qualification does not match saved candidate mode"),
        }
        Ok(())
    }
}

fn genesis() -> Result<[u8; 32]> {
    Ok(SUI_TESTNET_CHAIN.parse::<Digest>()?.into_inner())
}

fn fresh(invoice: &Invoice) -> Result<()> {
    POLICY.validate(invoice)?;
    invoice.validate_checkout(super::now()?, &genesis()?, &[POLICY.coin_type()])
}

// A private immediate parent plus canonical absolute path prevents surprising symlink aliases.
// Same-user malicious path replacement is outside this cooperative local custody profile.
fn private_parent(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "use an absolute dedicated path");
    let parent = path.parent().context("missing parent")?;
    ensure!(
        parent.canonicalize()? == parent,
        "symlink or noncanonical parent rejected"
    );
    let m = std::fs::symlink_metadata(parent)?;
    let uid = effective_uid();
    ensure!(
        m.is_dir() && m.uid() == uid && m.mode() & 0o777 == 0o700,
        "parent must be owned and mode 0700"
    );
    Ok(())
}

fn private_file(path: &Path) -> Result<File> {
    private_parent(path)?;
    let before = std::fs::symlink_metadata(path)?;
    ensure!(
        before.is_file() && before.nlink() == 1,
        "owned regular file required; links rejected"
    );
    let file = File::open(path)?;
    let m = file.metadata()?;
    ensure!(
        m.dev() == before.dev() && m.ino() == before.ino(),
        "file changed while opening"
    );
    ensure!(
        m.uid() == effective_uid() && m.mode() & 0o777 == 0o600,
        "file must be owned and mode 0600"
    );
    Ok(file)
}

fn effective_uid() -> u32 {
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    // SAFETY: Linux geteuid takes no arguments and always returns the effective UID.
    // /proc/self ownership can become root after checked dump protection is enabled.
    unsafe { geteuid() }
}

fn key(args: &KeyArgs) -> Result<Ed25519PrivateKey> {
    key_from_file(private_file(&args.keystore)?, args.expected_sender, 32)
}

fn key_from_file(
    mut file: File,
    expected_sender: Address,
    max_entries: usize,
) -> Result<Ed25519PrivateKey> {
    // Pinned official SDK ed25519.rs: from_base64 decodes legacy flag||private_key and
    // rejects non-Ed25519 schemes/lengths. Never parse or encode seed formats ourselves.
    let mut bytes = Vec::new();
    (&mut file).take(16_385).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 16_384, "dedicated keystore exceeds limit");
    let entries: Result<Vec<String>, _> = serde_json::from_slice(&bytes);
    bytes.fill(0);
    let entries = entries.map_err(|_| anyhow::anyhow!("invalid legacy keystore JSON"))?;
    ensure!(
        !entries.is_empty() && entries.len() <= max_entries,
        "invalid dedicated keystore entry count"
    );
    let mut selected = None;
    for entry in entries {
        let decoded = Ed25519PrivateKey::from_base64(&entry)
            .map_err(|_| anyhow::anyhow!("keystore requires valid Ed25519 legacy entries"))?;
        if decoded.public_key().derive_address() == expected_sender {
            ensure!(selected.is_none(), "ambiguous duplicate sender");
            selected = Some(decoded);
        }
    }
    // No guaranteed erasure of JSON strings or SDK decoder temporaries; not production custody.
    selected.context("expected public sender absent from dedicated keystore")
}

fn export(path: &Path, bytes: &[u8]) -> Result<()> {
    private_parent(path)?;
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
    {
        Ok(mut file) => {
            file.write_all(bytes)?;
            file.sync_all()?;
            File::open(path.parent().context("missing parent")?)?.sync_all()?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let mut stored = Vec::new();
            private_file(path)?.take(65_537).read_to_end(&mut stored)?;
            ensure!(
                stored == bytes,
                "existing artifact differs; refusing replacement"
            );
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::Init(args) => init::run(args),
        Command::Conversion(args) => conversion::run(args).await,
        Command::VerifySetup(args) => verify_setup::run(args),
        Command::Invoice(args) => {
            let mut id = [0; 16];
            rand::rngs::OsRng.fill_bytes(&mut id);
            let created_ms = super::now()?;
            let invoice = Invoice {
                chain: genesis()?,
                id,
                receiver: args.receiver.into_inner(),
                coin_type: POLICY.coin_type().into(),
                amount: args.amount,
                created_ms,
                expires_ms: created_ms + 900_000,
                merchant: args.merchant,
            };
            fresh(&invoice)?;
            ensure!(
                invoice.amount >= 10_000,
                "USDC amount below documented 0.01 minimum"
            );
            let key = key(&args.key)?;
            let signed = SignedInvoice::sign(&invoice, &key)?;
            export(&args.out, &serde_json::to_vec_pretty(&signed)?)?;
            println!(
                "TESTNET_INVOICE merchant_key={} chain_submission=false",
                signed.merchant_key_hex
            );
            Ok(())
        }
        Command::Prepare(args) => prepare(args).await,
        Command::ExecutionSeal(args) => settlement::export_execution_seal(args),
        Command::CheckExecutionSeal(args) => settlement::check_execution_seal(args),
        Command::CheckLabSimulation(args) => settlement::check_lab_simulation(args),
        Command::Submit(args) => settlement::submit(args).await,
        Command::Reconcile(args) => settlement::reconcile(args).await,
        Command::Register(args) => settlement::register(args),
        Command::Accept(args) => settlement::accept(args).await,
        Command::Confirm(args) => settlement::confirm(args).await,
    }
}

async fn prepare(args: PrepareArgs) -> Result<()> {
    let envelope = super::read_bounded(&args.invoice_inbox, MAX_ENVELOPE)?;
    let signed: SignedInvoice = serde_json::from_slice(&envelope)?;
    let merchant_pin = super::pin(&args.merchant_key)?;
    let invoice = signed.verify(&merchant_pin)?;
    POLICY.validate(&invoice)?;
    if args.offline_fixture_epoch.is_some() {
        ensure!(
            args.key.expected_sender
                == Ed25519PrivateKey::new([9; 32])
                    .public_key()
                    .derive_address()
                && merchant_pin == super::fixture_key(),
            "offline preparation accepts PUBLIC laboratory keys only"
        );
    }
    private_parent(&args.journal)?;
    match std::fs::symlink_metadata(&args.journal) {
        Ok(_) => {
            private_file(&args.journal)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let store = super::store::Store::open(&args.journal)?;
    let hash = hex::encode(signed.invoice_hash()?);
    let candidate: Candidate = match store.testnet_candidate(&hash)? {
        Some(value) => {
            ensure!(value.len() <= 65_536, "candidate exceeds limit");
            serde_json::from_str(&value)?
        }
        None => {
            ensure!(
                !args.recover_saved_candidate,
                "no saved candidate to recover"
            );
            fresh(&invoice)?;
            let epoch = match args.offline_fixture_epoch {
                Some(epoch) => epoch,
                None => {
                    let info =
                        tap_sui::probe(args.endpoint.as_deref().context("endpoint required")?)
                            .await?;
                    ensure!(
                        info.chain.as_deref() == Some("testnet")
                            && info.chain_id == SUI_TESTNET_CHAIN,
                        "pinned testnet required"
                    );
                    info.epoch
                }
            };
            let tx = tap_sui::build_direct_with_policy(
                &invoice,
                args.key.expected_sender,
                epoch,
                rand::random(),
                POLICY,
            )?;
            let candidate = Candidate {
                profile: PROFILE.into(),
                envelope_hex: hex::encode(&envelope),
                merchant_pin: hex::encode(merchant_pin),
                core_hash: hash.clone(),
                endpoint: args.endpoint.clone(),
                offline_fixture_epoch: args.offline_fixture_epoch,
                digest: tx.digest().to_string(),
                bcs_hex: hex::encode(bcs::to_bytes(&tx)?),
            };
            let mut issuance = merchant_pin.to_vec();
            issuance.extend_from_slice(&invoice.chain);
            issuance.extend_from_slice(&invoice.id);
            store.save_testnet_candidate(
                &hash,
                &hex::encode(tap_core::hash(&issuance)),
                &serde_json::to_string(&candidate)?,
            )?;
            candidate
        }
    };
    ensure!(
        candidate.profile == PROFILE
            && candidate.envelope_hex == hex::encode(&envelope)
            && candidate.merchant_pin == hex::encode(merchant_pin)
            && candidate.core_hash == hash
            && candidate.endpoint == args.endpoint
            && candidate.offline_fixture_epoch == args.offline_fixture_epoch,
        "saved candidate trust/profile mismatch; refusing replacement"
    );
    ensure!(candidate.bcs_hex.len() <= 32_768, "candidate exceeds limit");
    let tx: Transaction = bcs::from_bytes(&hex::decode(&candidate.bcs_hex)?)?;
    tap_sui::verify_template_with_policy(&invoice, &tx, POLICY)?;
    ensure!(
        tx.sender == args.key.expected_sender && tx.digest().to_string() == candidate.digest,
        "candidate sender/digest mismatch"
    );
    let reserved = validate_approvals(&store, &candidate, &invoice, &tx)?;
    if let Some(prepared) = store.testnet_artifact(&hash)? {
        ensure!(reserved, "artifact lacks issuance reservation");
        verify_artifact(&invoice, &tx, &prepared)?;
        export(&args.out, &serde_json::to_vec_pretty(&prepared)?)?;
        println!(
            "SAVED_TESTNET_ARTIFACT digest={} no_new_signing=true chain_submission=false status=PREPARED_NOT_PAID ready_to_submit=false live_qualification_required_before_future_dispatch=true",
            prepared.digest
        );
        return Ok(());
    }
    let reserved = reserved.then_some(());
    ensure!(
        reserved.is_none() || args.recover_saved_candidate,
        "UNRESOLVED_ISSUANCE: reservation without artifact; explicit --recover-saved-candidate required, never build a new transfer"
    );
    ensure!(
        reserved.is_some() || !args.recover_saved_candidate,
        "candidate was not reserved; use normal approval"
    );
    fresh(&invoice)?;
    ensure!(
        std::io::stdin().is_terminal(),
        "testnet approval requires terminal input; no automation approval flag exists"
    );
    eprintln!("\nTESTNET ONLY / PREPARATION, NOT PAYMENT / {PROFILE}");
    eprintln!(
        "Merchant pin: {} (local trust, not a verified business)",
        candidate.merchant_pin
    );
    eprintln!(
        "Amount: {}.{:06} USDC ({} base units)\nAsset: {}\nRecipient: {}",
        invoice.amount / 1_000_000,
        invoice.amount % 1_000_000,
        invoice.amount,
        invoice.coin_type,
        Address::new(invoice.receiver)
    );
    eprintln!(
        "Sender: {}\nGenesis: {}\nExpiry UTC ms: {}\nInvoice core hash: {}\nCandidate digest: {}",
        tx.sender, SUI_TESTNET_CHAIN, invoice.expires_ms, hash, candidate.digest
    );
    eprintln!(
        "Offline fixture: {} / software host approval / no hardware trust",
        args.offline_fixture_epoch.is_some()
    );
    eprintln!(
        "Expiration: {:?}\nGas: no objects, price=0, budget=0 (protocol eligibility still required)",
        tx.expiration
    );
    let action = if reserved.is_some() {
        "RECOVER TESTNET"
    } else {
        "APPROVE TESTNET"
    };
    eprintln!("Type {action} to sign ONLY this saved candidate. No transaction will be submitted:");
    let mut confirmation = String::new();
    std::io::stdin()
        .lock()
        .take(128)
        .read_line(&mut confirmation)?;
    ensure!(confirmation.trim() == action, "not approved");
    let qualification = if let Some(endpoint) = &args.endpoint {
        let report =
            tap_sui::qualification::qualify_candidate(endpoint, &invoice, &tx, POLICY).await?;
        ensure!(
            report.qualified,
            "actual candidate NOT QUALIFIED: {:?}; saved bytes retained",
            report.blockers
        );
        Some(QualificationSummary {
            qualified: report.qualified,
            chain_submission: report.chain_submission,
            genesis: report.node.chain_id.parse()?,
            epoch: report.node.epoch,
            digest: report.transaction_digest,
            asset: report.asset.into(),
            protocol_version: report
                .protocol
                .and_then(|p| p.protocol_version)
                .context("qualification omitted protocol version")?,
            checks_enabled: true,
            gas_selection: false,
        })
    } else {
        None
    };
    fresh(&invoice)?;
    let approval = Approval {
        profile: PROFILE.into(),
        digest: candidate.digest.clone(),
        approved_ms: super::now()?,
        action: action.into(),
        qualification,
    };
    approval.validate(&candidate, &invoice, &tx, action)?;
    let approval = serde_json::to_string(&approval)?;
    if reserved.is_none() {
        store.reserve_testnet(&hash, &approval)?;
    } else {
        store.approve_testnet_recovery(&hash, &approval)?;
    }
    let signer = DevelopmentSigner(key(&args.key)?);
    fresh(&invoice)?;
    let prepared = signer.sign_payment_with_policy(&invoice, &tx, POLICY)?;
    verify_artifact(&invoice, &tx, &prepared)?;
    store.save_testnet_artifact(&hash, &prepared)?;
    export(&args.out, &serde_json::to_vec_pretty(&prepared)?)?;
    println!(
        "TESTNET_PREPARED digest={} chain_submission=false status=PREPARED_NOT_PAID ready_to_submit=false live_qualification_required_before_future_dispatch=true",
        prepared.digest
    );
    Ok(())
}

fn validate_approvals(
    store: &super::store::Store,
    candidate: &Candidate,
    invoice: &Invoice,
    tx: &Transaction,
) -> Result<bool> {
    let reserved = store.testnet_reservation(&candidate.core_hash)?;
    if let Some(value) = &reserved {
        ensure!(value.len() <= 65_536, "approval exceeds limit");
        let approval: Approval = serde_json::from_str(value)?;
        approval
            .validate(candidate, invoice, tx, "APPROVE TESTNET")
            .context("invalid reservation approval")?;
    }
    store.check_testnet_recovery_approvals(&candidate.core_hash, |value| {
        ensure!(
            reserved.is_some(),
            "recovery approval lacks issuance reservation"
        );
        ensure!(value.len() <= 65_536, "recovery approval exceeds limit");
        let approval: Approval =
            serde_json::from_str(value).context("malformed recovery approval")?;
        approval
            .validate(candidate, invoice, tx, "RECOVER TESTNET")
            .context("invalid recovery approval")
    })?;
    Ok(reserved.is_some())
}

fn saved_payment(
    store: &super::store::Store,
    envelope: &[u8],
    merchant_pin: &[u8; 32],
    sender: Address,
) -> Result<(Invoice, Candidate, Prepared)> {
    let signed: SignedInvoice = serde_json::from_slice(envelope)?;
    let invoice = signed.verify(merchant_pin)?;
    POLICY.validate(&invoice)?;
    let hash = hex::encode(signed.invoice_hash()?);
    let value = store
        .testnet_candidate(&hash)?
        .context("no saved candidate; submission never prepares or signs")?;
    ensure!(value.len() <= 65_536, "candidate exceeds limit");
    let candidate: Candidate = serde_json::from_str(&value)?;
    ensure!(
        candidate.profile == PROFILE
            && candidate.core_hash == hash
            && candidate.merchant_pin == hex::encode(merchant_pin)
            && candidate.envelope_hex == hex::encode(envelope),
        "saved candidate trust/profile mismatch"
    );
    ensure!(candidate.bcs_hex.len() <= 32_768, "candidate exceeds limit");
    let tx: Transaction = bcs::from_bytes(&hex::decode(&candidate.bcs_hex)?)?;
    ensure!(
        tx.sender == sender && tx.digest().to_string() == candidate.digest,
        "candidate sender/digest mismatch"
    );
    tap_sui::verify_template_with_policy(&invoice, &tx, POLICY)?;
    ensure!(
        validate_approvals(store, &candidate, &invoice, &tx)?,
        "artifact lacks issuance reservation"
    );
    let prepared = store
        .testnet_artifact(&hash)?
        .context("no saved artifact; recover preparation separately")?;
    verify_artifact(&invoice, &tx, &prepared)?;
    Ok((invoice, candidate, prepared))
}

fn verify_artifact(invoice: &Invoice, candidate: &Transaction, prepared: &Prepared) -> Result<()> {
    let tx = prepared.transaction()?;
    tap_sui::verify_template_with_policy(invoice, &tx, POLICY)?;
    ensure!(
        tx == *candidate && prepared.bcs_hex == hex::encode(bcs::to_bytes(candidate)?),
        "artifact differs from reserved exact candidate"
    );
    Ok(())
}
