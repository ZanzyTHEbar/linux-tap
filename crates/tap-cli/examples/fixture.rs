//! Public, expired interoperability fixture. Never fund the test keys.
fn main() -> anyhow::Result<()> {
    let invoice = tap_core::Invoice {
        chain: [42; 32],
        id: [1; 16],
        receiver: [3; 32],
        coin_type: tap_core::USDC.into(),
        amount: 10000,
        created_ms: 1000,
        expires_ms: 2000,
        merchant: "PUBLIC EXPIRED TEST FIXTURE".into(),
    };
    let signed = tap_core::SignedInvoice::sign(
        &invoice,
        &sui_crypto::ed25519::Ed25519PrivateKey::new([7; 32]),
    )?;
    println!("{}", serde_json::to_string_pretty(&signed)?);
    Ok(())
}
