# Product design

**Current milestone, 29 October 2026:** the first native private-PC/SC emulated
real testnet checkout is verified complete: human-approved preparation/execution,
merchant credit and wallet association. [Public evidence](evidence/emulated-testnet-checkout-2026-10-29.json).
The recorded receiver is now used; retain custody/journals and do not rerun that
recipe. The subsequent scoped application-boundary recovery trial is verified
below; wire-level unknown outcomes and host/power failure remain unqualified.
Hardware stays deferred. The remaining target design below
is still proposed, not a production, custody or power-cut qualification.

Application-boundary recovery checkpoint (human-run PASS independently verified
29 October 2026; source review completed in
`ses_e7d355137ffeaXiT10YXbCLGve`, signal-orphan P2 **RESOLVED**, no remaining
material findings): submit can exit after receiving
verified execution but before observation persistence, and accept after atomic
credit but before receipt export. These are explicit 0.01 USDC first-use testnet
fault flags, not dropped RPC or power-cut injection. A keyless real-terminal
runner checks exact-artifact restart/reconciliation and authoritative-ledger
row preservation without resubmission. Both actual exit-86 branches and recovery
passed for one new 0.01 USDC payment, checkpoint 400484678.
[Sanitized evidence](evidence/emulated-testnet-recovery-2026-10-29.json) separates
cached duplicates from fresh queries and historical receipt-absence support from
a standalone filesystem snapshot (none exists). Two authoritative credits are
the first payment plus one trial payment. Both receivers are used; no new funds
activity or empty-ledger reset is authorized. No wire/host/power-cut, RF, custody
or authenticated remote-ACK qualification follows from this scoped trial.
Confirm's atomic observation/association is unchanged; no third fault is added.

Historical 28 October implementation notes:

Conversion continuation, 28 October 2026: separate native inspect/prepare/submit/
reconcile commands now exist. The inspected whole owned Circle coin goes only
to the same owner, with no invoice fabrication, splitting, sweep or paid-SUI
fallback. The reviewed P2 permits only the singleton unchanged deny-list
`PerEpochConfig` at `0x403`; changed effects remain exactly coin deletion and
full same-owner accumulator merge. One corrected public unsigned inspection
passed. Approval binds all whole-coin terms and foregone rebate through a
full-candidate operation hash, typed summaries, durable pre-sign reservation and
immutable artifact. Every saved recovery/dispatch/observation is validated.
Keyless submission reconciles the saved digest first, commits approved intent
before execution, and retains identical bytes on uncertainty. Historical
checkpoint-bearing responses prevent later resends. Address-balance indexing
is separate from checkpoint success. The journal is one cooperative local
operation, not hostile-storage authentication or cross-journal coordination.
Only public-fixture recovery/signing paths were exercised. Independent lifecycle
review completed 28 October 2026 (`ses_e7c57274cffe7qT1NSnKx1gggB`), with no
demonstrated P1/P2; the previous read-only `0x403` P2 is resolved. Review found
the 1,537 lines bounded to four responsibilities, without a redundant framework.
The human subsequently completed the 20 USDC same-owner conversion, with saved
keyless checkpoint verification and positive public payment qualification.
Merchant payment, merchant credit and wallet confirmation remain pending.
The private PC/SC runner's new delivery-only branch publishes an exact public
invoice inbox without wallet policy, keys or RPC; independent delivery/workflow
review completed 28 October 2026 (`ses_e7d4e67afffeE9pFbLkvW2fFdS`), with no
blocking/material P1/P2. Human-only preparation is next, not submission.
No new power-loss or custody qualification is claimed.

**Status: Remaining target design proposed; not fully implemented.** 28 October 2026.
Baseline `f6ea6a8`; see the dated [independent review record](adr/0003-reuse-first-product-boundaries.md#independent-review-record). This is a behavioral contract,
not an assertion that the current CLI exposes these records or APIs.
Read [architecture](product-architecture.md) for ownership and reuse rationale,
[protocol](protocol.md) for current bytes and [backlog](backlog.md) for execution.

Implementation update, 28 October 2026: [stage A simulation passed local checks](validation.md#integrated-apdu-ingress-and-lost-receipt-28-october-2026).
Stage B private PC/SC also passed local checks and independent read-only review.
The explicit testnet preparation checkpoint now implements durable candidate,
approval, pre-sign reservation, artifact and deterministic software recovery
records, verified only with public offline fixtures. Its independent review P2
was fixed and independently resolved with no remaining finding. The subsequent
controlled-dispatch slice implements keyless submit/reconcile, pre-RPC intent,
bounded SDK execution, independent keyless merchant registration/credit and wallet
chain-confirmed receipt association through private artifact files. This new
slice at that checkpoint awaited independent review and live dispatch/credit
evidence. Subsequent review and the 29 October milestone above close one
emulated testnet checkout, not the rest of the proposed state model or live fault
qualification. Physical hardware remains deferred.

## One checkout, two independent authorities

The merchant signs invoice terms and later credits an independently observed
payment. The wallet verifies terms, obtains explicit approval, issues one bound
artifact and retains it through uncertainty. Transport only moves requests.
The default offline application scenario uses public fixture keys and a synthetic chain.
It must visibly retain `chain_submission: false` and
`SIMULATED_PAID_NOT_ON_CHAIN`; no live-chain paid label is available.

### Wallet flow

1. Receive the complete invoice through the actual bounded APDU receiver.
2. Publish a ready, private, untrusted inbox item; return transport status only.
3. Verify the separately supplied merchant pin and strict invoice/policy terms.
4. Build one exact allowed candidate using SDK types; display verified terms.
5. Require explicit approval of that candidate and declared approval profile.
6. Revalidate, reserve issuance durably, then invoke the signer under single ownership.
7. Commit the immutable artifact before any claim or later chain dispatch.
8. Record dispatch intent, send the saved artifact and await evidence.
9. Persist evidence/association before showing simulated or independently observed paid.
10. On interruption reopen the same journal, reconcile or replay the same artifact.

Step 6's reservation exists in explicit testnet preparation; step 8's dispatch
records now exist in the controlled testnet command path. Simulator persistence
is unchanged. The baseline gap analysis below describes the earlier simulator;
it does not erase these bounded additions or qualify their live behavior.
`pay_exchange --invoice-inbox` now consumes the actual receiver
output, matching it against the socket's signed envelope before verification and
approval. The demo waits for receiver completion/ACK before launching the wallet.

### Merchant flow

Create a fresh valid lab invoice for the acceptance run, retaining its terms and
key policy. Push it as reader/PCD to the wallet card/PICC application. Receive
the simulator claim over a private Unix socket, not checkout APDUs. Verify the
artifact/template and invoice association, then commit the simulated ledger
before responding. Lost response does not remove that committed payment.
For future live acceptance, independently query an explicitly trusted node and
apply the complete receipt predicate before durable credit and receipt issuance.

## Conceptual records and identity

These are minimum data obligations, not a proposed public ABI or a mandate for
one table per row. Use the existing store and real constraints/transactions.

| Record | Identity and immutable content | Lifetime / authority |
| --- | --- | --- |
| RF assembly | Selected session, declared length/hash, bounded chunks/offset | Transient; reset/reselect discards only assembly |
| Inbox item | Complete exact envelope bytes, content hash, ready marker | Durable untrusted input; no payment authority |
| Verified request | Core bytes/hash, expected merchant policy/key, invoice ID, profile | Durable binding needed before issuance; validation is time/policy dependent |
| Candidate | Exact unsigned SDK BCS/digest, sender/scheme, chain, epoch, nonce | Freeze before approval; never silently retarget |
| Approval | Candidate digest/bytes identity, verified terms, policy version, signer/profile | Explicit decision with scope; not a generic “connected” flag |
| Issuance reservation | Unique request-to-candidate binding, owner and issuance status | Durable before signer side effect; blocks competing issuance |
| Artifact | Exact BCS, signature, digest and approval/reservation association | Immutable; durable before external send |
| Dispatch intent | Artifact identity, destination role, attempt and unresolved state | Durable before each send; retries retain same artifact |
| Observation | Source/node, chain/digest, returned transaction/effects/checkpoint or error evidence | Append facts; absence is not non-execution |
| Receipt association | Merchant policy/invoice identity, chain/digest, status/evidence reference | Durable before paid UI; simulated and live evidence separate |

Keep the invoice core hash distinct from envelope hash and transaction digest.
The core hash does not include merchant key. Future namespace is merchant policy
+ invoice ID/core hash + profile; identical IDs with changed terms conflict.
Reject conflicting artifact associations rather than silently credit twice.
Merchant `(chain, digest)` single credit prevents one transfer from being assigned
twice across invoices within each merchant operator's authoritative ledger, not
through a network-global shared registry or service. It does not put an invoice
ID into a direct Sui transfer.
Fresh receiving custody/correlation (B15) remains required before live acceptance.

## Inbox and transport contract

Accept only the existing private AID and short SELECT/OPEN/PUT/COMMIT profile.
Maintain 8,192-byte envelope and 220-byte chunk limits; the strict signed core
remains at most 2,048 bytes. Repeated exact chunks/COMMIT follow current receiver
semantics; changed/out-of-order input aborts. No new protocol fields are implied.

The APDU adapter owns transient assembly, reset, bounded reads and response
delivery. `9000` confirms transport, `6A80` rejects/aborts input, and `6581`
reports persistence failure. COMMIT's hash is byte integrity, not merchant trust.
Do not hold a callback open for user confirmation, unlock/KDF or network calls.
Actual callback/operation deadlines must be measured on the chosen stack; bound
subprocess stalls before hardware qualification. No invented timing SLA applies.

Hand off only a complete ready item over private IPC or a qualified publication
boundary. The current `apdu_stdio` final path can be visible during writing:
`create_new`, file `sync_all` and byte equality on replay do not establish atomic
publication or parent-directory power-loss durability. Do not let a file watcher
consume a partial prefix. Reject incomplete/conflicting input, preserve exact bytes,
and qualify no-clobber ready publication plus directory sync where files are used.
An adapter must not gain signer, wallet spend DB or policy-edit authority.

LocalHostApp integration exports/registers asynchronously, retains owned mode/tech
request handles, and unregisters/releases only those resources on exit. Daemon
loss requires safe re-registration, not wallet reissuance. Do not restart shared
nfcd, rewrite system NFC policy or stop Android to conceal controller contention.
Screen unlock, selected AID and ResponseStatus are not approval or settlement.
The current Python bridge is synchronous and deadline-unqualified, not a complete
production implementation of this lifecycle.

## Verification and exact candidate approval

Decode using the existing strict verifier. Require independently supplied merchant
key, full genesis, full asset type, integer base-unit amount, recipient, creation/
expiry and supported profile. No unsigned display field, URI host or self-presented
key can substitute for merchant trust. Reject unsupported chain/type/template;
do not switch to another network or coin to make a request payable.

Review shows verified merchant-key identity/trust scope, full network/type,
base-unit amount (and only policy-derived decimal presentation), recipient,
invoice hash, expiry, sender and actual approval profile. Explain simulation
and selected-node trust honestly; a merchant label alone is not business identity.
Do not present a hardware-held key as trusted transaction display.

Freeze the exact candidate bytes/digest and signer address/scheme/profile with
policy version and invoice terms before the approval action. Epoch/nonce are
part of that candidate, not regenerated after approval. UI refresh, second tap,
policy change or signer selection cannot mutate the approved object. A changed
candidate needs a new review. The signer checks the exact template again.

Recheck expiry, policy and applicable eligibility before issuance. A failed
recheck stops new signing; it never edits approved bytes. Account eligibility
and read-only simulation are separate from signature/template validity. Stage A
uses synthetic policy, not live gasless eligibility. Autoapproval is permitted
only as an explicitly named lab-fixture mode confined to the synthetic chain.

## User-visible states

State names below are proposed presentation/domain states, not existing CLI enums.
Persist consequential facts; UI is a view of evidence, not their authority.

| State | Entry / permitted action | Must not imply |
| --- | --- | --- |
| Received, unverified | Complete inbox; verify or discard | Trusted merchant or approved payment |
| Review | Verified request + exact candidate; approve or decline | Signing already authorized |
| Declined/cancelled before issue | No possible signer side effect; close | Revocation of any older artifact |
| Issuing, unresolved | Reservation exists; signer may have acted | Safe to invoke signer again |
| Prepared | Artifact durably committed; send/reconcile same artifact | Submitted, paid or revocable |
| Awaiting observation | Possible send or missing evidence; reconcile/replay same artifact | NotFound/timeout means failed |
| Confirmed execution failure | Verified terminal failed effects for exact chain/digest | No other transfer exists or automatic new approval |
| Simulated paid | Durable simulator association | Chain submission/finality |
| Independently observed paid | Merchant predicate and durable live association pass | Light-client proof or refund completion |

Before any possible issuance, cancellation stops the operation. After a signer
may have acted, closing the UI is not cancellation of issuance. After possible
dispatch, timeout, missing lookup, RF loss, suspend and cancellation remain unknown
until qualified evidence resolves the exact artifact. A “Stop waiting” action
can stop foreground observation, not release a replacement-payment path.

Invoice expiry blocks new issuance; it is not millisecond revocation of signed
bytes. Reconcile/read an existing receipt separately from fresh approval so an
expired request cannot hide an already-issued payment. The CLI now distinguishes
new issuance from saved-artifact replay, including interactive replay after expiry.
No automatic fresh nonce, rebuild, quote refresh-and-pay or fresh transfer follows
uncertainty. Any eventual new payment is a separate explicit reconciliation decision.

## Issuance and crash barriers

One wallet authority owns a private state directory. Acquire exclusion before
issuance and keep DB uniqueness/transactions as the final data guard. Do not
implement distributed leases, another queue or an in-memory-only lock as durable
authority. The CLI now holds a cooperative OS DB-file lock before review/signing.
Unique INSERT still follows signing; the lock excludes cooperating concurrent
owners, but supplies no durable reservation for interrupted issuance.

| Barrier | Durable fact before effect | Crash/restart rule |
| --- | --- | --- |
| Before signing | Verified request, exact approval and issuance reservation | No competing candidate; interrupted signer call remains unresolved |
| Before dispatch | Verified immutable artifact committed | Load/reverify that artifact; never regenerate |
| Before send | Dispatch intent committed for exact artifact | Assume possible delivery; reconcile or resend identical bytes |
| Before paid display/receipt | Validated observation and merchant association committed | Reconstruct state/return existing receipt from durable facts |

A hardware signature and SQLite commit cannot be one atomic transaction.
Recovery must establish signer recoverability/idempotency before claiming safe
automatic resume of interrupted issuance. If signature recovery is unavailable,
fail closed and show issuance uncertainty, rather than silently sign again.
For storage busy/uniqueness conflicts reload a committed winner and validate its
binding, or report conflict. Do not create a new payment to evade the error.

Current guarantees are narrower: `Prepared` stores BCS/signature/digest before
claim; unique INSERT prevents committed replacement; merchant simulator commits
before reply. There is no durable approval, issuance reservation, dispatch state
or automatic startup reconciler. Current receipt rows are not complete live
evidence records. Normal merchant processing is serial, not proven multiwriter
idempotent success under all races.

WAL/FULL depends on filesystem/storage sync. Qualify effective SQLite runtime
and the 3.50.2 WAL-reset-fix gap before expanding concurrent durability claims.
Private parent directories and restrictive files apply to existing paths too,
not just creation modes. Journals, signed artifacts and logs contain sensitive
payment data. Keep secrets out of shell args, UI properties and transport logs.
Use coordinated online DB/WAL backups outside temporary production storage;
restore/recovery evidence is separate from process restart and power-cut testing.

## Dispatch, observation and receipt predicate

Future submission sends **the saved BCS + signature + digest**, not a re-resolved
transaction. Reuse Mysten's pinned execute/checkpoint helper with a full receipt
read mask and an outer deadline covering setup, execution and lookup too. The
helper's checkpoint timeout does not bound the whole operation. Cancellation of
a future does not retract an already-delivered transaction.

For live receipt verification require local artifact signature/template validity,
selected node chain pin, matching execution/effects digest, identical returned
transaction, successful effects and checkpoint inclusion. Request all required
transaction/effects/checkpoint fields. SDK `Ok` or HTTP/gRPC success alone is
insufficient; missing fields/transport errors remain unresolved. A typed verified
failed-effects result is distinct from unknown. Current `anyhow` errors do not
already supply this durable distinction.

Merchant observations are independent of wallet assertions. Commit
`(chain, digest)` single credit across invoices within each merchant operator's
authoritative ledger and explicit invoice association transactionally;
same association returns its existing receipt, conflicting associations stop for
reconciliation. Trust is selected-node inclusion, not a light-client proof.
The simulator uses its separate ledger predicate and never upgrades to live paid.

Keep observed amount/asset and late/partial/overpaid exceptions without erasing
the original payment. A refund is separately authorized intent and payout with
its own destination, artifact and observations; neither an allowance nor a
notification is payout. Never auto-refund to an assumed sender address.
No webhooks are needed now. Later state-to-event delivery requires a durable
barrier plus deduplicating consumers; inspected BTCPay paths do not supply an
inheritable transactional outbox/atomic-refund guarantee.

## Evidence gates

### A — integrated application acceptance

Implemented and locally verified on 28 October 2026; see the dated validation
above. The scenario below is now in `scripts/demo.py`. This closes the original
APDU-ingress/post-commit lost-receipt gap only, not the full target state model.

Run one focused real-process scenario with public fixtures and fresh valid lab
terms. Send SELECT/OPEN/PUT/COMMIT through the actual receiver; feed its exact
persisted output into the actual verifier and explicit approval path. Commit the
artifact, deliver its claim, let the merchant commit, then suppress its receipt.
Restart using the same journals; replay identical BCS, signature and digest.
Independently inspect one merchant simulated payment and wallet receipt association.
Retain simulation labels and `chain_submission: false`. Existing pre-send
`--disconnect-after-journal` does not exercise this post-commit lost-reply boundary.
Do not substitute digest-only comparison or a separate APDU demonstration.

When product code changes, reuse existing checks and add only this focused
acceptance flow where practical; no mocked unit matrix. Documentation research
needs prose/link/scope checks, not builds or payment tests.

### B — real PC/SC path, virtual card

After A, use existing merchant PC/SC with isolated pcscd/vpcd and a thin VICC
adapter to the same receiver. Supply valid ATR, reset/power behavior, u16
big-endian framing and exact bounded reads. Record reader/protocol/config and
transcript. Reset only assembly, not approved artifacts. Inspected vpcd binds
`INADDR_ANY` without authentication: private VM/network namespace with no external
interface, or a reviewed equally private IPC restriction, is mandatory before
startup. Loopback alone is not authentication. PASS proves PC/SC, not RF or phone HCE.

Implementation decision, 28 October: the user's no-external-listener/host-service
constraint is preserved without a VM or TCP. The [bounded Unix-only reverse-mode
patch and pinned source rationale](pcsc-lab.md#private-ipc-decision-and-source-review)
replace vpcd networking with AF_UNIX in 0700 directories, 0600 sockets and mutual
same-UID checks; server mode fails closed. Private pcscd is compiled without
USB discovery/systemd and with private IPC/config paths. The focused real-process
gate passed locally; see [validation](validation.md#private-virtual-pcsc-stage-b-28-october-2026).
Independent read-only review of Stage B completed; see validation. This does not qualify RF,
phone HCE, production custody, power cuts or live settlement.

### C — optional NCI boundary

Use Casimir only for a concrete reset/discovery/routing question. Its inspected
`t4at.active` does not reply to APDUs, standalone Cargo metadata lacks rustutils,
and routing/config stubs remain. Qualify those before claiming integration; no
custom kernel ISO-DEP simulator or AOSP deployment is required for A/B.

### D — physical NFC; E — controlled testnet

D records exact PN7160 or working Sailfish/FuriOS model, kernel/driver/controller,
firmware/HAL, permissions, power/lock/suspend, APDU latency and RF-loss/reselect.
Controller marketing and software ACKs are not that evidence.
E requires separately authorized fresh controlled accounts, exact policy/custody,
address-balance eligibility and submission/reconciliation qualification. Prior
public `0x0` simulation supplies none of that control. Do not impose a blanket
SUI requirement on qualified protocol-gasless USDC.

Hardware and live-chain evidence are independent axes; optional C is no prerequisite
for D/E. Final pilot also needs custody recovery, merchant accounting, independent
security review and willing POS deployment permission. No buys, flashing, services,
accounts, keys, live RPC or funds actions are authorized by this design.
