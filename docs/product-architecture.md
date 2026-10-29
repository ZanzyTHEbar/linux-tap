# Product architecture

> **Current status, 29 October 2026:** the [resume checkpoint](checkpoints/2026-10-29-emulation-testnet.md)
> records implemented durable issuance/approval/reservation/dispatch, explicit
> Circle signing policy, live merchant registration/credit and wallet association,
> plus both real application-boundary recovery faults. The dated proposal and gap
> statements below describe the 28 October baseline, not today's missing work;
> those statements are superseded where the checkpoint records completion.
> The remaining target is still proposed: this is not blanket implementation,
> production custody, wire/host/power-failure or physical RF qualification.

**Status: Proposed target design; not implemented.** 28 October 2026.
Source baseline: `f6ea6a8`. See the dated [independent review record](adr/0003-reuse-first-product-boundaries.md#independent-review-record).
The [bootstrap architecture](architecture.md) records delivered facts;
[product design](product-design.md) defines target contracts and crash barriers;
[backlog](backlog.md) remains the execution list. Research is evidence, not adoption.

Implementation update, 28 October 2026: the stage A application simulation is
[implemented and locally verified](validation.md#integrated-apdu-ingress-and-lost-receipt-28-october-2026).
The full target, including durable issuance barriers and live acceptance, remains
proposed; the earlier review did not review this implementation increment.

## Product and constraints

Deliver a native Linux reference wallet and merchant SDK, with Sui first and a
strict, bounded USDC direct-payment profile. The immediate deliverable is an
integrated simulated checkout, not production custody or public POS acceptance.
Reuse the existing three crates; the boundaries below are logical responsibilities,
not ten services, new crates, a plugin registry or a distributed deployment.

Keep policy, exact transaction construction and approval enforcement in Rust.
No JavaScript/Node chain core, cloud signer or mandatory foundation service.
No whole-wallet fork, replacement chain engine, custom BCS or cryptography.
Users choose their trusted node; merchants observe settlement independently.
USDC issuer controls and network availability remain external dependencies.

The foundation charter and engineering specification v0.2 are not in this
repository. This design cannot claim conformance to them. The exact “Open
Contactless Payments Initiative” is unidentified; Open CryptoPay, Interledger,
EMV and similarly named initiatives are not substitutes for its canonical URL.

## Boundary and authority map

```text
Merchant invoice key + merchant ledger
    | signed invoice
    v
Keyless merchant reader (PCD) -> wallet card transport (PICC)
    | complete bounded untrusted inbox
    v
Wallet verifier + local trust/policy
    | exact candidate review + explicit approval
    v
PaymentSigner -> immutable artifact journal -> Sui adapter
                                                | same signed artifact
                                                v
                                         selected chain node

Merchant independent node observations
    -> durable payment/invoice association -> merchant receipt
```

Invoice signing authority is not wallet spending authority. Transport adapters
have neither. Initial wallet responsibilities may be colocated in one process;
this is **not OS-isolated custody** or a compromised-host defense. Production
custody isolation requires separate qualification, not a box in this diagram.

The current APDUs carry invoices only. Simulator claims and receipts return over
a private Unix socket, not RF. A future return transport needs its own versioned
contract. Transport hash/`9000` ≠ approval ≠ merchant ACK ≠ chain finality.

## Existing modules and target responsibilities

| Owner | Existing source and facts | Proposed responsibility / gap |
| --- | --- | --- |
| `tap-core` | [Invoice policy](../crates/tap-core/src/lib.rs): `SignedInvoice::verify`, `Invoice::validate_checkout`; [APDU receiver](../crates/tap-core/src/apdu.rs) | Keep strict invoice validation outside UI/transport; no DB, hardware or RPC authority |
| `tap-sui` | [Adapter](../crates/tap-sui/src/lib.rs): exact template, `Prepared`, `PaymentSigner`, `ApprovalProfile`, selected-node verification | Keep SDK serialization/signing; later dispatch/reconcile exact saved artifact under explicit policy |
| `tap-cli` | [Commands](../crates/tap-cli/src/main.rs), [Store](../crates/tap-cli/src/store.rs): fixture wallet/merchant, journals, reader and qualification commands | Initially orchestrate inbox, review, single-owner issuance, persistence and evidence; not production key management |
| NFC glue | [nfcd bridge](../adapters/nfcd/bridge.py), optional PC/SC `nfc_push` | Keyless lifecycle/framing only; synchronous callback deadline remains unqualified |
| Thin native UI | Qt/CXX-Qt selected, no implementation | Display Rust-verified terms and invoke bound approval; no independent transaction builder |
| Merchant SDK role | Simulator verification and unique ledger rows exist in CLI | Extract only when an actual consumer needs it; independent live observations/accounting remain open |

No new public ABI is declared here. Conceptual contracts in the design are
integration obligations, not implemented methods or stable SDK promises.

## Invoice and trust boundary

Retain [experimental v0](protocol.md): private AID `F04C54415000`, ten-field
canonical CBOR core and strict envelope. Pin the full genesis digest, full Move
asset type, integer base-unit amount, recipient, lifetime and direct profile.
Require an independently supplied expected merchant key. Signature validity
authenticates that key, not a self-signed label's business identity.

Unknown fields, alternate encoding and arbitrary transaction requests remain
rejected. No automatic asset/network change, URI rewriting or symbol-based
selection. The explicit Circle testnet read-only policy is not signer policy.
Preserve current byte bounds: 2,048-byte core, 8,192-byte NFC envelope and
220-byte PUT chunks. Reader/controller deadlines require measurements later;
this document invents no numeric timing SLA.

NDEF, QR, Open CryptoPay HTTP and EMV are not implicitly adopted. A future ingress
adapter must emit an untrusted request into the same verifier, never signing
instructions. Public AID allocation, independent interop and commercial-terminal
deployment rights are separate gates.

## Native NFC integration

Select nfcd LocalHostApp on a **qualified hardware adaptation**. Reuse asynchronous
registration, owned request handles and cleanup from C `ndef-share` and Qt
`sailfish-nfcshare`, not their NDEF AID/application semantics. See the
[implementation map](research/linux-phone-implementation-map.md).

The full path requires binder/pn54x or another real I/O/HAL backend beneath
libncicore/libnciplugin. Mainline NFC hardware, a readable tag or a PC/SC reader
does not establish host ISO-DEP listen routing. PN7160's vendor HCE stack is
another qualified direction, not an already-existing nfcd plugin.

Use the shared OS daemon. Release only our AID, mode and tech resources; handle
daemon loss and re-registration. Never restart shared services, change global
NFC policy or turn off Android to hide contention. FuriOS/Andromeda contention
is source evidence, not runtime arbitration proof.

APDU callbacks do bounded transport work, never human approval, network waits
or password/KDF work. Only a complete ready inbox may cross to wallet policy.
The existing `create_new` + file sync path exposes the final file while writing
and does not sync its parent: it is not atomic publication or power-cut proof.
Do not watch a partial filename and parse prefixes. Qualify stronger publication
before concurrent inbox consumption; ingress gains no wallet DB/key authority.

## Approval, issuance and persistence

Freeze candidate SDK BCS/digest, signer address/scheme/profile, policy version,
chain/epoch/nonce and verified terms before actual signing. Approval names this
candidate; a subsequent tap, UI update or policy change cannot retarget it.
Revalidate before issuance; changed conditions require new review/approval,
not mutation of already approved bytes.

Use one wallet authority per state directory and serialize issuance **before**
the signer side effect. Target records distinguish approval, issuance reservation,
immutable artifact, dispatch intent, observations and receipt. Current Store does
not implement those durable approvals/reservations/dispatch states.

Unique INSERT after signing alone protects the first committed artifact,
not against two concurrent signer invocations. The current CLI also holds a
cooperative OS DB-file owner lock before review/signing; that is not a durable
issuance reservation or recovery of a signature lost before commit. Keep DB constraints and
transactions; reload the committed winner on conflict or fail closed. Do not
add distributed leases or a second queue. An external hardware signature and
a DB commit are not atomic: interrupted issuance stays uncertain until signer
recovery/idempotency is qualified. Never blindly sign again.

Artifact durability precedes any dispatch; dispatch intent precedes send;
observation durability precedes a paid label. Reconcile and retransmit the same
BCS, signature and digest after possible external effect. Timeout, RF loss,
suspend or UI cancellation cannot authorize a fresh nonce or replacement payment.
Expiry blocks new issuance, not reconciliation or revocation of signed bytes.

WAL + FULL still depends on OS/storage/fsync behavior. The inspected bundled
SQLite is 3.50.2; effective runtime and WAL-reset fix qualification are required
before broader concurrent durability claims. See
[security notes](research/transport-security-notes.md). Process restart evidence
is not power-loss evidence. Secure private parents/files, journals and logs;
use coordinated online DB/WAL backup, not raw live copies or temporary production
backup storage. Custody recovery and signed-artifact recovery are distinct.

## Sui settlement and merchant accounting

Keep pinned Mysten SDK types, BCS, digest, signing and gRPC. Retain the narrow
offline construction adapter and [ADR 0002](adr/0002-empty-gas-builder.md).
The async SDK builder may mutate via simulation/gas selection; it is not a
drop-in replacement for an exact approved candidate. Do not add fake gas objects.

Later reuse the pinned execute/checkpoint helper, with full transaction/effects/
checkpoint read fields, chain pin, exact template/signature verification and an
outer whole-operation deadline. Helper success alone is not successful effects
or merchant acceptance. Inclusion trusts the selected node, not a light client.
Typed observed execution failure versus missing/transport evidence is proposed;
current `anyhow` errors are not a durable payment-state model.

Merchant acceptance independently observes the same artifact and commits a
single-credit `(chain, digest)` plus invoice association across invoices within
each merchant operator's authoritative ledger, not a network-global shared
registry or service. Direct transfers contain
**no invoice ID or onchain invoice deduplication**. The signed core hash excludes
the merchant key: future identity namespaces must bind merchant policy, invoice
ID/core hash and profile. Reject changed terms under one ID and conflicting
artifacts; do not imply cryptographic onchain invoice binding.

Unique receiving custody/correlation (B15) is a live-acceptance gate, not supplied
by local deduplication. Partial, overpaid, late and refund facts stay separate
from the original payment. Refunds need separately authorized intent, destination
and reconciliation; never infer a refund destination from sender alone.

## Selective reuse, not inherited guarantees

Gem's current MIT `core/LICENSE` makes its keystore a later candidate behind
`PaymentSigner`, not an adoption. Process-local mutex/rename does not guarantee
cross-process no-clobber; metadata is unauthenticated until decryption, and import
identity/private-parent/recovery gaps need review. Do not replace Mysten signing
with opaque metadata signing. See [payment source map](research/payment-implementation-map.md).

Ledger's inspected clear parser expects split-result wiring rather than our
direct withdrawal input. Keycard's inspected applet rejects Ed25519. Sui's other
schemes need explicit new account/policy qualification, not a fallback. Never
enable blind signing to bypass incompatibility. Profiles must truthfully distinguish
`DevelopmentSoftware`, `HardwareKeyWithHostApproval` and `TrustedDeviceApproval`.

Qt/CXX-Qt remains the future thin UI direction, not a stage A blocker. QML is
JS-capable; clarify whether the no-JS requirement covers only wallet/chain core
or every UI runtime. Qt components have separate licensing obligations.
Electrum supplies review/sign/export/submit UX lessons, not a Bitcoin engine.
BTCPay supplies states/restart/exception lessons, not a transactional outbox or
atomic refund guarantee in inspected paths. No webhooks are needed for stage A.
Later notification needs a durable state/event barrier and consumer deduplication;
notification delivery must never be authority for paid status.

## Evidence traceability and current gaps

| Source → decision | Current gap | Gate |
| --- | --- | --- |
| [Local/payment source map](research/payment-implementation-map.md) → wire existing receiver/verifier/journal | Original gap closed for local simulation; no production issuance-state claim | A verified; see dated validation above |
| [Phone source map](research/linux-phone-implementation-map.md) → external vpcd + thin VICC | [Private Unix-only real PC/SC flow verified](pcsc-lab.md); independent increment review pending, no RF evidence | B virtual PC/SC verified locally |
| Same map → optional Casimir only for NCI question | `t4at.active` has no APDU reply; rustutils manifest/routing gaps | C optional NCI |
| [Phone survey](research/linux-phones-nfc.md) → qualified PN7160 or working nfcd phone | Exact model/HAL/firmware/routing/timing unknown | D physical evidence |
| [Payments audit](research/open-payments-reuse.md), [qualification](testnet-qualification.md) → keep strict Sui profile | No controlled fresh-wallet eligibility or submission | E separately authorized testnet |
| [Security notes](research/transport-security-notes.md) → single-owner barriers and custody qualification | SQLite runtime fix, power loss, signer recovery unproven | Before expanded durability/custody claims |

The [design gates](product-design.md#evidence-gates) define pass evidence. Hardware
and live-chain qualification are independent axes, not a mandatory C → D → E
sequence. A pilot additionally needs recovery, accounting, security review and
willing POS permission. Emulated PASS cannot close those obligations.

## Open owner decisions

- Product owner: provide canonical initiative URL and missing charter/v0.2;
  reconcile requirements before any conformance statement.
- Trust/custody owner: select production merchant enrollment/revocation, node trust,
  custody/backup/recovery model and required approval profile.
- UI owner: resolve literal no-JS interpretation before Qt Quick implementation.
- Hardware owner: choose an already-owned exact board/phone and acceptable BSP/HAL
  dependencies before hardware work. No purchase or flashing is authorized here.

These do not block internal stage A design. [ADR 0003](adr/0003-reuse-first-product-boundaries.md)
records why these boundaries were selected over larger forks and services.
