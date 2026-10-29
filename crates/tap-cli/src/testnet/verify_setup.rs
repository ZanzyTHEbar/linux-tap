//! Human-private, read-only key matching. No entropy, writes, signing, RPC or funding.
use super::init::{Manifest, SETUP_PROFILE, SETUP_STATUS, disable_dumpability};
use anyhow::{Context, Result, ensure};
use clap::Args;
use std::{io::Read, path::PathBuf};
use sui_crypto::ed25519::Ed25519PrivateKey;
use tap_core::{CIRCLE_TESTNET_USDC, SUI_TESTNET_CHAIN};

#[derive(Args)]
pub struct VerifySetupArgs {
    /// Existing setup directory. Run privately; only public verification results are printed.
    #[arg(long)]
    directory: PathBuf,
}

pub(super) fn run(args: VerifySetupArgs) -> Result<()> {
    verify(args)
        .context("SETUP_KEY_MATCH_STOPPED: no files changed; do not fund an unverified setup")
}

fn verify(args: VerifySetupArgs) -> Result<()> {
    let directory = args.directory;
    ensure!(
        directory.is_absolute()
            && directory.file_name().is_some()
            && directory.components().all(|c| matches!(
                c,
                std::path::Component::RootDir | std::path::Component::Normal(_)
            )),
        "explicit canonical absolute setup directory required"
    );
    // Open only fixed names, checking owner/mode/regularity/link count and canonical parent.
    // Do this before prctl: Linux may change /proc/self ownership when dumpability is disabled.
    // These opens inspect metadata only; no manifest or private bytes are read until protected.
    let manifest_file = super::private_file(&directory.join("READY.json"))?;
    let roles = [
        (
            "sender.keystore",
            super::private_file(&directory.join("sender.keystore"))?,
        ),
        (
            "merchant.keystore",
            super::private_file(&directory.join("merchant.keystore"))?,
        ),
        (
            "receiver.keystore",
            super::private_file(&directory.join("receiver.keystore"))?,
        ),
    ];
    disable_dumpability().context("no private key material read")?;
    let mut bytes = Vec::new();
    manifest_file.take(65_537).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 65_536, "public manifest exceeds limit");
    let manifest: Manifest = serde_json::from_slice(&bytes)
        .map_err(|_| anyhow::anyhow!("invalid public setup manifest"))?;
    ensure!(
        manifest.profile == SETUP_PROFILE
            && manifest.status == SETUP_STATUS
            && manifest.directory == directory
            && manifest.chain_id == SUI_TESTNET_CHAIN
            && manifest.coin_type == CIRCLE_TESTNET_USDC
            && manifest.amount_base_units == 10_000
            && !manifest.chain_submission
            && !manifest.production_custody,
        "setup profile/status/testnet policy mismatch"
    );
    let wallets = [
        &manifest.sender,
        &manifest.merchant_invoice,
        &manifest.receiver,
    ];
    ensure!(
        wallets[0].address != wallets[1].address
            && wallets[0].address != wallets[2].address
            && wallets[1].address != wallets[2].address,
        "setup addresses must be distinct"
    );
    for (wallet, (name, _)) in wallets.iter().zip(&roles) {
        ensure!(
            wallet.keystore == directory.join(name),
            "manifest keystore path must match its fixed setup role"
        );
        // Known project and laboratory fixtures are public, never funding/custody candidates.
        // This cannot detect every disclosed key; it is not proof of secret freshness.
        for seed in [7, 9, 11, 12] {
            ensure!(
                wallet.address
                    != Ed25519PrivateKey::new([seed; 32])
                        .public_key()
                        .derive_address(),
                "known public fixture address is not a disposable private wallet"
            );
        }
    }
    for (wallet, (_, file)) in wallets.iter().zip(roles) {
        // Exactly one official legacy Ed25519 entry per role, using the existing sanitized loader.
        let key = super::key_from_file(file, wallet.address, 1)?;
        ensure!(
            hex::encode(key.public_key().as_bytes()) == wallet.public_key_hex,
            "setup public key does not match READY"
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "verified": true, "profile": SETUP_PROFILE,
            "status": "KEY_MATCHES_ONLY_NOT_FUNDED_OR_ELIGIBLE",
            "chain_submission": false, "production_custody": false,
            "sender": manifest.sender.address,
            "merchant_invoice": manifest.merchant_invoice.address,
            "merchant_key_pin": manifest.merchant_invoice.public_key_hex,
            "receiver": manifest.receiver.address,
        }))?
    );
    Ok(())
}
