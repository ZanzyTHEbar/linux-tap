# Research decision map

> **29 October 2026:** see the [current implementation checkpoint](../checkpoints/2026-10-29-emulation-testnet.md)
> for completed simulation, private PC/SC, real testnet checkout and application-
> boundary recovery. Read this 28 October research as an archival decision/source
> map, not today's TODO list; recommendations do not themselves mean adoption.

**28 October 2026 — recommendations, not adoption or production qualification.**
Baseline: `main` at `f6ea6a8`. Start here for decisions; follow the reports for
source paths, revision anchors, licenses and unresolved evidence.

The findings are synthesized in the proposed [product architecture](../product-architecture.md)
and [product design](../product-design.md), not implemented capabilities.
For concrete source reuse, read both the [Linux-phone implementation map](linux-phone-implementation-map.md)
and [payment implementation map](payment-implementation-map.md).

## The next deliverable stays small

**Emulated APDU invoice → verified exact terms → explicit approval → durable
signed bytes → lost receipt/restart → identical replay → one simulated receipt.**

Use the existing native Rust wallet and merchant, public laboratory signing
fixtures and synthetic chain. No new hardware, production keys, live network,
funding, wallet fork or foundation service is needed. Keep
`SIMULATED_PAID_NOT_ON_CHAIN` and `chain_submission: false` visible.
The expired checked-in invoice is transport input, not an approvable request;
the acceptance scenario needs valid laboratory terms using the existing fixture
mechanism, not new custody.

The current demo proves less: APDU reconstruction happens separately from the
wallet flow, and its controlled interruption is before sending the claim.
The next increment should connect those existing paths and lose the merchant
reply **after** its simulated payment has committed.

## Read by decision

| Question | Start with | What it supplies |
| --- | --- | --- |
| What does the named initiative actually implement? | [Payments and reuse audit](open-payments-reuse.md) | Unresolved exact identity; verified OCP/Ethereum/Interledger distinctions, SDK and wallet reuse |
| Which native Linux NFC boundary is credible? | [Linux phones and NFC](linux-phones-nfc.md) | Devices, drivers, nfcd/NXP paths, PC/SC emulation, Casimir and evidence ladder |
| What can break transport, durability or signing claims? | [Transport/security notes](transport-security-notes.md) | SQLite qualification, ACK boundaries, QR/UR/schema/tooling and Keycard source gaps |
| What is already runnable? | [Demo](../demo.md) and [validation](../validation.md) | Existing recorded simulation claims; not tests rerun by research |
| What is the current wire format? | [Protocol](../protocol.md) and [architecture](../architecture.md) | Experimental private AID, invoice and trust/process boundaries |
| What blocks live Sui? | [Testnet qualification](../testnet-qualification.md) | Exact network/asset policy and fresh-wallet/address-balance limitations |

The phones report remains the hardware worker's detailed source of truth;
this map does not duplicate its per-device inventory or certify any phone.

## Ordered stages: prove one boundary at a time

### A. Application-only acceptance — next

- Feed emulated SELECT/OPEN/PUT/COMMIT through the real bounded Rust receiver.
- Consume its exact persisted invoice in the existing wallet verifier.
- Verify the independent merchant pin, network, full asset, amount, recipient,
  expiry and exact transaction template before explicit approval.
- Persist the signed artifact before the claim; retain BCS, signature and digest.
- Let the merchant record its simulated payment, suppress the reply, restart,
  resend the journaled artifact and independently inspect both journals.
- Accept only identical replay with one simulated payment/receipt association.

**Evidence:** integrated process/journal scenario, not mocked helper assertions.
No human approval or RPC wait belongs inside an APDU callback.
APDU ACK, merchant payment ACK and chain finality are different events.

### B. Isolated virtual PC/SC — after A

Use the existing merchant `pcsc-rust` path with pcsc-lite/vpcd and a thin virtual
ICC adapter to the same receiver. Do not implement a second wallet.
The inspected vpcd server binds **`INADDR_ANY`**: isolate it in a private lab
environment or review a bounded bind restriction before any future launch.
Record framing/reset/ATR behavior, selected reader/protocol and lost-receipt
result. A PASS proves virtual PC/SC integration, not NFC RF or phone HCE.
No service is authorized or started by this research recommendation.

### C. Casimir/NCI — optional, only for a needed boundary

Use the phones report's native Casimir/NCI analysis if the next question is
controller reset/discovery/host routing, not wallet approval/idempotency.
Qualify current build dependencies, CE routing and adapter lifecycle first.
Do not insert a kernel simulator or full Android environment into stage A.
Stage letters here express decision order; the detailed phones report organizes
physical and optional NCI increments differently.

### D. One physical board or phone — separately scoped

Prefer already-owned hardware with a demonstrated host-APDU path. Candidate
directions are PN7160/vendor Linux HCE or a functioning Sailfish/nfcd adaptation.
Controller presence or tag reading alone is not ISO-DEP listen-mode proof.
Record exact board/model, kernel/driver, firmware, host routing, timing, field
loss and approval profile. Do not purchase, flash or replace a driver merely
to finish the application milestone.

### E. Controlled testnet — separate authorization gate

Only after fresh controlled-wallet eligibility, exact asset/address-balance
qualification, custody and reconciliation review, and explicit submission
authorization. Public-address simulation is neither control nor settlement.
No automatic fresh transfer after timeout; no silent network/asset substitution.
Do not add a blanket SUI requirement to protocol-gasless stablecoin transfers.

## Reuse decisions that matter

| Area | Recommendation | Important limit |
| --- | --- | --- |
| Sui core | Keep pinned Mysten Rust types, BCS, signing and gRPC | Pin is still observed upstream head; offline empty-gas builder gap remains |
| Submission later | Reuse pinned execute/checkpoint helper | Still needs durable journal, exact receipt predicate, chain pin and bounded reconciliation |
| Persistence | Keep first-signed-artifact and unique simulated-ledger rules | Bundled SQLite 3.50.2 needs WAL-reset-fix qualification before broader concurrent durability claims |
| Custody later | Review Gem's separately MIT-licensed Rust core/keystore | Not a blanket MIT license for apps or evidence of recovery/security qualification |
| Ledger later | Reuse app/transport tooling only after exact-template qualification | Current parser expects split-result wiring, unlike our direct withdrawal input |
| Keycard later | Native MIT Rust PC/SC/secure-channel SDK is a real candidate | Card applet rejects Ed25519 despite host API constants/README claims; no Sui trusted display proved |
| UI later | Keep selected Qt/CXX-Qt direction pending requirement interpretation | QML is JS-capable; Qt module licensing and mobile qualification are separate |
| Merchant operations | Adapt BTCPay/Taler/OCP exception/refund/reconciliation lessons | Their engines, HTTP status and payment rails are not Sui finality |
| QR/UR fallback | Optional transport of the same authenticated request | Not required next; no arbitrary URI auto-open or automatic Sui UR profile |
| Security tooling | Consider native dependency checks and isolated lab instrumentation | No new mandatory framework; scan success is not a security audit |

These are candidate selections, not adoption commits or permission to install,
launch services, provision keys, publish issues or spend funds.

## Stop misleading shortcuts

- The exact **“Open Contactless Payments Initiative”** is unresolved, not proved
  nonexistent. Obtain its canonical URL/author before asserting compatibility;
  missing identity does not block the simulation.
- Open CryptoPay's verified HTTP/LNURL flow is not our APDU application.
  Its `api` repository is a merchant map API, not a Sui settlement backend.
- Generic NFC, a UID, secure-element certification or `9000` cannot authorize
  payment. Host-held approval is not trusted-device approval.
- NFC Forum Type 4/NDEF transport and EMV kernel acceptance are not interchangeable.
  A specification being available does not grant terminal deployment rights.
- A wallet process restart is not a power-cut/fsync/filesystem durability test.
  Keep database/WAL handling and transitive bundled C versions in the evidence.
- “SDK supports Ed25519”, “UR supports bytes”, or “COSE signs CBOR” does not
  establish the actual card algorithm, application profile or merchant trust.

## Acceptance and missing proof

For stage A, compare the **exact signed artifact**, not only the digest printed
on screen. Show the merchant's existing record after lost receipt and replay.
Keep the return channel explicit: private socket for the simulator, not a claim
that signed payments or receipts travel over the current invoice-only APDUs.
Preserve the current experimental AID/version and reject unapproved terms.

Still missing: independent protocol interoperability, production custody and
recovery, exact hardware signer clear display, physical RF/timing, native mobile
packaging, controlled live submission and independently verified finality.
Neither public code nor an audit badge closes these gaps.

## Evidence provenance and work performed

The two detailed reports distinguish worker source inspection from upstream
claims. The supplement explicitly attributes the coordinator's independent
28 October findings; Keycard sources were additionally fetched for this followup.
Local Keycard access was limited to its public README, not deployment/key files.
Research validation is local Markdown link existence, whitespace/newline checks
and read-only Git scope inspection. Builds, test suites, demos, cameras, readers,
emulators, services, live RPC and physical devices were **not run** for these
research-only additions. **Owned runtimes: none.**
