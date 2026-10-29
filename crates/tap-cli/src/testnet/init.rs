//! Human-approved disposable setup. No defaults, key exports, RPC, signing or funding.
use anyhow::{Context, Result, ensure};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::{
    ffi::{c_int, c_uint, c_ulong},
    fs::{File, OpenOptions},
    io::{BufRead, IsTerminal, Read, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
};
use sui_crypto::ed25519::Ed25519PrivateKey;
use sui_sdk_types::Address;
use tap_core::{CIRCLE_TESTNET_USDC, SUI_TESTNET_CHAIN};

// Linux UAPI prctl.h and stdio.h: process-local dump protection and atomic no-replace publication.
const PR_GET_DUMPABLE: c_int = 3;
const PR_SET_DUMPABLE: c_int = 4;
const RENAME_NOREPLACE: c_uint = 1;
unsafe extern "C" {
    fn prctl(option: c_int, ...) -> c_int;
    fn renameat2(
        oldfd: c_int,
        old: *const std::ffi::c_char,
        newfd: c_int,
        new: *const std::ffi::c_char,
        flags: c_uint,
    ) -> c_int;
}

#[derive(Args)]
pub struct InitArgs {
    /// New absolute directory beneath an existing owned canonical mode-0700 parent.
    /// Must not exist. No default/home wallet is searched or modified.
    #[arg(long)]
    directory: PathBuf,
}

pub(super) const SETUP_PROFILE: &str = "circle-sui-testnet/disposable-software-setup/v1";
pub(super) const SETUP_STATUS: &str = "PROVISIONED_ONLY_NOT_FUNDED_OR_ELIGIBLE";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PublicWallet {
    pub(super) address: Address,
    pub(super) public_key_hex: String,
    pub(super) keystore: PathBuf,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    pub(super) profile: String,
    pub(super) status: String,
    pub(super) directory: PathBuf,
    pub(super) chain_id: String,
    pub(super) coin_type: String,
    pub(super) amount_base_units: u64,
    pub(super) chain_submission: bool,
    pub(super) production_custody: bool,
    pub(super) sender: PublicWallet,
    pub(super) merchant_invoice: PublicWallet,
    pub(super) receiver: PublicWallet,
}

// Retain private files on failure, but withdraw only our own published public manifest.
struct SetupGuard {
    directory: PathBuf,
    published: Option<(u64, u64)>,
    complete: bool,
}
impl Drop for SetupGuard {
    fn drop(&mut self) {
        if !self.complete {
            if let Some(identity) = self.published {
                let ready = self.directory.join("READY.json");
                let ours = std::fs::symlink_metadata(&ready)
                    .is_ok_and(|m| m.is_file() && (m.dev(), m.ino()) == identity);
                if !ours || std::fs::remove_file(&ready).is_err() {
                    eprintln!(
                        "TESTNET_SETUP_STOPPED: could not withdraw our READY.json; DO NOT USE this directory"
                    );
                }
            }
            if let Ok(directory) = File::open(&self.directory) {
                let _ = directory.sync_all();
            }
            eprintln!(
                "TESTNET_SETUP_STOPPED: retain partial private files; do not fund/use this directory or retry over it"
            );
        }
    }
}

pub(super) fn run(args: InitArgs) -> Result<()> {
    provision(args).context(
        "TESTNET_SETUP_STOPPED: no completed setup; no existing data is overwritten or deleted",
    )
}

pub(super) fn disable_dumpability() -> Result<()> {
    // SAFETY: Linux prctl uses integer variadic arguments; no pointers or private buffers
    // are passed. This process-local protection is checked before entropy or secret reads.
    let protected = unsafe {
        prctl(
            PR_SET_DUMPABLE,
            0 as c_ulong,
            0 as c_ulong,
            0 as c_ulong,
            0 as c_ulong,
        ) == 0
            && prctl(
                PR_GET_DUMPABLE,
                0 as c_ulong,
                0 as c_ulong,
                0 as c_ulong,
                0 as c_ulong,
            ) == 0
    };
    ensure!(protected, "cannot disable process dumpability");
    Ok(())
}

fn provision(args: InitArgs) -> Result<()> {
    let directory = args.directory;
    ensure!(
        !directory
            .to_str()
            .context("UTF-8 destination required for public manifest")?
            .chars()
            .any(char::is_control),
        "destination control characters are not supported"
    );
    ensure!(
        directory.file_name().is_some()
            && directory.components().all(|c| matches!(
                c,
                std::path::Component::RootDir | std::path::Component::Normal(_)
            )),
        "destination must be a new absolute child directory, not a root or traversal path"
    );
    super::private_parent(&directory)?;
    match std::fs::symlink_metadata(&directory) {
        Ok(_) => anyhow::bail!(
            "destination already exists; files, directories and links are never replaced"
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let owner = std::fs::metadata(directory.parent().context("missing parent")?)?.uid();
    ensure!(
        std::io::stdin().is_terminal(),
        "disposable setup requires a human terminal; no unattended approval flag exists"
    );
    eprintln!(
        "DISPOSABLE SUI TESTNET ONLY / SOFTWARE CUSTODY, NOT PRODUCTION\nNew directory: {}\nCreates sender, merchant invoice signer and receiving keystores.\nUnencrypted private keys stay in owner-only files. No keys are printed.\nNo RPC, faucet, funding or transaction signing occurs. No guaranteed memory erasure or backup.\nRun privately, without recording/debuggers. Never fund public fixtures or use production funds.",
        directory.display()
    );
    eprintln!("Type CREATE DISPOSABLE TESTNET WALLETS to create fresh private keys:");
    let mut confirmation = String::new();
    std::io::stdin()
        .lock()
        .take(128)
        .read_line(&mut confirmation)?;
    ensure!(
        confirmation.ends_with('\n') && confirmation.trim() == "CREATE DISPOSABLE TESTNET WALLETS",
        "not approved; no directory or keys created"
    );

    disable_dumpability().context("dump protection failed; no keys generated")?;

    std::fs::DirBuilder::new().mode(0o700).create(&directory)?;
    let mut guard = SetupGuard {
        directory: directory.clone(),
        published: None,
        complete: false,
    };
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    let dir = File::open(&directory)?;
    let metadata = dir.metadata()?;
    ensure!(
        metadata.is_dir() && metadata.uid() == owner && metadata.mode() & 0o777 == 0o700,
        "new destination ownership/mode mismatch"
    );
    // Make the new directory durable before generating any key, not a power-loss qualification.
    File::open(directory.parent().context("missing parent")?)?.sync_all()?;

    let manifest = Manifest {
        profile: SETUP_PROFILE.into(),
        status: SETUP_STATUS.into(),
        directory: directory.clone(),
        chain_id: SUI_TESTNET_CHAIN.into(),
        coin_type: CIRCLE_TESTNET_USDC.into(),
        amount_base_units: 10_000,
        chain_submission: false,
        production_custody: false,
        sender: create_wallet(&directory.join("sender.keystore"))?,
        merchant_invoice: create_wallet(&directory.join("merchant.keystore"))?,
        receiver: create_wallet(&directory.join("receiver.keystore"))?,
    };
    ensure!(
        manifest.sender.address != manifest.receiver.address
            && manifest.sender.address != manifest.merchant_invoice.address
            && manifest.receiver.address != manifest.merchant_invoice.address,
        "unexpected duplicate public address; setup stopped"
    );
    // Every private file is synced before any public READY manifest can appear.
    dir.sync_all()?;
    let pending = private_new(&directory.join(".READY.json.pending"))?;
    serde_json::to_writer_pretty(&pending, &manifest)?;
    pending.sync_all()?;
    dir.sync_all()?;
    let ready_metadata = pending.metadata()?;
    // SAFETY: both FDs borrow the live directory File; C string literals are valid,
    // NUL-terminated fixed relative names. Linux RENAME_NOREPLACE never overwrites a target.
    let published = unsafe {
        renameat2(
            dir.as_raw_fd(),
            c".READY.json.pending".as_ptr(),
            dir.as_raw_fd(),
            c"READY.json".as_ptr(),
            RENAME_NOREPLACE,
        )
    };
    ensure!(
        published == 0,
        "cannot atomically publish READY.json without replacement: {}",
        std::io::Error::last_os_error()
    );
    guard.published = Some((ready_metadata.dev(), ready_metadata.ino()));
    dir.sync_all()?;
    let mut output = std::io::stdout().lock();
    writeln!(
        output,
        "DISPOSABLE_TESTNET_SETUP_READY manifest={}",
        directory.join("READY.json").display()
    )?;
    writeln!(
        output,
        "PUBLIC sender_faucet_address={}\nPUBLIC merchant_invoice_address={}\nPUBLIC merchant_key_pin={}\nPUBLIC fresh_receiver={}",
        manifest.sender.address,
        manifest.merchant_invoice.address,
        manifest.merchant_invoice.public_key_hex,
        manifest.receiver.address
    )?;
    writeln!(
        output,
        "No RPC/funding/eligibility/settlement performed. Keep receiver.keystore private and retained; observers do not need it."
    )?;
    output.flush()?;
    guard.complete = true;
    Ok(())
}

fn private_new(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file()
            && metadata.nlink() == 1
            && metadata.mode() & 0o777 == 0o600
            && metadata.uid()
                == std::fs::metadata(path.parent().context("missing private parent")?)?.uid(),
        "private output must be a new owned mode-0600 regular file"
    );
    Ok(file)
}

fn create_wallet(path: &Path) -> Result<PublicWallet> {
    // rand 0.8 OsRng and pinned SDK rand_core 0.6 are compatible. Never accept seeds,
    // RNG overrides or fixture switches. SDK generate obtains secret bytes from OS entropy.
    let key = Ed25519PrivateKey::generate(rand::rngs::OsRng);
    let file = private_new(path)?;
    // Official legacy Sui CLI flag||private_key encoding only; no handwritten seed format.
    serde_json::to_writer(&file, &[key.to_base64()])?;
    file.sync_all()?;
    // Minimize lifetime/copies, but JSON strings and SDK temporaries are NOT guaranteed erased.
    Ok(PublicWallet {
        address: key.public_key().derive_address(),
        public_key_hex: hex::encode(key.public_key().as_bytes()),
        keystore: path.into(),
    })
}
