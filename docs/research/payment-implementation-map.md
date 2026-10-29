# Payment implementation-source map

**28 October 2026 — source research for the architect, not an adoption or security approval.**
Local baseline: `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`.
This supplements the [decision map](README.md), [payments survey](open-payments-reuse.md)
and [transport/security findings](transport-security-notes.md); it does not replace them.
The exact “Open Contactless Payments Initiative” remains unidentified. Cake's actual
Open CryptoPay client below establishes a different, named implementation, not equivalence.

## Recommendation and scope

Keep Linux Tap's Rust invoice policy and pinned Mysten chain engine. Reuse their
existing interfaces; add only the narrow ingress/lifecycle integration needed for:
**emulated APDU invoice → verified terms → explicit approval → journaled signed
bytes → lost merchant receipt/restart → identical replay → one simulated receipt.**
Keep `SIMULATED_PAID_NOT_ON_CHAIN` and `chain_submission: false`; no live submission
or production custody is necessary. The architect owns the eventual design.

Classification: **U** = use module/API unchanged; **A** = bounded native adapter;
**R** = design reference, not imported runtime. Entries describe source-observed
behavior, not independently tested behavior or audited end-to-end guarantees.

| Candidate / source IDs | Class and useful seam | Runtime / license boundary | Decision |
| --- | --- | --- | --- |
| Mysten SDK [M1–M3] | U: types, BCS, intent signing, digest; later execute/checkpoint API | Rust/Tokio/tonic; Apache-2.0 [ML] | Keep pinned engine; no replacement implementation |
| Linux Tap [L1–L4] | U + A: verifier, signer, journal; connect persisted APDU input | Native Rust/SQLite; existing project | Smallest next increment; qualify races below |
| Gem keystore [G1–G6] | U candidate + A: encrypted secret storage behind signer | Rust, Unix file implementation; `core/` MIT [GL] | Later custody review, not needed for public-fixture milestone |
| Gem Sui signer [G7] | R: multi-chain metadata/signing boundary | Rust `core/`; app license is not core license | Do not replace Mysten or accept opaque metadata signing |
| Electrum [E1–E3] | R: preparation, preview, sign, export, broadcast separation | Python/PyQt native UI; MIT [EL] | Borrow interaction boundaries, not Bitcoin wallet engine |
| BTCPay [B1–B7] | R: invoice exceptions, recovery, delivery/refund correlations | C#/ASP.NET/EF and server infrastructure; MIT [BL] | Translate bounded operations concepts; no server dependency |
| Cake Open CryptoPay [C1–C3] | R, optional future A: HTTP quote/address/commit/cancel client | Dart/Flutter and wallet services; MIT root [CL] | Actual ingress example, not APDU or Sui settlement reuse |

License observations do not clear dependencies/assets or redistribution obligations.
No whole-wallet fork, chain-engine substitution or new mandatory backend.
Ledger/Keycard findings remain in the earlier reports; no new hardware claim here.
Taler's different cash/exchange system adds no necessary module to this milestone.

## 1. Local ingress and approval: retain policy outside transport/UI
- [L1] `apdu_stdio` drives `apdu::Receiver`, parses completed bytes as `SignedInvoice`,
  creates a `0600` file with `create_new`, writes and `sync_all`s it before ACK.
  An existing file is accepted only when bounded-read bytes equal the new message.
  Failure maps to `6581`; success explicitly logs `UNVERIFIED_INVOICE_STORED`.
  It does **not** verify merchant identity, checkout terms or payment success.
- Publication caveat: the final path is visible during writing, and the parent
  directory is not synced here. A crash can leave partial input; concurrent readers
  can fail parsing. An ACK is not a power-cut or cross-process publication proof.
  Preserve exact bytes and reject incomplete/conflicting data; see the supplement.
- [L4] `SignedInvoice::verify(expected_key)` uses an independently supplied merchant
  pin. `Invoice::validate_checkout` supplies current time, exact chain and asset policy.
  A display name, URI host or self-presented key must not become the merchant pin.
- [L1] `pay_exchange` currently receives a socket/pipe invoice, not the APDU file.
  Its interactive branch validates and displays chain, full asset, integer amount,
  recipient, invoice hash and fixture approval profile before reading confirmation.
  First-time preparation rechecks checkout after that wait; this matters at expiry.
- Proposed adapter boundary: feed the same persisted envelope into the existing
  verifier/approval path; do not parse a second invoice model in the UI or NFC driver.
  No user prompt, password/KDF work or network wait belongs in an APDU callback.
- Approval must identify the concrete verified terms and signer profile, not merely
  “connected device”. [L3] `PaymentSigner::sign_payment(invoice, transaction)` and
  `ApprovalProfile` are the existing seam. Approval remains a caller responsibility;
  the trait itself is not a user-consent token or a durable approval record.

**Electrum's useful separation [E1–E3].** `pay_onchain_dialog` provides a `make_tx`
closure, calls `confirm_tx_dialog`, returns on cancellation, opens preview separately,
then signs and calls `broadcast_or_show` only from successful signing completion.
`ConfirmTxDialog.run` stops editor updates and returns no transaction when cancelled.
`TxDialog.set_tx` deep-copies the transaction to avoid unrelated main-window updates
destroying signatures; this is UI object isolation, not a complete immutable approval
protocol. It can fetch input information from wallet/network while populating review.
`sign`, `save`, `export_to_file` and `do_broadcast` are distinct operations: complete
transactions export network hex; partial transactions export Bitcoin PSBT bytes.

Borrow explicit review/sign/export/submit actions and clearly labelled offline state.
Do not port PSBT, UTXO selection, fee batching or Bitcoin payment status to Sui.
The inspected send flow saves a pending invoice, not necessarily the exact signed
artifact durably before broadcast. Export uses ordinary file writes, not a journal
durability guarantee. Its own broadcast-status comment asks to move state out of GUI.
Missing proof: restart-safe exact-byte submission and approval binding for our template.

## 2. Exact artifact issuance: what the current journal really guarantees

[L3] `DevelopmentSigner` first calls `verify_template`; `Prepared::sign` requires
sender/key agreement and uses Mysten's `sign_transaction`, transaction digest and BCS.
`Prepared` contains `digest`, `bcs_hex`, `signature`. `Prepared::transaction` bounds
input, decodes BCS, recomputes digest, verifies Ed25519 and binds public key to sender.
`verify_template` rebuilds the complete permitted transaction and requires equality;
it rejects changed commands, recipients, gas and expiration structure. Template
validity is not proof that an account is eligible or that the chain will execute it.

[L2] `Store::open` uses WAL, `synchronous=FULL`, a five-second busy timeout and
`prepared(invoice_hash PRIMARY KEY, value, digest)`. `save_prepared` is a plain INSERT:
the first **committed row** wins; a second INSERT errors instead of replacing bytes.
[L1] first-time flow signs, saves that row, then sends `Claim`; save failure propagates
before send. A later attempt loads and verifies the stored artifact instead of building
another random nonce. Keep this issuance-before-external-effect order.

Important non-guarantees for the architect:

1. Load → generate/sign → INSERT is not a single reservation. Two processes can both
   sign different nonce-bearing artifacts before one INSERT loses. The loser does
   not send on this path, but “only ever signed once” would be false. Do not let a
   future signer/broadcaster bypass the committed artifact or hide uniqueness errors.
2. There is no submission-attempt state, durable work queue, lease, uncertainty marker
   or automatic startup reconciler. Restart means the user reruns the same flow.
3. The receipt table stores invoice hash/digest only, not complete receipt evidence.
   `record_simulated_receipt` uses INSERT OR IGNORE then checks the stored digest;
   conflicting receipts are errors, not silent overwrites.
4. SQLite settings are not filesystem/power-loss qualification. The prior supplement
   identifies bundled SQLite 3.50.2 and a WAL-reset-fix qualification gap. Creation
   mode does not repair an existing permissive file or secure its parent directory.

**Integration obligation, not an architecture prescription:** preserve a canonical
invoice-to-artifact binding and return/reload the winner after a conflict if retry is
supported. Never solve a uniqueness/busy error by silently generating a new transfer.

## 3. Submission uncertainty and reconciliation: reuse Mysten, retain local evidence

[M1] `Client::execute_transaction_and_wait_for_checkpoint(request, timeout)` accepts
an `ExecuteTransactionRequest`, computes the digest from its transaction, subscribes
before execution, and polls subscription, execution and a ledger lookup concurrently.
A checkpointed lookup can answer a duplicate while execution is still in flight.
After an execution error it queries that digest again and can return the committed
transaction instead of the RPC error. It also checks the ledger after execution.
This is useful existing duplicate/race handling; do not implement another RPC engine.

The response contract is narrower than “payment succeeded”:

- `CheckpointTimeout(response)` and `CheckpointStreamError { response, error }`
  retain execution evidence. `RpcError` can still follow an uncertain submission;
  absence from one node's lookup does not prove non-execution or authorize replacement.
- The supplied timeout is applied to the **later checkpoint wait**, not subscription
  setup, execution, probes or the full method. A caller needs a whole-operation
  deadline; cancelling the future does not retract a possibly delivered transaction.
- Lookup read masks preserve the caller's requested fields and always include digest,
  checkpoint and timestamp; the default only adds `effects.status`. Request all fields
  needed by the receipt predicate, especially transaction and effects, rather than
  assuming the helper's `Ok` provides a full independently verified receipt.
- `Ok` establishes its node's checkpoint observation, not successful execution,
  merchant identity or business acceptance. No durable local journal is provided.

Use [L3] `verify_on_node`/`verify_execution` as the existing policy seam: verify signed
artifact/template; compare node chain ID; require matching execution/effects digests,
checkpoint, successful effects and equality of returned transaction to signed input.
It explicitly trusts the selected node for inclusion; it is not a light-client proof.
Node-reported chain identity and transport TLS do not eliminate trust in that node.
Persist evidence separately from UI and reconcile the same digest after interruption.
Keep [testnet qualification](../testnet-qualification.md) as a separate gate: neither
SDK support nor zero gas fields establishes fresh-wallet/address-balance eligibility.

**Current merchant simulation [L1–L2].** `merchant_exchange` verifies the claim's invoice
hash, signed artifact and exact template, checks checkout on a previously unpaid
invoice, commits `accept_simulated`, then sends the receipt. Dropping the reply after
that commit is the relevant unknown-outcome boundary. Existing `--disconnect-after-journal`
returns on the wallet **before sending** and therefore does not exercise it.

`accept_simulated` reads then inserts inside a deferred SQLite transaction. Primary
key on invoice and unique digest prevent conflicting committed rows; same invoice/
digest returns replay, different digest requires reconciliation. Concurrent first
writers can encounter busy/snapshot/uniqueness errors rather than both getting an
idempotent success. A five-second busy timeout is not an automatic transaction retry.
The normal merchant loop is serial; multi-process race handling is not demonstrated.
Digest equality is the replay predicate, not a stored raw-byte comparison. The
acceptance scenario should independently compare full BCS/signature as well as rows.

## 4. Merchant operations: BTCPay concepts, not an inherited outbox guarantee

[B1] `InvoiceWatcher.UpdateInvoice` separates `New`, `Processing`, `Expired` and
exceptions including `PaidPartial`, `PaidOver`, `PaidLate`. Expiry does not erase a
late observed payment. `StartAsync` calls `WaitPendingInvoices`, which queries persisted
pending invoice rows and schedules observation again. This is concrete startup
reconstruction from storage, not dependence solely on an in-memory timer.

But `StartLoop` calls [B2] `UpdateInvoiceStatus` before publishing events through
`EventAggregator`; price update is another operation. That repository method is an
SQL UPDATE, not a transaction also inserting an event/outbox record. A crash between
update and publication can lose the transition notification. Rescanning pending
state is not proof of regenerating every historical event, especially terminal ones.

[B3–B4] webhook provider creates delivery IDs and enqueues in memory. Sender serializes
per webhook, HMAC-signs HTTP requests and records result/history. Automatic retry
uses in-process delays; `StartAsync` is a no-op, not a persisted queue replay scan.
`SendAndSaveDelivery` awaits HTTP **before** `AddWebhookDelivery`: a crash can occur
after remote effect but before local history. A queued unsent event can also be lost.
Persisted manual redelivery is real: it loads an existing delivery, assigns a new
delivery ID and preserves `OriginalDeliveryId`. It cannot recover a never-saved one.
Therefore this path is neither a transactional durable outbox nor exactly-once HTTP.

Borrow stable invoice/payment/delivery correlation, exception visibility, authenticated
notifications and restart observation. If downstream notification becomes necessary,
the architect must explicitly close the state/event persistence gap and require
idempotent consumers. Notification delivery is not settlement; keep it outside the
merchant's authoritative payment decision. No webhook system is required for next demo.

[B5–B7] `CreateRefundPullPayment` first saves a new pull payment, then saves a
`RefundData` association, then publishes an invoice refund event. Association key is
`(InvoiceDataId, PullPaymentDataId)`, not a unique original-payment refund command.
`CreatePullPaymentCore` sets currency, positive limit, start/end, payout methods and
claim-approval policy. A refund request/claim allowance is not a completed payout.
Separate saves leave an orphan window; retry can create another allowance. Do not
infer atomic refund creation or deduplicated retries from that composite key.

Translate only the needed accounting distinctions: original invoice, accepted
payment/digest, observed amount/asset, exception and a separately authorized refund
intent/payout. Do not overwrite the original payment with “refunded”, reuse the
original signature or automatically return funds to an unverified supplied address.
Refund identity, amounts, authorization, execution and reconciliation remain future
work; this report does not qualify BTCPay's Bitcoin/Lightning handlers for Sui.

## 5. Actual Open CryptoPay wallet ingress: Cake's concrete HTTP client

[C1] `OpenCryptoPayService` accepts LNURL directly or a `lightning` query parameter,
fetches provider data, checks `callback`, `transferAmounts`, `quote`, then builds
`OpenCryptoPayRequest(receiverName, expiry, callbackUrl, quote, methods)` [C2].
Expiry becomes relative seconds; assets are symbol/decimal-string amounts, not our
pinned chain/full Move type/base-unit schema. It does not filter an `available` flag
in the inspected methods/assets loop and logs the complete quote response.
`isOpenCryptoPayQR` is a loose prefix/substring detector, not validation or trust.

`getOpenCryptoPayAddress` GETs callback with quote/asset/method over constructed HTTPS,
requires `expiryDate` and `uri`, but does not compare that returned expiry with now.
A returned URI already containing `amount` is returned unchanged; otherwise quoted
amount is inserted and the rebuilt path is split at `@`. Do not transplant generic
URI rewriting into exact Sui asset/network/recipient validation. There is no merchant
signature/pin verification in these inspected service methods; provider data drives
display and address selection. Callback/redirect/response-size policy needs its own
review; the service alone is not evidence of restrictive SSRF or host-binding rules.

[C3] `SendViewModel._handleOcpRequest` first calls `pendingTransaction.commit()` for
XMR/Zano/SOL and SOL-tagged assets; then calls `commitOpenCryptoPayRequest` for every
asset. For the others this method sends signed hex/tx ID to the provider rather
than invoking that local commit here. The HTTP commit rewrites `/cb/` to `/tx/` and
GET query includes quote/asset/method/tx and optional hex; HTTP 200 with a `txId` key
is accepted. The returned ID is not compared here with the pending transaction ID.
This is a provider handoff, not independent settlement verification; signed bytes
in query strings also raise access-log/proxy retention concerns.

Client-commit success followed by provider failure is an obvious uncertain-outcome
window. The shown `ocpRequest`/pending-transaction flow supplies no durable
quote-to-signed-artifact recovery journal; wider wallet persistence was not audited.
`cancelOpenCryptoPayRequest` rewrites `/cb/` to `/cancel/`, DELETEs and ignores HTTP
status; `dismissTransaction` invokes cancellation without awaiting it. Cancellation cannot be
interpreted as invalidating previously signed or broadcast bytes.

Borrow the explicit quote → method/asset → destination → pending transaction →
commit/cancel stages only. A future Rust adapter would emit an **untrusted request**
into existing policy, preserve provider/quote correlation and keep provider ACK
separate from verified payment. No Dart/Flutter dependency or OCP adapter is needed
for APDU simulation. No evidence here establishes OCP Sui/address-balance support.

## 6. Custody boundary: Gem is separable, not automatically qualified

[G1] `Keystore` exposes import/decrypt/verify/change-password/meta/list/delete;
decrypt returns `Zeroizing<String>` or `Zeroizing<Vec<u8>>`. [G2–G5] `FileKeystore`
uses encrypted v4 payloads, a default Argon2id KDF and AES-256-GCM with authenticated
header bytes. Unix writes use unique `0600` create-new temp files, file sync, rename
and directory sync. `storage` is a selectable feature [G6]; Linux does not require
embedding the Gem mobile UI. Non-Unix owner-only file creation explicitly errors.

Limits to carry into any adoption review:

- `queue.rs` is a process-local `OnceLock<Mutex<()>>`. Existence check followed by
  ordinary rename is not cross-process no-clobber publication; competing processes
  can replace the same destination. Directory creation does not explicitly set `0700`.
- Import of an existing caller-supplied ID verifies the old file/password and returns
  metadata without comparing the newly requested secret. Treat that as “existing
  authenticated entry”, not proof that this particular secret was imported.
- `get_meta`/`list` parse metadata without decrypting/authenticating ciphertext;
  use `verify`/decrypt before trusting it for a custody operation. Password rotation
  decrypts and replaces the file; recovery/backup/concurrent-operation policy is ours.
- Zeroizing return values do not prove absence of plaintext copies, secure UI/input,
  anti-swap protection, hardware isolation, backup integrity or recovery usability.
- [G7] `SuiChainSigner` signs a digest from `SignerInput.metadata` encoded as
  `prefix_digesthex` and returns `prefix_signature`. This interface does not verify
  merchant terms or our exact transaction. Keep Mysten's typed signing and [L3]
  policy; do not expose arbitrary metadata signing as approved payment signing.

Candidate seam: a reviewed native custody adapter behind `PaymentSigner`, with
explicit unlock/approval lifetime, address derivation and secret disposal. Reuse
storage/crypto rather than copy encryption code. Cross-process exclusion, private
directory policy, import identity and recovery need proof before production adoption.

## 7. Expiry, cancellation and acceptance obligations

Invoice deadline, quote deadline and signed transaction epoch bounds are different.
[L1] a fresh wallet preparation checks checkout again after approval; loading an
existing prepared artifact skips that second check. Interactive mode checks earlier
and so can block retry of an expired invoice, while noninteractive replay may proceed.
Merchant checks checkout only before the first recorded payment and permits a known
same-digest replay after expiry. Define retry/reconciliation separately from permission
to issue a **new** artifact; do not blindly apply fresh-invoice rules to unknown outcomes.
Electrum [E1] checks invoice expiry again in the broadcast thread; that rule is a
Bitcoin UI example, not proof that expiry cancels a submitted Sui transaction.

Before signing, cancellation can stop issuance. After signing, keep the artifact;
after possible submission, report unresolved until authoritative evidence permits a
terminal conclusion. Closing UI, RF loss, DELETE success or a timeout is not reversal.
Next acceptance must use valid laboratory terms, suppress reply after merchant commit,
restart, replay identical BCS/signature, and inspect one merchant payment and wallet
receipt association. Include expiry and storage errors as explicit outcomes, not
silent replacement. No new chain engine, production key or live RPC is required.

## Immutable source register
Repository origins: [Mysten](https://github.com/MystenLabs/sui-rust-sdk),
[Gem](https://github.com/gemwalletcom/wallet), [Electrum](https://github.com/spesmilo/electrum),
[BTCPay](https://github.com/btcpayserver/btcpayserver), [Cake](https://github.com/cake-tech/cake_wallet).
Every upstream link below pins the full inspected revision; paths are extraction
boundaries, not permission to copy a whole repository. Local paths refer to baseline
source and are relative to this document. Line anchors aid lookup; symbols govern.

- **L1:** [CLI](../../crates/tap-cli/src/main.rs) — `apdu_stdio`, `pay_exchange`, `merchant_exchange`, `prepare`.
- **L2:** [Store](../../crates/tap-cli/src/store.rs) — `open`, `save_prepared`, `accept_simulated`, `record_simulated_receipt`.
- **L3:** [Sui adapter](../../crates/tap-sui/src/lib.rs) — `PaymentSigner`, `Prepared`, `verify_template`, `verify_execution`, `verify_on_node`.
- **L4:** [Invoice policy](../../crates/tap-core/src/lib.rs) and [pins](../../deps/upstreams.lock.json) — `SignedInvoice::verify`, `Invoice::validate_checkout`.
- **M1:** [transaction_execution.rs](https://github.com/MystenLabs/sui-rust-sdk/blob/1313d5112464ecc25d7407f42442adea08589fc5/crates/sui-rpc/src/client/transaction_execution.rs#L98-L303) — execute/wait, error evidence, concurrent duplicate lookup and timeout scope.
- **M2:** [sui-crypto/lib.rs](https://github.com/MystenLabs/sui-rust-sdk/blob/1313d5112464ecc25d7407f42442adea08589fc5/crates/sui-crypto/src/lib.rs) — `SuiSigner`, `SuiVerifier`.
- **M3:** [Local SDK dependency usage](../../crates/tap-sui/Cargo.toml) — actual engine integration, not a separate wallet SDK proposal.
- **ML:** [Mysten LICENSE](https://github.com/MystenLabs/sui-rust-sdk/blob/1313d5112464ecc25d7407f42442adea08589fc5/LICENSE).
- **G1:** [keystore.rs](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/src/storage/keystore.rs) — `Keystore` contract.
- **G2:** [file_keystore.rs](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/src/storage/file_keystore.rs#L92-L155) — import identity, verification and publication; same file implements metadata/rotation/deletion.
- **G3:** [queue.rs](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/src/storage/queue.rs) — process-local lock.
- **G4:** [file_io.rs](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/src/storage/file_io.rs) — capped reads, Unix mode and directory sync.
- **G5:** [secret.rs](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/src/storage/secret.rs) — encryption/decryption and header authentication.
- **G6:** [gem_keystore/Cargo.toml](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/Cargo.toml) — storage feature and dependency surface.
- **G7:** [chain_signer.rs](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_sui/src/signer/chain_signer.rs#L44-L56) — metadata/digest signing boundary.
- **GL:** [Gem core/LICENSE](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/LICENSE) — MIT subtree, not the top-level GPL application license.
- **E1:** [send_tab.py](https://github.com/spesmilo/electrum/blob/bf9447ccbf78509c2b2ef8b5871cea990926ce48/electrum/gui/qt/send_tab.py#L292-L361) — prepare/preview/sign; `broadcast_transaction` at lines 784–834.
- **E2:** [confirm_tx_dialog.py](https://github.com/spesmilo/electrum/blob/bf9447ccbf78509c2b2ef8b5871cea990926ce48/electrum/gui/qt/confirm_tx_dialog.py#L608-L626) — run/cancel/preview.
- **E3:** [transaction_dialog.py](https://github.com/spesmilo/electrum/blob/bf9447ccbf78509c2b2ef8b5871cea990926ce48/electrum/gui/qt/transaction_dialog.py#L597-L629) — review copy; sign/save/export at lines 715–779.
- **EL:** [Electrum LICENCE](https://github.com/spesmilo/electrum/blob/bf9447ccbf78509c2b2ef8b5871cea990926ce48/LICENCE).
- **B1:** [InvoiceWatcher.cs](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer/HostedServices/InvoiceWatcher.cs) — `UpdateInvoice`, `WaitPendingInvoices`, `StartLoop`.
- **B2:** [InvoiceRepository.cs](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer/Services/Invoices/InvoiceRepository.cs#L452-L463) — `UpdateInvoiceStatus`.
- **B3:** [WebhookProviderHostedService.cs](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer/Plugins/Webhooks/HostedServices/WebhookProviderHostedService.cs#L37-L62) — event-to-delivery enqueue.
- **B4:** [WebhookSender.cs](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer/Plugins/Webhooks/WebhookSender.cs) — `StartAsync`, `Process`, `CreateRedeliveryRequest`, `SendAndSaveDelivery`.
- **B5:** [PullPaymentHostedService.cs](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer/HostedServices/PullPaymentHostedService.cs#L103-L162) — refund and allowance creation.
- **B6:** [RefundData.cs](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer.Data/Data/RefundData.cs) — invoice/pull-payment association key.
- **B7:** [PullPaymentData.cs](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer.Data/Data/PullPaymentData.cs) — persisted allowance, not payment finality.
- **BL:** [BTCPay LICENSE](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/LICENSE).
- **C1:** [open_cryptopay_service.dart](https://github.com/cake-tech/cake_wallet/blob/de159ddb479aff0e67fda2081a68f1c4a944077f/lib/core/open_crypto_pay/open_cryptopay_service.dart) — ingress/address/commit/cancel implementation.
- **C2:** [models.dart](https://github.com/cake-tech/cake_wallet/blob/de159ddb479aff0e67fda2081a68f1c4a944077f/lib/core/open_crypto_pay/models.dart) — request/asset contract.
- **C3:** [send_view_model.dart](https://github.com/cake-tech/cake_wallet/blob/de159ddb479aff0e67fda2081a68f1c4a944077f/lib/view_model/send/send_view_model.dart#L1068-L1079) — actual client/provider commit ordering; `dismissTransaction` cancellation at lines 595–605.
- **CL:** [Cake LICENSE.md](https://github.com/cake-tech/cake_wallet/blob/de159ddb479aff0e67fda2081a68f1c4a944077f/LICENSE.md).

## Evidence limits and handoff

Validated local links, whitespace, final newline and read-only Git scope. Only this
new report was written. No installs, builds, tests, services, emulators, accounts,
keys, chain RPC, faucets or funds; owned runtimes: **none**. Missing proof: integrated
lost-receipt acceptance, concurrent writer recovery, power-loss durability, custody/
recovery, trusted display, physical NFC and qualified live settlement. These remain
later gates, not reasons to enlarge the simulation milestone.
