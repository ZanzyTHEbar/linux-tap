# Resume checkpoint: emulation and real testnet

**As of 29 October 2026: both emulated real-testnet milestones PASS; not production-ready.**
**Later 29 October:** [SQLite foundation](../evidence/sqlite-wal-qualification-2026-10-29.json) passed independent review;
[read-query response loss](../evidence/query-response-loss-2026-10-29.json) passed independent review; execution-loss offline native guards and heavy checks now pass.
This is the canonical high-level handoff, not permission to run another payment.
Read this, the two public evidence excerpts below and the [backlog](../backlog.md)
before consulting the longer [validation history](../validation.md).

## Goal and current work state

Build a native OSS Linux reference wallet and merchant implementation, Sui first,
with strict Circle USDC policy. Prove the complete emulated/simulated flow and
real testnet settlement **before physical hardware**. Those bounded milestones
are complete; the wider product, merchant SDK and production safety are not.

Worktree: `$PROJECT_ROOT`, dirty `main`, HEAD
`f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`. Implementation and documentation
remain uncommitted. Preserve all existing work; do not reset, stage or commit it.
[Backlog](../backlog.md) is the execution record; no in-session task list exists.
The original save left no active workers/runtimes or new work authorization.
Subsequent separately authorized B14 storage work is qualified below, not new funds activity.

## What is implemented

- Three Rust crates: `tap-core` owns signed invoices, merchant-pin/policy checks
  and bounded APDU assembly; `tap-sui` owns exact permitted transaction construction
  and selected-node verification; `tap-cli` orchestrates wallet/merchant workflows.
- Software checkout integrates APDU ingress with invoice verification, explicit
  terminal approval, durable signed bytes, post-credit lost receipt and identical
  restart/replay. Default demo payments remain labelled simulation, not chain-paid.
- Full private PC/SC delivery uses real pcsc-lite/pcscd and upstream vpcd with a
  small reviewed Unix-only socket patch, private permissions/UID checks and a thin
  VICC adapter delegating to the Rust receiver. Transport ACK grants no approval.
- Explicit testnet preparation qualifies the exact candidate and persists approval,
  reservation and immutable prepared BCS/signature/digest before dispatch. Durable
  intent/attempt records precede execution; unknown outcomes preserve the artifact.
  Keyless reconciliation queries its saved digest, never invents a replacement.
- Merchant registration reserves a fresh receiver locally; authoritative-ledger
  uniqueness binds invoice and `(chain, digest)` credit. Acceptance commits linked
  observation/credit; wallet confirmation records the verified receipt association.
- Human-private development setup uses OS entropy and official Sui SDK Ed25519
  key encoding/legacy decoding. Software keystores are unencrypted development
  custody, not production isolation. Only humans generated real keys, signed and
  submitted payments; agents did not open real keystores.

## Concrete completed evidence

All three executions below were Sui testnet, epoch 1267, under the explicit Circle
profile. The two payment excerpts retain exact genesis/type, parties and hashes.

| Milestone | Executed amount / checkpoint | Saved digest |
| --- | --- | --- |
| Same-owner gasless whole-coin conversion, not merchant payment | 20 USDC / 400301849 | `4DoX6pEdoo7ScZy5Pda2Yeuk3XkQ89Kyxw8s5BqBP121` |
| [First complete native private-PC/SC checkout](../evidence/emulated-testnet-checkout-2026-10-29.json) | 0.01 USDC / 400344020 | `7LemioWwAmjRYA9gqsZZBCnCnUgpESH3qfg7mJzJCJxz` |
| [Application-boundary recovery trial](../evidence/emulated-testnet-recovery-2026-10-29.json) | One new 0.01 USDC / 400484678 | `52wFhV654LoLhXycxB5nxn3B2ddM4siC7Mm72opYt9sW` |

The real recovery trial hit both native exit-86 faults: received VerifiedSuccess
before wallet observation persistence, then atomic merchant credit before receipt
export. Fresh-process reconciliation recovered the saved execution; restarted
acceptance regenerated the exact stored receipt, and confirmation associated it.
Each payment retains one artifact, intent, attempt, credit and association. All
prior merchant rows survived; the authoritative ledger has **two total credits**,
the first payment plus one recovery payment, not two recovery payments.

Independent keyless verification matched immutable BCS, signatures, digests and
typed checkpoint deltas (sender SPLIT 10000 / receiver MERGE 10000 per payment).
Duplicate accept/confirm used cached credit/association, not fresh queries; each
milestone's extra reconcile supplied fresh node evidence. Wallet and merchant
queried the same official fullnode separately: not independent node operators or
light-client finality. No current balance was read; do not infer a balance here.
Receipt absence at the credit fault rests on its marker and reviewed contemporaneous
runner assertion, not an independently retained historical filesystem snapshot.
Local receipt files are not authenticated remote business acknowledgements.

**Both receivers are now USED.** Retain private custody, journals and original
authoritative merchant history. Do not rerun historical recipes, reuse a receiver,
reset history with an empty ledger or sign a replacement after uncertainty.
Any future funded trial needs a fresh receiver and separate explicit native human
approval; no default faucet/funding/broadcast authority follows from this save.

## Research leverage: adopted now versus future candidates

The [28 October decision map](../research/README.md) and its
[payment](../research/payment-implementation-map.md) / [phone](../research/linux-phone-implementation-map.md)
source maps are historical research, not a current TODO list or adoption ledger.

| Area | Actually used now | Recommendation for later, not adopted |
| --- | --- | --- |
| Chain | Pinned Mysten Rust types, BCS, crypto/Ed25519, gRPC and execute/checkpoint helper, wrapped by a whole-operation deadline and strict predicates | Keep the SDK; no custom crypto, chain engine or whole-wallet fork |
| NFC | pcsc-rust/libpcsclite, pcsc-lite/vpcd, bounded Unix-only patch and thin VICC/Rust adapter; nfcd bridge and upstream D-Bus XML exist | Qualify phone NFCC/HCE separately; `ndef-share` C and Sailfish Qt lifecycle patterns are references, not adopted wallet code |
| Persistence | rusqlite/SQLite, transactions, uniqueness and cooperative owner locking; patched 3.51.3 runtime and fixture checkpoint/reopen verified | Broader wire/host/power durability remains open; do not build a storage engine |
| Custody | Official SDK key import/encoding and development signer | Gem's separately MIT-licensed Rust keystore is a candidate, not imported code or inherited cross-process security; no GPL whole-wallet fork |
| UI | Terminal approval; Rust retains policy authority | Qt/CXX-Qt thin UI selected, not built; Electrum UX lessons only. Resolve literal no-JS/QML scope and module licensing first |
| Merchant operations | Native CLI/modules and local authoritative credit/receipt workflow | BTCPay, Taler and Open CryptoPay state/refund/reconciliation lessons only, not their engines or a mandatory foundation service; inspected BTCPay paths do not supply atomic outbox/refund guarantees |

Defer Casimir/AOSP/custom-kernel simulation unless a specific NCI question needs
it. Ledger's inspected clear parser mismatches this transaction template; the
inspected Keycard applet rejects Ed25519. Neither justifies blind-signing fallback.
QR/UR remains optional transport, not the next prerequisite. Recheck pinned source,
licenses and profiles before adoption; [THIRD_PARTY](../../THIRD_PARTY.md) records
current provenance. Source review and resolved P2s are not a security audit.

## Outstanding unknowns and actual blockers

- **B05/B14 partial:** process restart, both live application faults and a real
  local read-query response reset are verified. Execution-response loss, host/power
  failure and filesystem/fsync durability are not. The SQLite 3.50.2 source risk is now
  addressed by verified bundled 3.51.3; no baseline race/power-cut reproduction
  is claimed. See [historical security research](../research/transport-security-notes.md).
- **Custody/product safety:** encrypted keys, backup/lost-device recovery, OS
  isolation and trusted-device approval are not qualified. Full application/
  approval separation, remote merchant authentication, trust enrollment/revocation,
  refunds, partial/overpayment accounting and a security audit remain open.
- **Productization/hardware:** no Qt UI, reusable public merchant SDK/API/POS
  deployment or Omarchy packaging is delivered beyond CLI/modules. Private
  experimental AID interoperability is not EMV/public-terminal acceptance;
  physical RF, phone HAL/NFCC and hardware signing remain deferred.
- **Conformance-only blocker:** canonical Open Contactless Payments Initiative
  URL, charter and engineering v0.2 are missing; OCP/Interledger are not substitutes.
  This blocks conformance claims, not the already proven internal emulated flow.
- Testnet protocol/gasless allowlists, metadata and epochs can change. Future
  candidate qualification must be fresh and bounded; prior PASS is not eligibility
  or signing authority today. Human private keys/CAPTCHA are not needed now.

These gaps block production claims, not local B14 qualification design.

## Single best next work item — recommendation only

**The [execution-loss candidate](../execution-response-loss-lab.md) has compiled/offline native guards
PASS and completed independent source/evidence review; live loss remains unqualified.**
Establish wire/host-storage fault evidence preserving existing
artifacts and ledger history. Do not substitute another normal payment, new asset,
UI or hardware task. Productization and secure custody follow this safety work.
This checkpoint does not authorize another fault run or payment.

## Resume map and validation already recorded

Start with this checkpoint, both evidence JSONs and backlog; use the long validation
record only for details. Source entry points: [core](../../crates/tap-core/src/lib.rs),
[Sui adapter](../../crates/tap-sui/src/lib.rs), [CLI testnet](../../crates/tap-cli/src/testnet.rs),
`crates/tap-cli/src/testnet/`, `crates/tap-cli/src/store/`,
[PC/SC runner](../../scripts/pcsc-lab.py) and [recovery runner](../../scripts/testnet-recovery.py).
Recorded PASS commands, also rerun successfully for the SQLite continuation:
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh` — format, Clippy, 15 tests, build and smoke.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo test --workspace --features pcsc-reader --locked` — 15 tests.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo build --locked -p tap-cli --features pcsc-reader` — final PC/SC-enabled build.
Human real checkout/recovery and independent verification passed. The original
save was docs-only; the later SQLite continuation ran the checks above with no
live RPC, real journal/key access, funds activity or VCS mutation. Execution-loss default/PCSC checks also pass; see the candidate for current binary/evidence.
