//! Exact-object qualification and observation of whole-coin conversion to the same owner's balance.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sui_rpc::{
    field::{FieldMask, FieldMaskUtil},
    proto::sui::rpc::v2::{
        GetCoinInfoRequest, GetEpochRequest, GetObjectRequest, ListOwnedObjectsRequest,
        SimulateTransactionRequest, SimulateTransactionResponse,
        simulate_transaction_request::TransactionChecks,
    },
};
use sui_sdk_types::{
    AccumulatorOperation, AccumulatorValue, Address, Argument, Command, Digest, GasPayment,
    IdOperation, Identifier, Input, MoveCall, Object, ObjectIn, ObjectOut, ObjectReference, Owner,
    ProgrammableTransaction, Transaction, TransactionEffects, TransactionExpiration,
    TransactionKind, TypeTag, framework::Coin,
};
use tap_core::{DirectAssetPolicy, SUI_TESTNET_CHAIN};

pub const ENDPOINT: &str = "https://fullnode.testnet.sui.io:443";
const POLICY: DirectAssetPolicy = DirectAssetPolicy::CircleTestnetUsdc;
const PAYMENT_MINIMUM: u64 = 10_000;
pub mod settlement;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedCoin {
    pub id: Address,
    pub version: u64,
    pub digest: Digest,
    pub owner: Address,
    pub value: u64,
    pub storage_rebate: u64,
}

#[derive(Serialize)]
pub struct Inspection {
    pub endpoint: &'static str,
    pub node: super::NodeInfo,
    pub protocol_version: u64,
    pub asset: &'static str,
    pub decimals: u32,
    pub protocol_minimum: u64,
    pub selected_coin: SelectedCoin,
    pub recipient: Address,
    pub transaction_digest: String,
    pub transaction_bcs_hex: String,
    pub simulation: SimulateTransactionResponse,
    pub blockers: Vec<String>,
    pub qualified: bool,
    pub chain_submission: bool,
    pub notice: &'static str,
}

/// One bounded snapshot, not signing, payment approval, execution or checkpoint settlement.
pub async fn inspect(sender: Address) -> Result<Inspection> {
    tokio::time::timeout(Duration::from_secs(60), inspect_once(sender, None))
        .await
        .context("conversion inspection exceeded 60 seconds; no transaction submitted")?
}

/// Recheck only the frozen candidate; never rediscover a coin or refresh its bytes.
pub async fn qualify(coin: &SelectedCoin, tx: &Transaction) -> Result<Inspection> {
    verify_template(coin, tx)?;
    tokio::time::timeout(
        Duration::from_secs(60),
        inspect_once(coin.owner, Some((coin, tx))),
    )
    .await
    .context("conversion qualification exceeded 60 seconds")?
}

async fn inspect_once(
    sender: Address,
    saved: Option<(&SelectedCoin, &Transaction)>,
) -> Result<Inspection> {
    let node = super::probe(ENDPOINT).await?;
    ensure!(
        node.chain.as_deref() == Some("testnet")
            && node.chain_id.parse::<Digest>()? == SUI_TESTNET_CHAIN.parse::<Digest>()?,
        "conversion requires the pinned testnet genesis"
    );
    let mut c = super::client(ENDPOINT)?;
    let mut epoch_req = GetEpochRequest::default();
    epoch_req.epoch = Some(node.epoch);
    epoch_req.read_mask = Some(FieldMask::from_paths(["epoch", "protocol_config"]));
    let epoch = c
        .ledger_client()
        .get_epoch(epoch_req)
        .await?
        .into_inner()
        .epoch
        .context("missing epoch")?;
    ensure!(epoch.epoch == Some(node.epoch), "epoch response mismatch");
    let protocol = epoch
        .protocol_config
        .context("missing protocol configuration")?;
    let protocol_version = protocol
        .protocol_version
        .context("missing protocol version")?;
    let json = serde_json::to_value(&protocol)?;
    let configs = &json["configs"];
    for flag in ["enable_gasless", "enable_accumulators"] {
        ensure!(
            configs[flag].as_bool() == Some(true),
            "protocol flag {flag} disabled or unknown"
        );
    }
    let asset = POLICY.coin_type();
    let minimum = configs["gasless_allowed_token_types"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row[0].as_str() == Some(asset)))
        .and_then(|row| row[1].as_str())
        .and_then(|s| s.parse::<u64>().ok())
        .context("exact Circle testnet asset/minimum not gasless-allowlisted")?;
    let mut info_req = GetCoinInfoRequest::default();
    info_req.coin_type = Some(asset.into());
    let info = c.state_client().get_coin_info(info_req).await?.into_inner();
    ensure!(
        info.coin_type.as_deref() == Some(asset)
            && info.metadata.as_ref().and_then(|m| m.decimals) == Some(6),
        "exact asset/6-decimal metadata not confirmed"
    );

    let selected = if let Some((coin, tx)) = saved {
        ensure!(
            epoch_of(tx)? == node.epoch,
            "saved candidate epoch is no longer current; bytes retained"
        );
        ensure!(
            coin.value >= minimum.max(PAYMENT_MINIMUM),
            "saved coin no longer meets protocol minimum"
        );
        coin.clone()
    } else {
        let mut list_req = ListOwnedObjectsRequest::default();
        list_req.owner = Some(sender.to_string());
        list_req.object_type = Some(format!("0x2::coin::Coin<{asset}>"));
        list_req.page_size = Some(1000);
        list_req.read_mask = Some(FieldMask::from_paths([
            "bcs",
            "object_id",
            "version",
            "digest",
        ]));
        let listed = c
            .state_client()
            .list_owned_objects(list_req)
            .await?
            .into_inner();
        // No partial scan may claim to have selected the smallest eligible coin.
        ensure!(
            listed.next_page_token.is_none(),
            "owned-coin scan exceeds bounded single page; no candidate selected"
        );
        let mut coins = Vec::new();
        for object in listed.objects {
            let selected = read_coin(&object, sender)?;
            if selected.value >= minimum.max(PAYMENT_MINIMUM) {
                coins.push(selected);
            }
        }
        coins.sort_by_key(|coin| (coin.value, coin.id));
        coins
            .into_iter()
            .next()
            .context("no owned Circle coin covers the payment and gasless minimum")?
    };
    // Re-read the exact reference rather than trusting an aggregate balance or stale index row.
    let mut object_req = GetObjectRequest::default();
    object_req.object_id = Some(selected.id.to_string());
    object_req.read_mask = Some(FieldMask::from_paths([
        "bcs",
        "object_id",
        "version",
        "digest",
    ]));
    let fresh = c
        .ledger_client()
        .get_object(object_req)
        .await?
        .into_inner()
        .object
        .context("selected coin missing")?;
    let fresh = read_coin(&fresh, sender)?;
    ensure!(
        fresh == selected,
        "selected coin changed; no refresh or alternate coin attempted"
    );
    let tx = match saved {
        Some((_, tx)) => tx.clone(),
        None => build(&selected, node.epoch)?,
    };
    let mut req = SimulateTransactionRequest::default();
    req.transaction = Some(tx.clone().into());
    req.read_mask = Some(FieldMask::from_paths(["*"]));
    req.checks = Some(TransactionChecks::Enabled as i32);
    req.do_gas_selection = Some(false);
    let simulation = c
        .execution_client()
        .simulate_transaction(req)
        .await?
        .into_inner();
    let mut blockers = Vec::new();
    if let Err(error) = verify_simulation(&tx, &selected, &simulation) {
        blockers.push(format!(
            "checked conversion simulation not qualified: {error:#}"
        ));
    }
    let after = super::probe(ENDPOINT).await?;
    ensure!(
        after.chain_id == node.chain_id && after.chain.as_deref() == Some("testnet"),
        "chain changed during inspection"
    );
    if after.epoch != node.epoch {
        blockers.push("epoch changed; this candidate must not be signed".into());
    }
    Ok(Inspection {
        endpoint: ENDPOINT,
        node,
        protocol_version,
        asset,
        decimals: 6,
        protocol_minimum: minimum,
        recipient: sender,
        transaction_digest: tx.digest().to_string(),
        transaction_bcs_hex: hex::encode(bcs::to_bytes(&tx)?),
        selected_coin: selected,
        simulation,
        qualified: blockers.is_empty(),
        blockers,
        chain_submission: false,
        notice: "UNSIGNED INSPECTION ONLY. Converts the WHOLE selected coin to the SAME owner's address balance, not a merchant payment. No SUI gas objects/price/budget. Storage rebate is foregone. Not a signing or submission artifact.",
    })
}

fn read_coin(
    object: &sui_rpc::proto::sui::rpc::v2::Object,
    sender: Address,
) -> Result<SelectedCoin> {
    let decoded: Object = object
        .bcs
        .as_ref()
        .context("missing owned object BCS")?
        .deserialize()?;
    let coin = Coin::try_from_object(&decoded).context("not a canonical Coin<T>")?;
    ensure!(
        *coin.coin_type() == POLICY.coin_type().parse::<TypeTag>()?,
        "wrong coin type"
    );
    ensure!(
        *decoded.owner() == Owner::Address(sender),
        "coin is not address-owned by sender"
    );
    let digest = decoded.digest();
    ensure!(
        object
            .object_id
            .as_deref()
            .context("missing coin ID")?
            .parse::<Address>()?
            == *coin.id()
            && object.version == Some(decoded.version())
            && object
                .digest
                .as_deref()
                .context("missing coin digest")?
                .parse::<Digest>()?
                == digest,
        "owned coin BCS/reference mismatch"
    );
    Ok(SelectedCoin {
        id: *coin.id(),
        version: decoded.version(),
        digest,
        owner: sender,
        value: coin.balance(),
        storage_rebate: decoded.storage_rebate(),
    })
}

pub fn build(coin: &SelectedCoin, epoch: u64) -> Result<Transaction> {
    Ok(Transaction {
        sender: coin.owner,
        gas_payment: GasPayment {
            objects: vec![],
            owner: coin.owner,
            price: 0,
            budget: 0,
        },
        expiration: TransactionExpiration::ValidDuring {
            min_epoch: Some(epoch),
            max_epoch: Some(epoch),
            min_timestamp: None,
            max_timestamp: None,
            chain: SUI_TESTNET_CHAIN.parse()?,
            nonce: 0,
        },
        kind: TransactionKind::ProgrammableTransaction(ProgrammableTransaction {
            inputs: vec![
                Input::ImmutableOrOwned(ObjectReference::new(coin.id, coin.version, coin.digest)),
                Input::Pure(bcs::to_bytes(&coin.owner)?),
            ],
            commands: vec![Command::MoveCall(MoveCall {
                package: Address::from_static("0x2"),
                module: Identifier::new("coin")?,
                function: Identifier::new("send_funds")?,
                type_arguments: vec![POLICY.coin_type().parse()?],
                arguments: vec![Argument::Input(0), Argument::Input(1)],
            })],
        }),
    })
}

fn verify_simulation(
    tx: &Transaction,
    coin: &SelectedCoin,
    simulation: &SimulateTransactionResponse,
) -> Result<()> {
    super::qualification::verify_simulation(tx, simulation)?;
    let executed = simulation
        .transaction
        .as_ref()
        .context("missing simulated execution")?;
    let effects: TransactionEffects = executed
        .effects
        .as_ref()
        .context("missing effects")?
        .try_into()?;
    verify_effects(tx, coin, &effects)
}

/// Exact successful conversion effects, shared by simulation and checkpoint observation.
pub fn verify_effects(
    tx: &Transaction,
    coin: &SelectedCoin,
    effects: &TransactionEffects,
) -> Result<()> {
    verify_template(coin, tx)?;
    let TransactionEffects::V2(effects) = effects else {
        anyhow::bail!("V2 effects required")
    };
    ensure!(
        effects.transaction_digest == tx.digest()
            && effects.status == sui_sdk_types::ExecutionStatus::Success,
        "effects identity/status mismatch"
    );
    ensure!(
        effects.gas_object_index.is_none()
            && effects.epoch
                == match tx.expiration {
                    TransactionExpiration::ValidDuring {
                        min_epoch: Some(epoch),
                        ..
                    } => epoch,
                    _ => anyhow::bail!("epoch-bound candidate required"),
                },
        "unexpected gas object or execution epoch"
    );
    // Pinned SDK effects/v2: these are unchanged reads, not mutations. CoinDenyList
    // is exactly 0x403; PerEpochConfig carries no owner/version. Runtime-loaded
    // deny-list child information is not a changed-object exception.
    ensure!(
        effects.unchanged_consensus_objects.len() == 1
            && effects.unchanged_consensus_objects[0].object_id == Address::from_static("0x403")
            && matches!(
                effects.unchanged_consensus_objects[0].kind,
                sui_sdk_types::UnchangedConsensusKind::PerEpochConfig
            ),
        "unexpected consensus inputs: {:?}",
        effects.unchanged_consensus_objects
    );
    ensure!(
        effects.changed_objects.len() == 2,
        "expected exactly coin deletion and same-owner accumulator credit"
    );
    let balance_type: TypeTag = format!("0x2::balance::Balance<{}>", POLICY.coin_type()).parse()?;
    let mut deleted = false;
    let mut credited = false;
    for change in &effects.changed_objects {
        match (
            &change.input_state,
            &change.output_state,
            &change.id_operation,
        ) {
            (
                ObjectIn::Exist {
                    version,
                    digest,
                    owner,
                },
                ObjectOut::NotExist,
                IdOperation::Deleted,
            ) if change.object_id == coin.id
                && *version == coin.version
                && *digest == coin.digest
                && *owner == Owner::Address(coin.owner)
                && !deleted =>
            {
                deleted = true;
            }
            (ObjectIn::NotExist, ObjectOut::AccumulatorWrite(write), IdOperation::None)
                if *write.address() == coin.owner
                    && *write.accumulator_type() == balance_type
                    && write.operation() == AccumulatorOperation::Merge
                    && *write.value() == AccumulatorValue::Integer(coin.value)
                    && !credited =>
            {
                credited = true;
            }
            _ => anyhow::bail!(
                "unexpected conversion effect; no ordinary writes, gas, or other recipients permitted"
            ),
        }
    }
    ensure!(
        deleted && credited,
        "exact consumed coin and full same-owner credit not confirmed"
    );
    Ok(())
}

pub fn epoch_of(tx: &Transaction) -> Result<u64> {
    match tx.expiration {
        TransactionExpiration::ValidDuring {
            min_epoch: Some(min),
            max_epoch: Some(max),
            ..
        } if min == max => Ok(min),
        _ => anyhow::bail!("exact single epoch required"),
    }
}

/// Reconstruction guard, never an invoice-bound payment or an arbitrary transaction signer.
pub fn verify_template(coin: &SelectedCoin, tx: &Transaction) -> Result<()> {
    ensure!(
        coin.value >= PAYMENT_MINIMUM && coin.version > 0,
        "invalid conversion coin value/version"
    );
    ensure!(
        *tx == build(coin, epoch_of(tx)?)?,
        "conversion differs from exact same-owner whole-coin template"
    );
    Ok(())
}
