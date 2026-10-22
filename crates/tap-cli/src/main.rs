mod store;
use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use rand::RngCore;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{IsTerminal, Read},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sui_crypto::ed25519::Ed25519PrivateKey;
use sui_sdk_types::Digest;
use tap_core::{Invoice, MAX_ENVELOPE, SignedInvoice, USDC, apdu};
use tap_sui::{DevelopmentSigner, PaymentSigner, Prepared};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

#[derive(Parser)]
#[command(
    about = "Experimental native Linux payments assembly. No broadcast command is provided.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Local merchant simulator. All keys and chain identity are public laboratory fixtures.
    ServeDemo {
        #[arg(long)]
        socket: Option<PathBuf>,
        #[arg(long)]
        stdio: bool,
        #[arg(long)]
        db: PathBuf,
        #[arg(long, default_value_t = 1)]
        connections: usize,
    },
    /// Approve a SIMULATED checkout; never sends a transaction to a blockchain.
    PayDemo {
        #[arg(long)]
        socket: Option<PathBuf>,
        #[arg(long)]
        stdio: bool,
        #[arg(long)]
        journal: PathBuf,
        #[arg(long)]
        approve_simulation: bool,
        /// Review exact terms and type APPROVE SIMULATION in a terminal before signing/retry.
        #[arg(long, requires = "socket", conflicts_with = "approve_simulation")]
        interactive_simulation: bool,
        #[arg(long)]
        disconnect_after_journal: bool,
    },
    /// Inspect real Sui gRPC service identity. Read-only.
    Probe {
        #[arg(long)]
        endpoint: String,
    },
    /// Read-only testnet qualification of the exact pinned USDC template. No signing or broadcast.
    QualifyTestnet {
        #[arg(long)]
        endpoint: String,
        /// Full expected genesis digest, pinned independently of this endpoint.
        #[arg(long)]
        chain_id: Digest,
        /// Public disposable testnet address only; no key is read.
        #[arg(long)]
        sender: sui_sdk_types::Address,
        #[arg(long)]
        receiver: sui_sdk_types::Address,
        #[arg(long, default_value_t = 10_000)]
        amount: u64,
    },
    /// Read-only verification against an explicitly trusted node; no local paid status is written.
    VerifyOnNode {
        #[arg(long)]
        endpoint: String,
        #[arg(long)]
        invoice: PathBuf,
        #[arg(long)]
        merchant_key: String,
        #[arg(long)]
        prepared: PathBuf,
    },
    /// Offline preparation with an explicit development key; not a network eligibility check.
    PrepareDraft {
        #[arg(long)]
        invoice: PathBuf,
        #[arg(long)]
        merchant_key: String,
        #[arg(long)]
        chain_id: String,
        #[arg(long)]
        seed_file: PathBuf,
        #[arg(long)]
        epoch: u64,
        #[arg(long)]
        out: PathBuf,
    },
    /// Generate APDU frames and prove receiver reconstruction in software.
    ApduRoundtrip {
        #[arg(long)]
        input: PathBuf,
    },
    /// Pipe APDUs to the Rust receiver. Optional nfcd development bridge uses this.
    ApduStdio {
        #[arg(long)]
        out: PathBuf,
    },
    /// List actual PC/SC readers. Build with --features pcsc-reader.
    Readers,
    /// Push an invoice envelope to our experimental card-emulation application; acknowledgement only.
    NfcPush {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        reader: String,
    },
}

fn now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}
fn read_bounded(path: &Path, max: usize) -> Result<Vec<u8>> {
    let mut b = vec![];
    std::fs::File::open(path)?
        .take((max + 1) as u64)
        .read_to_end(&mut b)?;
    ensure!(b.len() <= max, "file exceeds limit");
    Ok(b)
}
fn read_json<T: DeserializeOwned>(p: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&read_bounded(p, 65_536)?)?)
}
fn pin(s: &str) -> Result<[u8; 32]> {
    hex::decode(s)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 32-byte merchant public key"))
}
fn fixture_key() -> [u8; 32] {
    Ed25519PrivateKey::new([7; 32])
        .public_key()
        .as_bytes()
        .try_into()
        .unwrap()
}

async fn send<T: Serialize, W: AsyncWrite + Unpin>(s: &mut W, value: &T) -> Result<()> {
    let b = serde_json::to_vec(value)?;
    ensure!(b.len() <= 65_536, "frame too large");
    s.write_u32(b.len() as u32).await?;
    s.write_all(&b).await?;
    s.flush().await?;
    Ok(())
}
async fn receive<T: DeserializeOwned, R: AsyncRead + Unpin>(s: &mut R) -> Result<T> {
    // Bound stalled sessions while allowing time to review the terminal approval.
    tokio::time::timeout(Duration::from_secs(120), async {
        let n = s.read_u32().await? as usize;
        ensure!(n <= 65_536, "frame too large");
        let mut b = vec![0; n];
        s.read_exact(&mut b).await?;
        Ok(serde_json::from_slice(&b)?)
    })
    .await?
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    invoice_hash: String,
    prepared: Prepared,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    status: String,
    digest: String,
    replay: bool,
}

async fn serve(
    socket: Option<PathBuf>,
    stdio: bool,
    db: PathBuf,
    connections: usize,
) -> Result<()> {
    ensure!(
        connections > 0 && connections <= 100,
        "invalid connection count"
    );
    ensure!(
        stdio != socket.is_some(),
        "choose exactly one of --socket or --stdio"
    );
    let ledger = store::Store::open(&db)?;
    let signed = match ledger.load_invoice()? {
        Some(s) => s,
        None => {
            let mut id = [0; 16];
            rand::rngs::OsRng.fill_bytes(&mut id);
            let t = now()?;
            // Synthetic recipient is unique per invoice and has no corresponding spend key here.
            let i = Invoice {
                chain: [42; 32],
                id,
                receiver: tap_core::hash(&id),
                coin_type: USDC.into(),
                amount: 10_000,
                created_ms: t,
                expires_ms: t + 900_000,
                merchant: "LOCAL SIMULATION ONLY".into(),
            };
            let s = SignedInvoice::sign(&i, &Ed25519PrivateKey::new([7; 32]))?;
            ledger.save_invoice(&s)?;
            s
        }
    };
    let invoice = signed.verify(&fixture_key())?;
    tap_core::reject_mainnet_simulation(&invoice.chain)?;
    if stdio {
        eprintln!("SIMULATION_READY transport=pipes");
        let mut reader = tokio::io::stdin();
        let mut writer = tokio::io::stdout();
        for _ in 0..connections {
            merchant_exchange(&mut reader, &mut writer, &signed, &invoice, &ledger).await?;
        }
        return Ok(());
    }
    let socket = socket.context("socket missing")?;
    let parent = socket.parent().context("socket parent missing")?;
    ensure!(
        std::fs::metadata(parent)?.permissions().mode() & 0o077 == 0,
        "socket directory must be private mode 0700"
    );
    let listener = UnixListener::bind(&socket)
        .context("socket already exists or cannot be bound; no existing socket is deleted")?;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))?;
    println!(
        "SIMULATION_READY merchant_key={} socket={}",
        hex::encode(fixture_key()),
        socket.display()
    );
    for _ in 0..connections {
        let (mut stream, _) = listener.accept().await?;
        let (mut reader, mut writer) = stream.split();
        if let Err(error) =
            merchant_exchange(&mut reader, &mut writer, &signed, &invoice, &ledger).await
        {
            eprintln!("SIMULATION_SESSION_FAILED: {error}");
        }
    }
    drop(listener);
    std::fs::remove_file(socket)?;
    Ok(())
}

async fn merchant_exchange<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    reader: &mut R,
    writer: &mut W,
    signed: &SignedInvoice,
    invoice: &Invoice,
    ledger: &store::Store,
) -> Result<()> {
    send(writer, signed).await?;
    let claim: Claim = receive(reader).await?;
    let ih = hex::encode(signed.invoice_hash()?);
    ensure!(claim.invoice_hash == ih, "invoice mismatch");
    let tx = claim.prepared.transaction()?;
    tap_sui::verify_template(invoice, &tx)?;
    if ledger.payment_digest(&ih)?.is_none() {
        invoice.validate_checkout(now()?, &[42; 32], &[USDC])?;
    }
    let replay = ledger.accept_simulated(&ih, &claim.prepared)?;
    let receipt = Receipt {
        status: "SIMULATED_PAID_NOT_ON_CHAIN".into(),
        digest: claim.prepared.digest,
        replay,
    };
    send(writer, &receipt).await?;
    eprintln!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}

async fn pay(
    socket: Option<PathBuf>,
    stdio: bool,
    journal: PathBuf,
    approve: bool,
    interactive: bool,
    disconnect: bool,
) -> Result<()> {
    ensure!(
        approve || interactive,
        "choose --interactive-simulation with a socket, or --approve-simulation for fixture automation"
    );
    ensure!(
        !interactive || (!stdio && std::io::stdin().is_terminal()),
        "interactive simulation requires a socket and terminal input"
    );
    ensure!(
        stdio != socket.is_some(),
        "choose exactly one of --socket or --stdio"
    );
    if stdio {
        pay_exchange(
            &mut tokio::io::stdin(),
            &mut tokio::io::stdout(),
            journal,
            interactive,
            disconnect,
        )
        .await
    } else {
        let mut s = UnixStream::connect(socket.context("socket missing")?).await?;
        let (mut r, mut w) = s.split();
        pay_exchange(&mut r, &mut w, journal, interactive, disconnect).await
    }
}

async fn pay_exchange<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    reader: &mut R,
    writer: &mut W,
    journal: PathBuf,
    interactive: bool,
    disconnect: bool,
) -> Result<()> {
    let signed: SignedInvoice = receive(reader).await?;
    let invoice = signed.verify(&fixture_key())?;
    tap_core::reject_mainnet_simulation(&invoice.chain)?;
    if interactive {
        invoice.validate_checkout(now()?, &[42; 32], &[USDC])?;
        eprintln!("\nLINUX TAP / SIMULATION ONLY / NO FUNDS MOVE");
        eprintln!("Merchant trust: pinned public laboratory key (not a verified business)");
        eprintln!("Merchant key: {}", hex::encode(fixture_key()));
        eprintln!("Network: synthetic laboratory chain; not Sui mainnet or testnet");
        eprintln!(
            "Amount: {}.{:06} USDC ({} base units)",
            invoice.amount / 1_000_000,
            invoice.amount % 1_000_000,
            invoice.amount
        );
        eprintln!("Asset type: {}", invoice.coin_type);
        eprintln!("Recipient: 0x{}", hex::encode(invoice.receiver));
        eprintln!("Invoice hash: {}", hex::encode(signed.invoice_hash()?));
        eprintln!(
            "Approval: development software signer; public fixture key, no hardware approval"
        );
        eprintln!("Type APPROVE SIMULATION to sign or retry these exact terms:");
        let mut confirmation = String::new();
        std::io::stdin().read_line(&mut confirmation)?;
        ensure!(confirmation.trim() == "APPROVE SIMULATION", "not approved");
    }
    let ih = hex::encode(signed.invoice_hash()?);
    let journal = store::Store::open(&journal)?;
    let prepared = match journal.load_prepared(&ih)? {
        Some(p) => p,
        None => {
            invoice.validate_checkout(now()?, &[42; 32], &[USDC])?;
            eprintln!(
                "SIMULATION_APPROVAL amount_base_units={} asset={} recipient={}",
                invoice.amount,
                invoice.coin_type,
                hex::encode(invoice.receiver)
            );
            let key = Ed25519PrivateKey::new([9; 32]);
            let tx = tap_sui::build_direct(
                &invoice,
                key.public_key().derive_address(),
                42,
                rand::random(),
            )?;
            let p = DevelopmentSigner(key).sign_payment(&invoice, &tx)?;
            journal.save_prepared(&ih, &p)?;
            p
        }
    };
    tap_sui::verify_template(&invoice, &prepared.transaction()?)?;
    eprintln!("JOURNALED digest={}", prepared.digest);
    if disconnect {
        eprintln!("SIMULATED_INTERRUPT_BEFORE_SEND");
        return Ok(());
    }
    send(
        writer,
        &Claim {
            invoice_hash: ih.clone(),
            prepared: prepared.clone(),
        },
    )
    .await?;
    let r: Receipt = receive(reader).await?;
    ensure!(
        r.digest == prepared.digest && r.status == "SIMULATED_PAID_NOT_ON_CHAIN",
        "unrecognized receipt"
    );
    journal.record_simulated_receipt(&ih, &r.digest)?;
    eprintln!("{}", serde_json::to_string_pretty(&r)?);
    Ok(())
}

fn prepare(
    invoice: PathBuf,
    merchant_key: String,
    chain_id: String,
    seed_file: PathBuf,
    epoch: u64,
    out: PathBuf,
) -> Result<()> {
    ensure!(
        std::fs::metadata(&seed_file)?.permissions().mode() & 0o077 == 0,
        "development seed file must be mode 0600"
    );
    let signed: SignedInvoice = read_json(&invoice)?;
    let invoice = signed.verify(&pin(&merchant_key)?)?;
    let chain: Digest = chain_id.parse()?;
    let chain_bytes: [u8; 32] = chain.as_bytes().try_into()?;
    invoice.validate_checkout(now()?, &chain_bytes, &[USDC])?;
    let mut seed = read_bounded(&seed_file, 32)?;
    ensure!(
        seed.len() == 32,
        "seed file must contain exactly 32 raw bytes"
    );
    let key = Ed25519PrivateKey::new(seed.as_slice().try_into()?);
    seed.fill(0);
    eprintln!(
        "OFFLINE DEVELOPMENT APPROVAL REQUIRED: asset={} amount_base_units={} recipient={} epoch={}",
        invoice.coin_type,
        invoice.amount,
        hex::encode(invoice.receiver),
        epoch
    );
    eprintln!("Type APPROVE to sign these exact terms. No transaction will be broadcast.");
    let mut confirmation = String::new();
    std::io::stdin().read_line(&mut confirmation)?;
    ensure!(confirmation.trim() == "APPROVE", "not approved");
    let tx = tap_sui::build_direct(
        &invoice,
        key.public_key().derive_address(),
        epoch,
        rand::random(),
    )?;
    let p = DevelopmentSigner(key).sign_payment(&invoice, &tx)?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(out)?;
    serde_json::to_writer_pretty(&file, &p)?;
    file.sync_all()?;
    println!("PREPARED_ONLY digest={} eligibility=UNVERIFIED", p.digest);
    Ok(())
}

#[cfg(feature = "pcsc-reader")]
fn readers() -> Result<()> {
    let context = pcsc::Context::establish(pcsc::Scope::User)?;
    for r in context.list_readers_owned()? {
        println!("{}", r.to_string_lossy());
    }
    Ok(())
}
#[cfg(not(feature = "pcsc-reader"))]
fn readers() -> Result<()> {
    anyhow::bail!("rebuild with --features pcsc-reader; PC/SC service and reader are required")
}
#[cfg(feature = "pcsc-reader")]
fn nfc_push(input: PathBuf, reader: String) -> Result<()> {
    let message = read_bounded(&input, MAX_ENVELOPE)?;
    let _: SignedInvoice = serde_json::from_slice(&message)?;
    let context = pcsc::Context::establish(pcsc::Scope::User)?;
    let names = context.list_readers_owned()?;
    let name = names
        .iter()
        .find(|r| r.to_string_lossy() == reader)
        .context("reader name must match exactly")?;
    let card = context.connect(name, pcsc::ShareMode::Shared, pcsc::Protocols::T1)?;
    for frame in apdu::frames(&message)? {
        let mut buffer = [0u8; 4096];
        let response = card.transmit(&frame, &mut buffer)?;
        ensure!(
            response.ends_with(&[0x90, 0]),
            "card rejected APDU: {}",
            hex::encode(response)
        );
    }
    println!("INVOICE_TRANSPORT_ACK_ONLY not_payment=true");
    Ok(())
}
#[cfg(not(feature = "pcsc-reader"))]
fn nfc_push(_: PathBuf, _: String) -> Result<()> {
    anyhow::bail!("rebuild with --features pcsc-reader")
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::ServeDemo {
            socket,
            stdio,
            db,
            connections,
        } => serve(socket, stdio, db, connections).await,
        Command::PayDemo {
            socket,
            stdio,
            journal,
            approve_simulation,
            interactive_simulation,
            disconnect_after_journal,
        } => {
            pay(
                socket,
                stdio,
                journal,
                approve_simulation,
                interactive_simulation,
                disconnect_after_journal,
            )
            .await
        }
        Command::Probe { endpoint } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&tap_sui::probe(&endpoint).await?)?
            );
            Ok(())
        }
        Command::QualifyTestnet {
            endpoint,
            chain_id,
            sender,
            receiver,
            amount,
        } => {
            let report = tap_sui::qualification::qualify_testnet(
                &endpoint, chain_id, sender, receiver, amount,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            ensure!(
                report.qualified,
                "TESTNET NOT QUALIFIED; submission work blocked"
            );
            Ok(())
        }
        Command::VerifyOnNode {
            endpoint,
            invoice,
            merchant_key,
            prepared,
        } => {
            let s: SignedInvoice = read_json(&invoice)?;
            let i = s.verify(&pin(&merchant_key)?)?;
            let p = read_json(&prepared)?;
            let checkpoint = tap_sui::verify_on_node(&endpoint, &i, &p).await?;
            println!("VERIFIED_ON_EXPLICITLY_TRUSTED_NODE checkpoint={checkpoint}");
            Ok(())
        }
        Command::PrepareDraft {
            invoice,
            merchant_key,
            chain_id,
            seed_file,
            epoch,
            out,
        } => prepare(invoice, merchant_key, chain_id, seed_file, epoch, out),
        Command::ApduRoundtrip { input } => {
            let m = read_bounded(&input, MAX_ENVELOPE)?;
            let mut r = apdu::Receiver::default();
            let frames = apdu::frames(&m)?;
            for a in &frames {
                ensure!(r.process(a).ends_with(&[0x90, 0]), "APDU rejected");
            }
            ensure!(r.message() == Some(m.as_slice()), "reconstruction failed");
            println!(
                "APDU_SIMULATOR_OK frames={} no_hardware_test=true",
                frames.len()
            );
            Ok(())
        }
        Command::ApduStdio { out } => apdu_stdio(out),
        Command::Readers => readers(),
        Command::NfcPush { input, reader } => nfc_push(input, reader),
    }
}

fn apdu_stdio(out: PathBuf) -> Result<()> {
    use std::io::{BufRead, Write};
    let mut receiver = apdu::Receiver::default();
    let mut input = std::io::stdin().lock();
    loop {
        let mut line = String::new();
        let n = (&mut input).take(1025).read_line(&mut line)?;
        if n == 0 {
            break;
        }
        ensure!(
            n <= 1024 && line.ends_with('\n'),
            "oversize or unterminated APDU line"
        );
        if line.trim() == "RESET" {
            receiver = apdu::Receiver::default();
            println!("9000");
            std::io::stdout().flush()?;
            continue;
        }
        let bytes = hex::decode(line.trim())?;
        let mut response = receiver.process(&bytes);
        if let Some(message) = receiver.message() {
            let persist = (|| -> Result<()> {
                ensure!(message.len() <= MAX_ENVELOPE, "envelope too large");
                let _: SignedInvoice = serde_json::from_slice(message)?;
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&out)
                {
                    Ok(mut f) => {
                        f.write_all(message)?;
                        f.sync_all()?;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => ensure!(
                        read_bounded(&out, MAX_ENVELOPE)? == message,
                        "output already contains another invoice"
                    ),
                    Err(e) => return Err(e.into()),
                }
                Ok(())
            })();
            if let Err(error) = persist {
                eprintln!("INVOICE_STORE_FAILED: {error}");
                response = vec![0x65, 0x81];
            } else {
                eprintln!(
                    "UNVERIFIED_INVOICE_STORED: review merchant trust, chain and terms before signing"
                );
            }
        }
        println!("{}", hex::encode(response));
        std::io::stdout().flush()?;
    }
    Ok(())
}
