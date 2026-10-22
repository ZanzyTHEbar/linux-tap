//! Signed experimental invoices. See docs/protocol.md before integration.
pub mod apdu;

use anyhow::{Result, bail, ensure};
use blake2::{Blake2b, Digest as _, digest::consts::U32};
use minicbor::{Decoder, Encoder};
use serde::{Deserialize, Serialize};
use sui_crypto::{
    Signer, Verifier,
    ed25519::{Ed25519PrivateKey, Ed25519VerifyingKey},
};
use sui_sdk_types::{Ed25519PublicKey, Ed25519Signature, TypeTag};

pub const USDC: &str =
    "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC";
pub const MAX_CORE: usize = 2048;
pub const MAX_ENVELOPE: usize = 8192;
const DOMAIN: &[u8] = b"linux-sui-tap/invoice/v0\0";

pub fn hash(bytes: &[u8]) -> [u8; 32] {
    Blake2b::<U32>::digest(bytes).into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invoice {
    pub chain: [u8; 32],
    pub id: [u8; 16],
    pub receiver: [u8; 32],
    pub coin_type: String,
    pub amount: u64,
    pub created_ms: u64,
    pub expires_ms: u64,
    pub merchant: String,
}

impl Invoice {
    fn structural_check(&self) -> Result<()> {
        ensure!(self.amount > 0, "zero amount");
        ensure!(self.coin_type.len() <= 512, "coin type too long");
        self.coin_type.parse::<TypeTag>()?;
        ensure!(
            !self.merchant.is_empty() && self.merchant.len() <= 128,
            "invalid merchant label"
        );
        ensure!(
            !self.merchant.chars().any(char::is_control),
            "control character in merchant label"
        );
        let ttl = self
            .expires_ms
            .checked_sub(self.created_ms)
            .ok_or_else(|| anyhow::anyhow!("invalid expiry"))?;
        ensure!(
            ttl > 0 && ttl <= 900_000,
            "invoice lifetime exceeds 15 minutes"
        );
        ensure!(self.receiver != [0; 32], "zero recipient");
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        self.structural_check()?;
        let mut e = Encoder::new(Vec::new());
        e.map(10)?;
        e.u8(0)?.u8(0)?;
        e.u8(1)?.bytes(&self.chain)?;
        e.u8(2)?.bytes(&self.id)?;
        e.u8(3)?.bytes(&self.receiver)?;
        e.u8(4)?.str(&self.coin_type)?;
        e.u8(5)?.u64(self.amount)?;
        e.u8(6)?.u64(self.created_ms)?;
        e.u8(7)?.u64(self.expires_ms)?;
        e.u8(8)?.str(&self.merchant)?;
        e.u8(9)?.u8(0)?; // direct transfer profile only
        let bytes = e.into_writer();
        ensure!(bytes.len() <= MAX_CORE, "invoice too large");
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= MAX_CORE, "invoice too large");
        let mut d = Decoder::new(bytes);
        ensure!(d.map()? == Some(10), "expected definite ten-field map");
        fn key(d: &mut Decoder<'_>, expected: u8) -> Result<()> {
            ensure!(d.u8()? == expected, "duplicate, unknown or unordered field");
            Ok(())
        }
        key(&mut d, 0)?;
        ensure!(d.u8()? == 0, "unsupported draft version");
        key(&mut d, 1)?;
        let chain = d.bytes()?.try_into()?;
        key(&mut d, 2)?;
        let id = d.bytes()?.try_into()?;
        key(&mut d, 3)?;
        let receiver = d.bytes()?.try_into()?;
        key(&mut d, 4)?;
        let coin_type = d.str()?.to_owned();
        key(&mut d, 5)?;
        let amount = d.u64()?;
        key(&mut d, 6)?;
        let created_ms = d.u64()?;
        key(&mut d, 7)?;
        let expires_ms = d.u64()?;
        key(&mut d, 8)?;
        let merchant = d.str()?.to_owned();
        key(&mut d, 9)?;
        ensure!(d.u8()? == 0, "unsupported settlement profile");
        ensure!(d.position() == bytes.len(), "trailing bytes");
        let invoice = Self {
            chain,
            id,
            receiver,
            coin_type,
            amount,
            created_ms,
            expires_ms,
            merchant,
        };
        ensure!(invoice.encode()? == bytes, "noncanonical CBOR");
        Ok(invoice)
    }

    pub fn validate_checkout(
        &self,
        now_ms: u64,
        expected_chain: &[u8; 32],
        allowed_assets: &[&str],
    ) -> Result<()> {
        self.structural_check()?;
        ensure!(&self.chain == expected_chain, "wrong chain");
        ensure!(
            now_ms >= self.created_ms && now_ms < self.expires_ms,
            "invoice is future dated or expired"
        );
        ensure!(
            allowed_assets.contains(&self.coin_type.as_str()),
            "asset is not locally allowed"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedInvoice {
    pub core_hex: String,
    pub merchant_key_hex: String,
    pub signature_hex: String,
}

impl SignedInvoice {
    pub fn sign(invoice: &Invoice, key: &Ed25519PrivateKey) -> Result<Self> {
        let bytes = invoice.encode()?;
        let mut message = DOMAIN.to_vec();
        message.extend_from_slice(&bytes);
        let signature: Ed25519Signature = key.try_sign(&message)?;
        Ok(Self {
            core_hex: hex::encode(bytes),
            merchant_key_hex: hex::encode(key.public_key().as_bytes()),
            signature_hex: hex::encode(signature.as_bytes()),
        })
    }

    /// The key pin is supplied by local merchant trust policy, never by this envelope alone.
    pub fn verify(&self, expected_key: &[u8; 32]) -> Result<Invoice> {
        ensure!(self.core_hex.len() <= MAX_CORE * 2, "invoice too large");
        ensure!(
            self.merchant_key_hex.len() == 64 && self.signature_hex.len() == 128,
            "invalid signature lengths"
        );
        let public: [u8; 32] = hex::decode(&self.merchant_key_hex)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("key length"))?;
        ensure!(&public == expected_key, "merchant key is not trusted");
        let bytes = hex::decode(&self.core_hex)?;
        let sig: [u8; 64] = hex::decode(&self.signature_hex)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("signature length"))?;
        let mut message = DOMAIN.to_vec();
        message.extend_from_slice(&bytes);
        Ed25519VerifyingKey::new(&Ed25519PublicKey::new(public))?
            .verify(&message, &Ed25519Signature::new(sig))?;
        Invoice::decode(&bytes)
    }

    pub fn invoice_hash(&self) -> Result<[u8; 32]> {
        ensure!(self.core_hex.len() <= MAX_CORE * 2, "invoice too large");
        let mut bytes = DOMAIN.to_vec();
        bytes.extend(hex::decode(&self.core_hex)?);
        Ok(hash(&bytes))
    }
}

pub fn reject_mainnet_simulation(chain: &[u8; 32]) -> Result<()> {
    if chain != &[42; 32] {
        bail!("simulation accepts only its synthetic chain identity");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn invoice() -> Invoice {
        Invoice {
            chain: [42; 32],
            id: [1; 16],
            receiver: [3; 32],
            coin_type: USDC.into(),
            amount: 10000,
            created_ms: 1000,
            expires_ms: 2000,
            merchant: "Test merchant".into(),
        }
    }
    #[test]
    fn roundtrip_and_golden_prefix() {
        let i = invoice();
        let b = i.encode().unwrap();
        assert_eq!(b[0], 0xaa);
        assert_eq!(Invoice::decode(&b).unwrap(), i);
    }
    #[test]
    fn signature_and_pinning() {
        let k = Ed25519PrivateKey::new([7; 32]);
        let mut s = SignedInvoice::sign(&invoice(), &k).unwrap();
        let pin: [u8; 32] = k.public_key().as_bytes().try_into().unwrap();
        assert!(s.verify(&pin).is_ok());
        assert!(s.verify(&[0; 32]).is_err());
        s.core_hex.replace_range(10..12, "ff");
        assert!(s.verify(&pin).is_err());
    }
    #[test]
    fn policy_rejections() {
        let i = invoice();
        assert!(i.validate_checkout(1500, &[42; 32], &[USDC]).is_ok());
        assert!(i.validate_checkout(2000, &[42; 32], &[USDC]).is_err());
        assert!(i.validate_checkout(999, &[42; 32], &[USDC]).is_err());
        assert!(i.validate_checkout(1500, &[41; 32], &[USDC]).is_err());
        assert!(i.validate_checkout(1500, &[42; 32], &[]).is_err());
    }
    #[test]
    fn reject_noncanonical_and_trailing() {
        let b = invoice().encode().unwrap();
        let mut non = b.clone();
        non.splice(1..2, [0x18, 0x00]);
        assert!(Invoice::decode(&non).is_err());
        let mut trailing = b;
        trailing.push(0);
        assert!(Invoice::decode(&trailing).is_err());
        assert!(Invoice::decode(&vec![0; MAX_CORE + 1]).is_err());
    }
    #[test]
    fn reject_duplicate_fields_and_zero_amount() {
        let mut b = invoice().encode().unwrap();
        b[1] = 1;
        assert!(Invoice::decode(&b).is_err());
        let mut i = invoice();
        i.amount = 0;
        assert!(i.encode().is_err());
    }
}
