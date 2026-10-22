//! Native Mysten SDK adapter. Construction is not proof of network eligibility.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sui_crypto::{
    SuiSigner, SuiVerifier,
    ed25519::{Ed25519PrivateKey, Ed25519Verifier},
};
use sui_rpc::{
    Client,
    field::{FieldMask, FieldMaskUtil},
    proto::sui::rpc::v2::{ExecutedTransaction, GetServiceInfoRequest, GetTransactionRequest},
};
use sui_sdk_types::{
    Address, Argument, Command, Digest, FundsWithdrawal, GasPayment, Identifier, Input, MoveCall,
    ProgrammableTransaction, Transaction, TransactionExpiration, TransactionKind, TypeTag,
    UserSignature, WithdrawFrom,
};
use tap_core::{DirectAssetPolicy, Invoice};

pub mod qualification;

/// Experimental address-balance template using only official SDK types and BCS.
/// The pinned offline builder rejects empty gas objects; see ADR 0002.
pub fn build_direct(
    invoice: &Invoice,
    sender: Address,
    epoch: u64,
    nonce: u32,
) -> Result<Transaction> {
    build_direct_with_policy(invoice, sender, epoch, nonce, DirectAssetPolicy::PinnedUsdc)
}

// The explicit alternate profile is qualification-only; signing keeps its default policy.
pub(crate) fn build_direct_with_policy(
    invoice: &Invoice,
    sender: Address,
    epoch: u64,
    nonce: u32,
    policy: DirectAssetPolicy,
) -> Result<Transaction> {
    policy.validate(invoice)?;
    ensure!(
        invoice.amount >= 10_000,
        "USDC amount below documented 0.01 minimum"
    );
    let ty: TypeTag = invoice.coin_type.parse()?;
    let call = |name: &str, args: Vec<Argument>| -> Result<Command> {
        Ok(Command::MoveCall(MoveCall {
            package: Address::from_static("0x2"),
            module: Identifier::new("balance")?,
            function: Identifier::new(name)?,
            type_arguments: vec![ty.clone()],
            arguments: args,
        }))
    };
    Ok(Transaction {
        sender,
        gas_payment: GasPayment {
            objects: vec![],
            owner: sender,
            price: 0,
            budget: 0,
        },
        expiration: TransactionExpiration::ValidDuring {
            min_epoch: Some(epoch),
            max_epoch: Some(epoch),
            min_timestamp: None,
            max_timestamp: None,
            chain: Digest::new(invoice.chain),
            nonce,
        },
        kind: TransactionKind::ProgrammableTransaction(ProgrammableTransaction {
            inputs: vec![
                Input::FundsWithdrawal(FundsWithdrawal::new(
                    invoice.amount,
                    ty.clone(),
                    WithdrawFrom::Sender,
                )),
                Input::Pure(bcs::to_bytes(&Address::new(invoice.receiver))?),
            ],
            commands: vec![
                call("redeem_funds", vec![Argument::Input(0)])?,
                call("send_funds", vec![Argument::Result(0), Argument::Input(1)])?,
            ],
        }),
    })
}

/// Rebuild the only allowed template. Rejects extra commands, other recipients and gas changes.
pub fn verify_template(invoice: &Invoice, tx: &Transaction) -> Result<()> {
    verify_template_with_policy(invoice, tx, DirectAssetPolicy::PinnedUsdc)
}

pub(crate) fn verify_template_with_policy(
    invoice: &Invoice,
    tx: &Transaction,
    policy: DirectAssetPolicy,
) -> Result<()> {
    let (epoch, nonce) = match tx.expiration {
        TransactionExpiration::ValidDuring {
            min_epoch: Some(min),
            max_epoch: Some(max),
            nonce,
            ..
        } if min == max => (min, nonce),
        _ => anyhow::bail!("unsupported transaction expiration"),
    };
    ensure!(
        *tx == build_direct_with_policy(invoice, tx.sender, epoch, nonce, policy)?,
        "transaction does not match invoice template"
    );
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prepared {
    pub digest: String,
    pub bcs_hex: String,
    pub signature: UserSignature,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ApprovalProfile {
    DevelopmentSoftware,
    HardwareKeyWithHostApproval,
    TrustedDeviceApproval,
}

/// Provider boundary for future hardware adapters. User approval is a caller responsibility.
/// Implementations must verify the template before asking hardware to sign, and must
/// report the actual approval profile rather than equating key storage with trusted UI.
pub trait PaymentSigner {
    fn address(&self) -> Address;
    fn approval_profile(&self) -> ApprovalProfile;
    fn sign_payment(&self, invoice: &Invoice, transaction: &Transaction) -> Result<Prepared>;
}

pub struct DevelopmentSigner(pub Ed25519PrivateKey);
impl PaymentSigner for DevelopmentSigner {
    fn address(&self) -> Address {
        self.0.public_key().derive_address()
    }
    fn approval_profile(&self) -> ApprovalProfile {
        ApprovalProfile::DevelopmentSoftware
    }
    fn sign_payment(&self, invoice: &Invoice, transaction: &Transaction) -> Result<Prepared> {
        verify_template(invoice, transaction)?;
        Prepared::sign(transaction, &self.0)
    }
}
impl Prepared {
    pub fn sign(tx: &Transaction, key: &Ed25519PrivateKey) -> Result<Self> {
        ensure!(
            tx.sender == key.public_key().derive_address(),
            "sender does not match signer"
        );
        Ok(Self {
            digest: tx.digest().to_string(),
            bcs_hex: hex::encode(bcs::to_bytes(tx)?),
            signature: key.sign_transaction(tx)?,
        })
    }
    pub fn transaction(&self) -> Result<Transaction> {
        ensure!(self.bcs_hex.len() <= 32_768, "transaction too large");
        let tx: Transaction = bcs::from_bytes(&hex::decode(&self.bcs_hex)?)?;
        ensure!(tx.digest().to_string() == self.digest, "digest mismatch");
        Ed25519Verifier::new().verify_transaction(&tx, &self.signature)?;
        match &self.signature {
            UserSignature::Simple(sui_sdk_types::SimpleSignature::Ed25519 {
                public_key, ..
            }) => ensure!(
                public_key.derive_address() == tx.sender,
                "signature sender mismatch"
            ),
            _ => anyhow::bail!("unsupported signer"),
        };
        Ok(tx)
    }
}

fn client(endpoint: &str) -> Result<Client> {
    // HTTPS is required except for explicit local development nodes.
    ensure!(
        endpoint.starts_with("https://")
            || endpoint.starts_with("http://127.0.0.1:")
            || endpoint.starts_with("http://localhost:"),
        "endpoint must use TLS or loopback"
    );
    Ok(Client::new(endpoint)?.with_response_headers_timeout(Duration::from_secs(10)))
}

#[derive(Debug, Serialize)]
pub struct NodeInfo {
    pub chain_id: String,
    pub chain: Option<String>,
    pub epoch: u64,
    pub checkpoint: Option<u64>,
}
pub async fn probe(endpoint: &str) -> Result<NodeInfo> {
    let mut c = client(endpoint)?;
    let r = tokio::time::timeout(
        Duration::from_secs(15),
        c.ledger_client()
            .get_service_info(GetServiceInfoRequest::default()),
    )
    .await??
    .into_inner();
    Ok(NodeInfo {
        chain_id: r.chain_id.context("node omitted chain identity")?,
        chain: r.chain,
        epoch: r.epoch.context("node omitted epoch")?,
        checkpoint: r.checkpoint_height,
    })
}

/// Explicitly trusts this node for checkpoint inclusion. No foundation RPC is required.
pub async fn verify_on_node(endpoint: &str, invoice: &Invoice, prepared: &Prepared) -> Result<u64> {
    let tx = prepared.transaction()?;
    verify_template(invoice, &tx)?;
    let info = probe(endpoint).await?;
    ensure!(
        info.chain_id.parse::<Digest>()? == Digest::new(invoice.chain),
        "node is on wrong chain"
    );
    let mut req = GetTransactionRequest::default();
    req.digest = Some(prepared.digest.clone());
    req.read_mask = Some(FieldMask::from_paths(["*"]));
    let mut c = client(endpoint)?;
    let r = tokio::time::timeout(
        Duration::from_secs(15),
        c.ledger_client().get_transaction(req),
    )
    .await??
    .into_inner();
    verify_execution(
        invoice,
        prepared,
        &r.transaction.context("missing execution")?,
    )
}

/// Pure receipt predicate, tested with fixtures. Finality evidence is trusted to the chosen node.
pub fn verify_execution(
    invoice: &Invoice,
    prepared: &Prepared,
    executed: &ExecutedTransaction,
) -> Result<u64> {
    let tx = prepared.transaction()?;
    verify_template(invoice, &tx)?;
    ensure!(
        executed.digest.as_deref() == Some(&prepared.digest),
        "execution digest mismatch"
    );
    let checkpoint = executed.checkpoint.context("not checkpointed")?;
    let effects = executed.effects.as_ref().context("missing effects")?;
    ensure!(
        effects.transaction_digest.as_deref() == Some(&prepared.digest),
        "effects digest mismatch"
    );
    ensure!(
        effects.status.as_ref().and_then(|s| s.success) == Some(true),
        "execution failed or unknown"
    );
    let observed: Transaction = executed
        .transaction
        .as_ref()
        .context("missing transaction")?
        .try_into()?;
    ensure!(observed == tx, "node returned different transaction");
    Ok(checkpoint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tap_core::USDC;
    fn inv() -> Invoice {
        Invoice {
            chain: [42; 32],
            id: [1; 16],
            receiver: [3; 32],
            coin_type: USDC.into(),
            amount: 10000,
            created_ms: 1000,
            expires_ms: 2000,
            merchant: "Lab".into(),
        }
    }
    #[test]
    fn build_sign_verify_roundtrip() {
        let k = Ed25519PrivateKey::new([9; 32]);
        let i = inv();
        let t = build_direct(&i, k.public_key().derive_address(), 42, 1).unwrap();
        let p = Prepared::sign(&t, &k).unwrap();
        assert_eq!(p.transaction().unwrap(), t);
        verify_template(&i, &t).unwrap();
    }
    #[test]
    fn reject_changes() {
        let k = Ed25519PrivateKey::new([9; 32]);
        let i = inv();
        let mut t = build_direct(&i, k.public_key().derive_address(), 42, 1).unwrap();
        t.gas_payment.budget = 1;
        assert!(verify_template(&i, &t).is_err());
        let mut wrong = i;
        wrong.receiver = [4; 32];
        assert!(verify_template(&wrong, &t).is_err());
    }
    #[test]
    fn reject_signature_tamper() {
        let k = Ed25519PrivateKey::new([9; 32]);
        let mut p = Prepared::sign(
            &build_direct(&inv(), k.public_key().derive_address(), 42, 1).unwrap(),
            &k,
        )
        .unwrap();
        p.bcs_hex.replace_range(0..2, "ff");
        assert!(p.transaction().is_err());
    }
    #[test]
    fn capture_offline_builder_gap() {
        let mut b = sui_transaction_builder::TransactionBuilder::new();
        b.set_sender(Address::ZERO);
        b.set_gas_price(0);
        b.set_gas_budget(0);
        assert!(b.try_build().is_err());
    }

    #[test]
    fn receipt_rejections() {
        use sui_rpc::proto::sui::rpc::v2::{ExecutionStatus, TransactionEffects};
        let i = inv();
        let k = Ed25519PrivateKey::new([9; 32]);
        let t = build_direct(&i, k.public_key().derive_address(), 42, 1).unwrap();
        let p = Prepared::sign(&t, &k).unwrap();
        let mut e = ExecutedTransaction::default();
        e.digest = Some(p.digest.clone());
        e.transaction = Some(t.clone().into());
        e.checkpoint = Some(10);
        let mut effects = TransactionEffects::default();
        effects.transaction_digest = Some(p.digest.clone());
        let mut status = ExecutionStatus::default();
        status.success = Some(true);
        effects.status = Some(status);
        e.effects = Some(effects);
        assert_eq!(verify_execution(&i, &p, &e).unwrap(), 10);
        let mut bad = e.clone();
        bad.checkpoint = None;
        assert!(verify_execution(&i, &p, &bad).is_err());
        let mut bad = e.clone();
        bad.effects
            .as_mut()
            .unwrap()
            .status
            .as_mut()
            .unwrap()
            .success = Some(false);
        assert!(verify_execution(&i, &p, &bad).is_err());
        let mut bad = e.clone();
        bad.effects.as_mut().unwrap().transaction_digest = Some("other".into());
        assert!(verify_execution(&i, &p, &bad).is_err());
        let mut bad = e;
        let mut wrong = t;
        wrong.sender = Address::ZERO;
        bad.transaction = Some(wrong.into());
        assert!(verify_execution(&i, &p, &bad).is_err());
    }

    #[test]
    fn signer_checks_the_invoice() {
        let signer = DevelopmentSigner(Ed25519PrivateKey::new([9; 32]));
        let i = inv();
        let t = build_direct(&i, signer.address(), 42, 1).unwrap();
        assert!(signer.sign_payment(&i, &t).is_ok());
        let mut wrong = i;
        wrong.amount += 1;
        assert!(signer.sign_payment(&wrong, &t).is_err());
    }
}
