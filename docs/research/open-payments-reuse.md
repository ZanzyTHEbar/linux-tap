# Open contactless payments: identity, source audit and reuse plan

Research date: **28 October 2026**. Local baseline: clean `main`,
`f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`. This is a research report, not a
protocol endorsement, security audit, hardware qualification or payment trial.
Only this report was added. No dependencies were installed, applications built,
services started, live chain RPCs called, accounts provisioned, keys generated,
funds requested or transactions submitted.

## Conclusions

1. **The exact name “Open Contactless Payments Initiative” remains unresolved.**
   No authoritative site, organization, specification or package was established
   under that exact name. Do not silently substitute Interledger Open Payments,
   Open Payment Initiative (OPI), Open Charge Point Interface (OCPI), EMV, or an
   Ethereum/Solana project. An original URL or author is needed to resolve identity.
2. **Open CryptoPay is a real, relevant but differently named candidate.** Its
   official site links to an MIT-licensed HTTP/LNURL payment-negotiation document.
   The examined specification is not an APDU transport specification and does not
   define Linux Tap's signed merchant invoice or durable exact-byte journal.
   Its newly named `api` repository is a **merchant map API**, not the settlement
   backend implied by its name. Do not fork it expecting a payment processor.
3. **Reuse the existing Mysten Rust stack.** The pinned SDK revision `1313d511…`
   is still upstream's default-branch head observed today. Important duplicate
   execution/checkpoint and transparent transport retry work is already in that
   source. The offline empty-gas builder gap is still present; no newer fix was
   verified. Keep the narrow SDK-type adapter rather than replacing it with JS.
4. **There is more native custody reuse than the earlier audit suggests.** Gem's
   app README says GPL-3.0, but its current `core/LICENSE` is MIT and its Rust
   workspace declares MIT. `gem_keystore` is a concrete portable encrypted-file
   candidate. This is not permission to assume every monorepo file is MIT, nor
   proof of production custody or a Linux wallet UI.
5. **Ledger support needs transaction-shape qualification, not a brand checkbox.**
   Current Sui app source supports SIP-58, but its `balance::redeem_funds` clear
   parser expects `Argument::Result(FundsWithdrawalSplit)`. Linux Tap passes a
   withdrawal input directly. Existing SUI/address-balance emulator tests do not
   prove clear signing of our two-call, zero-price USDC transaction.
6. **Do not expand the immediate milestone.** Use the existing Rust verification,
   signing and SQLite paths for emulated APDU invoice → explicit approval →
   interrupted/lost-ACK recovery → identical signed bytes → one simulated receipt.
   Protocol adoption, custody, Qt, live settlement and RF qualification are
   separate decisions, not prerequisites for this offline demonstration.

## Evidence and discovery limits

The required `AGENTS.md`, `README.md`, `docs/{reuse,architecture,protocol,
testnet-qualification,backlog,demo}.md` and `deps/upstreams.lock.json` were read,
along with the bundled Sui SDK/Rust/gRPC guidance. Relevant local source was
checked, including `tap-core/src/apdu.rs`, `tap-sui/src/lib.rs`, and
`tap-cli/src/{main,store}.rs`. Local paths below refer to the baseline above.

Public research used fetched primary pages, raw source, GitHub repository trees
and GitHub API metadata. Search snippets were discovery aids, not implementation
or support evidence. Unauthenticated GitHub API requests eventually hit their
rate limit; raw source and public HTML remained available. Not every candidate
could receive a release-by-release audit. Mutable documentation links describe
what was fetched on the research date; commit links identify frozen evidence.
No remote test suite was executed: some upstream suites create/cancel invoices
or expect signing devices and would exceed read-only research scope.

### Exact initiative discovery

Searches included the exact plural phrase, “Open Contactless”, “Open Contactless
Initiative”, “contactless payment initiative”, OCP/NFC/crypto/APDU variants, and
Ethereum, Solana and Sui combinations. GitHub repository searches for
`"open contactless"`, `"contactless payments" initiative`, and the exact
initiative wording in READMEs did not identify the named organization. This is
**an unsuccessful identification, not proof that the initiative does not exist**.

| Candidate | Primary evidence actually found | Identity/fit decision |
| --- | --- | --- |
| Open CryptoPay | Official `opencryptopay.io` links its `openCryptoPay/landingPage` documentation and DFX integrations [O1–O5] | Closest verified multi-rail checkout candidate; different name, no proof it is what the user meant |
| Ethereum Tap to Pay / proposed ERC-7798 | `ethereum/ERCs` PR #686 contains a draft specification [E1] | Real draft proposal; EVM/provider/URI semantics, not a verified initiative with the requested name |
| Solana mobile payments | Official Solana Mobile Stack announcement describes Android NFC capture of Solana Pay URLs [S1] | Different ecosystem and custody/platform assumptions; announcement is not APDU implementation proof |
| Interledger Open Payments | Official site/repository describe account-servicing APIs and GNAP [I1–I2] | Not a contactless wire protocol; reject identity conflation |
| OCPI | Maintainer repository explicitly expands name to **Open Charge Point Interface**, with EV-roaming specifications [D1] | EV charging; even a Payments module does not make this the requested NFC initiative |
| OPI | Search located Open Payment Initiative terminology and Oracle's separately named **Oracle Payment Interface** [D2] | No authoritative source connecting either to the requested initiative; do not treat names/acronyms as interchangeable |
| EMV / open-loop transit | AID/kernel/card-network acceptance is a different deployment and certification model | Not supplied by an open-source APDU receiver or a crypto wallet; no compatibility claim |
| Open Contactless Application Platform / CIPURSE | Search surfaced a historical platform announcement | Different name and transit/application model; not promoted to an identified crypto initiative or reusable library without primary implementation evidence |

**Unresolved contract:** the missing original initiative URL and the unavailable
final engineering specification/foundation charter prevent full requirements
acceptance. They do not prevent the current simulation milestone. Keep any
compatibility statement provisional until the actual initiative is identified.

## 1. Open CryptoPay: what can actually be reused?

### Provenance and source inventory

The official site explicitly directs developers to the `openCryptoPay`
organization's landing-page README and to `DFXswiss/services` for merchant API
integration. Reviewed revisions returned by GitHub on this date:

| Repository | Revision observed | What it contains / runtime | License evidence |
| --- | --- | --- | --- |
| `openCryptoPay/landingPage` | `e97197a76793e60c5ba23b8a34998c69c304865a`, 27 Oct 2026 | Website plus wallet integration specification in README | Actual `LICENSE`: MIT [O2–O3] |
| `openCryptoPay/api` | `c9d36b959d96e59455960f45b9c1b4fcfb55809f`, 27 Oct 2026 | Bun/TypeScript merchant-place map, SQLite, shop/photo/admin ingestion API | GitHub license metadata: MIT; verify full dependency/file licenses before copying [O4] |
| `openCryptoPay/tests` | `01410bd6cb78c56dc0385a7d7cc3c7b678a7bde0`, 9 Sep 2026 | Node 22 HTTP client tests against DFX dev/prod endpoints | README and metadata: MIT [O5] |

The map API's Ed25519 admin request signing, Nostr shop identities and ingestion
replay rules are **not** the wallet payment authentication scheme. They must not
be cited as evidence that an OCP checkout invoice is merchant-signed.
The tests explicitly exclude completing real onchain payment and create/cancel
small fiat-denominated invoices when run. They were inspected, not run.

### Protocol-level findings

The fetched README [O2] specifies this flow:

1. A static cash-register QR URL carries a `lightning` query parameter containing
   an LNURL/bech32 API URL. A wallet decodes it and fetches pending payment data.
   Anyone may offer a provider under their own domain; DFX is the example, not a
   protocol-mandated central operator.
2. The provider returns an ID, display/recipient information, an expiring quote,
   payment ID, available methods/assets/amounts, minimum fees and callback URL.
   No pending payment waits up to ten seconds by default, then yields HTTP 404.
3. The wallet selects quote/method/asset and fetches transaction details. These
   are generally a chain URI and expiry, or a Lightning invoice. A simplified
   single-request flow is also defined.
4. The wallet constructs and authorizes the actual rail operation. Depending on
   the method it sends a raw signed transaction to the provider, or broadcasts
   itself and returns a transaction identifier. The provider associates payment
   with checkout and handles its merchant-side booking/settlement relationship.

| Question | Specification/source result | Linux Tap consequence |
| --- | --- | --- |
| AID, SELECT/APDU framing, reader/card role? | No APDU AID, SELECT sequence, chunking or ISO-DEP state machine is defined in the examined wallet spec. Official marketing mentions NFC; the POS-device section also says under development [O1–O2]. | OCP is not a drop-in replacement for `F04C54415000`. A URL via NFC is a separate transport profile, not proof of shared APDUs or existing-terminal acceptance. |
| Invoice authenticity? | Provider-supplied JSON/URI with merchant fields; no detached merchant invoice signature or independent key-pin scheme specified there. | HTTPS/provider origin and authentication policy matter. Do not equate `displayName`, registration number, static QR placement or map identity with a verified merchant. |
| Explicit user approval? | Site says shopper confirms; spec tells wallet to construct/sign. No trusted-display API or approval transcript supplied. | Retain the native policy/approval boundary and exact template verification; never accept arbitrary callback instructions as authority to sign. |
| Rail assumptions? | Examples cover Lightning, Bitcoin/Firo/EVM, Monero/Zano, Solana, Tron/Cardano, Spark, Internet Computer and BinancePay. Methods are provider-advertised and may be unavailable. | No Sui method or Sui gasless template was established in this specification. “All blockchains” marketing is extensibility, not working Sui support. |
| Submission/return path? | EVM/Bitcoin/Firo: provider receives raw signed `hex` via HTTP GET, then broadcasts. Several other chains: wallet broadcasts and reports `tx`. ICP can authorize `icrc2_approve` and report `sender`; provider pulls funds. Lightning/BinancePay use different completion paths. | Transport and custody differ by rail. Do not transplant an allowance/pull model into Sui or log reusable signed authorizations in URLs. |
| Fees? | `minFee` is gas price in wei for EVM, sat/vB for Bitcoin; ICP approval covers transfer fee and itself costs a fee. | Not Sui gasless eligibility. A quote amount, merchant service fee, network fee and conversion spread are distinct terms needing explicit presentation. |
| Sessions, expiry, ACK? | Quote/payment IDs, quote expiry, HTTP status and some method-specific retry/booking rules. Not an APDU-session ACK. | HTTP success is a provider assertion in that integration, not independent chain finality. Persist our approved bytes and reconcile separately. |
| Refunds? | Optional/mandatory `payerData.refund` per method, wallet-owned return destination; conditions handle expired/wrong/duplicate/failed-route funds. Warns not to infer payer from onchain sender. | Useful operational requirements to adopt. No Sui refund method or Linux-native implementation proved. |
| Unknown outcome? | Spark explicitly says report the **same transfer ID**, not a second transfer, on retry. Other methods' HTTP completion language does not supply a general crash-safe wallet journal. | Reuse the principle, but preserve Linux Tap's first-signed-bytes-wins storage. Never fetch a new quote and silently pay again after uncertainty. |

**Reuse/adapt:** adopt expiring quote selection, method availability, provider
versus wallet broadcast distinction, refund destination collection and exception
accounting as requirements. A future optional OCP HTTP adapter can translate a
verified supported method into a native typed request. Keep URL fetching outside
invoice verification/signing, constrain origins/redirects and response sizes, and
avoid logging query-carried transaction material or payer metadata.

**Reject for this milestone:** importing Bun/Node, assuming a ready Sui backend,
replacing the locally authenticated CBOR invoice with unsigned display JSON, or
using the merchant map API as a payment engine. No maintained OCP Rust APDU or
Sui settlement package was verified. Do not invent one from an organization name.

## 2. Ethereum and Solana: bounded comparisons, not assumed identity

### Ethereum proposed Tap to Pay

The official `ethereum/ERCs` PR #686 file diff [E1] proposes ERC-7798, status
**Draft**, created 25 October 2024, requiring ERC-20, ERC-681 and EIP-712. The PR
page surfaced as closed; the guessed current official `ERCS/erc-7798.md` raw
path returned 404 during this research. That does not establish a finalized or
currently adopted ERC. The examined source is the PR diff, not an accepted
interoperability certification. Its copyright section waives rights via CC0.

The proposal defines `requestContactlessPayment` on the EIP-1193 JavaScript
provider API, URI payload types and an optional HTTP relayer for larger payloads.
In its merchant example **the merchant device emulates a tag/card and the
customer wallet reads it**—the reverse of Linux Tap's merchant-reader/wallet-card
profile. Payloads permit simple ERC-681 sends, arbitrary contract calls with
approval transactions, or EIP-712 messages plus a submission endpoint. An
optional verification code limits casual public retrieval; it is not merchant
identity authentication. Transaction hashes or signed messages return over HTTP.

No AID, SELECT exchange, APDU chunk/session/ACK protocol, durable restart journal,
native Linux package or Sui mapping was established in that draft. Reuse the
transport-independent URI/intents separation as a comparison, not arbitrary EVM
calldata handling. A Rust parser could implement a URI without running JS, but
that would not make the proposed EVM wallet contract a Sui protocol.

### Solana

Solana's own 2022 Mobile Stack announcement [S1] describes Android libraries,
Mobile Wallet Adapter, Seed Vault, and NFC/QR capture of Solana Pay URLs. It
does not establish Linux NFC listen mode, a portable Sui signer or universal
secure-element support. Candidate Sui NFC hackathon search hits were not
promoted to standards or libraries without primary code evidence.

There is also a current-source hazard: following
`solana-foundation/solana-pay` redirected to `solana-foundation/pay`, whose
current README describes a Rust/TypeScript **agentic HTTP x402/MPP payment CLI**
with local wallet approval, not the historical NFC APDU protocol [S2]. It may
provide useful future custody/approval reference code, but redirect continuity
does not prove unchanged protocol scope. Do not cite its present source as
implementation of the old NFC announcement, nor copy unattended agent spending
permissions into an explicitly approved checkout flow.

## 3. Local specification versus actual implementation

| Boundary | Draft promise | Source evidence at baseline | Gap / required proof |
| --- | --- | --- | --- |
| Merchant identity | Signed invoice plus independently expected key | `SignedInvoice` verification; CLI pins a public lab key | Production trust enrollment/revocation absent; label is not business identity |
| Invoice encoding | Fixed ten-field canonical CBOR, chain/type/recipient/amount/expiry bound | `tap-core/src/lib.rs`; domain-separated signing; strict re-encoding | Independent implementation/golden vectors and full missing v0.2 requirements not certified |
| Application selection | Private lab AID `F04C54415000` | `apdu.rs`: exact five-byte short-APDU header plus `Lc` bytes; SELECT resets receiver and returns version 0 | No public AID allocation, PPSE/EMV selection, extended APDU or timing qualification |
| Reader/card role | Merchant pushes invoice to wallet card application | `frames()` sends SELECT/OPEN/PUT/COMMIT; receiver accumulates invoice | Reverses ERC proposal's merchant-HCE model; PC/SC reader is not a card-emulation API |
| Session/ACK | Length/hash/open, exact repeated chunks, repeated COMMIT | 8,192-byte cap; 220-byte chunks; offsets bounded; changed/out-of-order chunks abort; `9000` success, `6A80` abort | Session is transient transport state, not payment authorization; RF field-loss behavior needs device integration evidence |
| Persistence of incoming NFC request | Bridge stores unverified invoice; `6581` persistence failure | Rust receiver plus nfcd glue, not a signer | COMMIT hash authenticates bytes only, not merchant or payment; card-side integration must pass those exact bytes into verification |
| Approval | Show exact terms, then sign only approved template | `pay_exchange()` verifies invoice, displays full terms and requires `APPROVE SIMULATION`; `DevelopmentSigner` rebuilds template | Host CLI approval only, public fixture key; no isolated trusted UI |
| Transaction | Sender address-balance withdrawal → redeem → send, zero gas, chain/epoch/nonce | `tap-sui/src/lib.rs:35–88`; exact reconstruction at `95–113` | Eligibility and live policy distinct; alternate Circle testnet qualification is not signer policy |
| Signed artifact durability | Store bytes before any claim | `main.rs:365–400`; `store.rs` INSERT into invoice-hash primary key; WAL/FULL synchronous | Stores serialized `Prepared`, including BCS/signature/digest; not a secure keystore. Concurrent/multi-wallet/onchain invoice uniqueness not supplied |
| Replay/receipt | First payment wins; same digest returns replay | `accept_simulated()` transaction + unique invoice/digest; conflict rejected; merchant persists before sending receipt | Local simulated idempotency, not live chain settlement or independently signed merchant receipt |
| Restart/lost ACK | Resume exact existing artifact | Wallet loads prepared row before signing; merchant replay path returns existing digest | Demo deliberately interrupts **before send**. A lost receipt after merchant commit is supported by code structure but not demonstrated by the documented guided scenario |
| APDU-to-wallet end-to-end | Immediate milestone asks APDU-delivered invoice to approval to restart | Existing demo separately reconstructs APDUs; normal `pay_exchange()` receives invoice on pipe/socket | Do not present separate reconstruction as an integrated APDU ingress approval proof. Wire through existing verification in the next focused increment |
| Native UI and RF | Future Qt and tested adapter | No UI; PC/SC compiled, nfcd hardware adapter required | No physical NFC, Omarchy-host acceptance, commercial POS or hardware signer proof |

The simulator is correctly labelled and has no broadcast command. Preserve that
separation. In particular, `verify_on_node` is a selected-node trust predicate;
it is not a cryptographically verified light client. The stored receipt's status
string is intentionally `SIMULATED_PAID_NOT_ON_CHAIN`.

## 4. Mysten Rust: retain and reuse more of the pinned implementation

### Version and update assessment

The GitHub repository's default branch is **`master`**, not `main`.
The commits endpoint returned `1313d5112464ecc25d7407f42442adea08589fc5` as its
head (15 October 2026), exactly the local pin [M1]. A compare against guessed
`main` returned 404; that is not evidence of a newer revision or removal.
Fetched `sui-sdk-types` and `sui-rpc` manifests both declare `0.4.0` and
Apache-2.0 [M2]. This verifies source versions, not that every crate release
on crates.io includes every subsequent head commit. No dependency update is
recommended solely because the research date is later than the pin.

Relevant changes already included in the inspected history:

- `1313d511…`: additional `INSUFFICIENT_GAS_BALANCE` and
  `TRANSACTION_EXECUTION_FAILED` RPC reasons.
- `db79f343…`: transparent retry **only when HTTP/2 evidence establishes a request
  was not processed** (bounded refused-stream/GOAWAY or pre-write cancellation
  cases), preserving per-call deadlines [M3]. This is not authorization to
  repeat possibly processed requests with new signed bytes.
- `2c5db7c…`: rejects invalid legacy multisig bitmaps during protobuf conversion;
  new checked API avoids panic/inconsistent signer sets. Relevant to future
  signature expansion, not a reason to enable multisig in today's Ed25519-only
  acceptance predicate.
- `0d663362…`: shared private-key decoding rejects invalid secp256 scalars rather
  than panicking. Relevant to future import paths, not fixture custody proof.

### Builder and settlement reuse

The pinned `builder.rs` [M4] already offers native `funds_withdrawal`,
`funds_withdrawal_balance`, `funds_withdrawal_coin` and balance/coin intents.
However, `try_build()` still returns `MissingGasObjects` for empty gas before
considering price/budget. Async `build()` constructs a simulation request with
`do_gas_selection(true)` and returns the resolved transaction. It is not a
drop-in exact-byte offline gasless builder. Do not add a fake gas object, accept
a mutated simulated transaction after approval, or recreate chain serialization.
Keep our small direct SDK-type construction and upstreamable reproducer.

The **existing pinned**
`Client::execute_transaction_and_wait_for_checkpoint()` [M5] is a valuable reuse
target for later authorized submission. It:

- computes the request digest using SDK transaction types;
- subscribes before executing and polls checkpoint scanning concurrently;
- probes `GetTransaction` during execution to handle already committed replays;
- rechecks the ledger after execution errors, avoiding false failure on races;
- returns distinct checkpoint timeout/stream-error variants, including the
  execution response where available;
- populates checkpoint/digest/timestamp in duplicate lookup responses, while
  respecting the caller's read mask.

**Integration conditions:** journal exact BCS/signature/digest first; never
rebuild on a timeout; request the full fields needed by `verify_execution`, not
just the helper's default `effects.status`; independently check chosen node's
chain, exact transaction, successful effects and checkpoint. The helper is not a
wallet database, merchant invoice association, global invoice deduplicator or
light-client proof. Its `timeout` is used for checkpoint waiting after the
execution phase; keep an explicit outer operation deadline rather than assume
it bounds every network await. An RPC or checkpoint error after sending is
**unknown/pending**, not permission for fresh signing.

### Gasless/address-balance constraints

Current official Sui documentation [M6] explicitly allows qualified stablecoin
transfers **without holding SUI**. It requires protocol-allowlisted token types
and operation shapes, at least 0.01 token, empty gas payment and zero gas price;
the examples also set zero budget. It excludes arbitrary Move writes/app calls
and prioritizes paid transactions during congestion. Its automatic detection
example is TypeScript gRPC/GraphQL behavior, not evidence that the Rust offline
builder has the same policy/resolution support.

Crucially, official docs now mention eligible coin-to-address-balance helpers
as well as address-balance-only flows. Therefore do not claim *all* initial
conversion necessarily requires SUI; equally, do not claim Linux Tap already
supports those shapes. Our exact two-call template assumes pre-existing address
balance and does not consume coin objects. Conversion needs its own reviewed
shape and qualification, not silent mutation of the approved payment.

Existing local read-only evidence remains the correct boundary: mainnet USDC
type fails on testnet; explicit Circle testnet type simulated using a
pre-existing funded third-party public address. Neither establishes a controlled
fresh-wallet flow, signing, submission or successful receipt. This research did
not refresh those network facts. A type symbol, summed coin balance or faucet
receipt is not address-balance eligibility.

## 5. Native wallets and custody: selective modules, not whole-wallet forks

### Gem: revise the blanket license assumption

At observed monorepo revision
`08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7`, the top-level README describes
native SwiftUI iOS, Kotlin/Compose Android and a shared Rust core, and says
GPL-3.0. **The current `core/LICENSE` separately grants MIT**, and
`core/Cargo.toml` declares MIT with workspace version `2.114.83` [G1–G3].
This is stronger evidence than merely finding an older standalone MIT repository.
The old `gemwalletcom/core` README also remains available as a separate source;
do not infer its migration state or all new files' licenses from it alone.

Concrete candidates [G4–G6]:

- `core/crates/gem_keystore`: Rust, feature-separated `mnemonic`, `storage`,
  legacy `v3` migration. File-backed v4 format uses Argon2id and AES-256-GCM;
  all metadata except ciphertext is bound as authenticated additional data.
  Defaults documented: 19,456 KiB memory, two iterations, parallelism one,
  16-byte salt, 32-byte derived key, 12-byte nonce, 16-byte tag. These are
  upstream defaults, **not this report's recommended universal password policy**.
- `storage/crypto.rs`: uses RustCrypto AEAD/KDF primitives, bounded parameter
  validation and `Zeroizing` derived keys—not novel cryptography to recreate.
- `storage/file_io.rs`: Unix owner-only mode `0600`, `create_new`, capped reads
  and directory synchronization. Non-Unix owner-only creation is explicitly
  unsupported in this module. This is genuine Linux-portable code, not an
  Android Keystore wrapper.
- `core/crates/gem_sui`: uses official `sui-sdk-types` and
  `sui-transaction-builder` 0.4.0. However its RPC feature includes Gem's own
  `gem_jsonrpc`/client/provider stack and many workspace dependencies. Do not
  replace our narrower gRPC adapter just to inherit a multi-chain app core.

**Recommended disposition:** shortlist `gem_keystore` as the first encrypted-file
custody reuse experiment after the offline milestone, not an entire Gem fork.
Review the exact chosen subtree, every dependency/file license, file replacement
and symlink handling, password rotation, backup/restore, cancellation, memory
lifetimes and error/log behavior. Pin a reviewed commit. Its package manifest
does not individually repeat a `license` field, so keep the subtree license and
obtain clarification if extraction provenance is ambiguous. Do not say “audited
Linux custody”: no build, integration, audit-scope review or recovery drill was
performed here. Do not generate/import any real secret to evaluate it.

### Other wallet bases

| Project | Verified primary source / portable portion | Fit and decision |
| --- | --- | --- |
| Trust Wallet Core | Apache-2.0; primarily C++ with C interfaces and Rust chain crates. Current `rust/chains/tw_sui/src/signer.rs` builds/signs through its own `TWTransactionBuilder`/`TxSigner`; observed revision `2a8aa02011ec42b43689667682fa99ae905418d5` [W1–W2] | Real Linux library, not Android-only. But another Sui engine duplicates Mysten types/policy. Reject whole-engine adoption. `src/Keystore/` may be separately reviewed only for a demonstrated import/storage gap; no generic hardware custody guarantee. |
| Electrum | MIT `LICENCE`; Python Bitcoin wallet, PyQt6 native UI and hardware plugins [W3] | Excellent UX reference for watch-only/offline signing and recovery concepts. Not Rust; replacing Bitcoin synchronization/transaction model is a major rewrite. Reject as Sui base, do not call it a Rust wallet. |
| Monero GUI | BSD-3-Clause-style top-level license; C++/Qt GUI of Monero core; Linux packaging/builds, experimental Android build [W4] | Reusable Qt/recovery/watch-only UX ideas, not drop-in Sui custody or transaction engine. Check Qt/submodules separately. No port/fork selected. |

### Secure-keystore boundary

An encrypted file or desktop keyring protects data at rest under a particular
threat model. It is not a hardware non-exportable key, trusted transaction
display or proof that a compromised UI cannot request a signature.

Freedesktop Secret Service provides D-Bus collections/items, sessions,
lock/unlock and prompts [K1]. Keyring-rs exposes secret retrieval across native
stores; current guidance favors `keyring-core` plus the specific store rather
than importing all CLI stores [K2]. These are plausible places for an encrypted
vault wrapping secret or application credential. They do not implement a Sui
transaction-policy enclave. A login-unlocked keyring can still release a secret
to an authorized/compromised client; no per-payment approval should be assumed.

Retain `PaymentSigner` and explicit `ApprovalProfile` distinctions. For software
custody, decrypt only inside the signer boundary and bind approval to immutable
verified terms/digest; never pass key material through NFC, UI properties, shell
arguments, logs or the merchant. TPM/FIDO/PKCS#11/secure elements require exact
curve, key-import/derivation, Sui intent-signature and approval-profile evidence.
Generic Ed25519 or NFC support is not sufficient Sui compatibility.

## 6. Native UI: Qt/CXX-Qt remains reasonable, with explicit limits

The project already selects Qt Quick/CXX-Qt at pinned revision `f62f0570…`.
Current upstream README [Q1] describes native Rust↔C++/QObject generation,
signals/properties and CMake or Cargo integration. It lists Linux, Windows,
macOS and WebAssembly CI; other Qt/Rust platforms are not regularly tested.
It also calls the API early-development/frequently changing. Qt's own Android,
iOS and Linux support matrix [Q2] does **not** establish CXX-Qt mobile CI or
support for a particular Linux phone/NFC controller.

Qt Quick is not Electron or Node. However **QML supports JavaScript expressions
and its runtime is part of the UI stack**. If “no JavaScript runtime” means no JS
in the chain/wallet core, a thin QML view with all approval/policy in Rust meets
that architectural intent. If it literally forbids any JS-capable runtime in the
wallet process, Qt Quick needs a product decision; Qt Widgets/C++ with Rust
core or GTK is the more direct alternative. Do not conceal this distinction.

`gtk4-rs` is a real native Rust binding alternative (MIT bindings; GTK's own
licenses separately apply), with explicit main-thread/non-`Send` UI constraints
[Q4]. It reduces C++ bridge work for a Linux-only UI, but does not automatically
deliver the desired mobile portability. Do not switch toolkit just for the
offline CLI milestone. Later keep rendering and terminal/NFC/network events
outside the policy engine, marshal work off the UI thread, and require a fresh
approval whenever immutable payment terms change.

CXX-Qt is MIT OR Apache-2.0, with noted BSD-3-Clause CMake files. **Qt itself is
not covered by that license**: inspect each module's LGPLv3/GPL/commercial terms,
notices, corresponding library source, relinking/replacement and installation
information obligations [Q3]. Static mobile builds/app-store rules deserve
specific review. An Apache project is not automatically exempt from those duties.

## 7. Merchant checkout: BTCPay and Taler supply mature operational lessons

### BTCPay: useful code, wrong engine for a wholesale native-wallet fork

Observed revision: `f60549da343751e031a6edc383716aa4c9b65ec5`.
BTCPay is MIT; its checkout server is C#/.NET, not a portable Rust wallet core.
No built-in Sui support was established. The useful boundary is merchant
operations/reference or an optional integration with a willing merchant—not
making a mandatory BTCPay server part of Linux Tap's offline demo.

Actual code paths inspected [B1–B3]:

- `BTCPayServer/HostedServices/InvoiceWatcher.cs::UpdateInvoice`: separates
  `New`, `Processing`, `Settled`, `Expired`, `Invalid`; tracks partial, overpaid
  and paid-late exceptions. Full payment does not automatically mean settled.
  Pending invoices are reloaded on startup, with time for payment listeners to
  catch up. Updates persist before publishing the generated events.
- `BTCPayServer/Plugins/Webhooks/WebhookSender.cs`: per-webhook ordered queue,
  serialized JSON bytes signed with HMAC-SHA256 in `BTCPay-Sig`, saved delivery
  request/result, explicit `OriginalDeliveryId` and redelivery flag. Automatic
  retry policy distinguishes 408/429/5xx/network failures from other failures.
  Its bounded retry schedule is source evidence, not an exactly-once guarantee
  across every crash. Merchant consumers must deduplicate business effects.
- Official invoice documentation: refunds create a **pull-payment claim** so the
  customer supplies a destination; invoice links preserve refund accounting and
  reopen an existing active refund rather than blindly duplicate it [B4].

**Adapt:** durable payment observations versus invoice state, an explicit
pending/unknown state, startup reconciliation, independently defined settled
predicate, exception/refund ledger, authenticated notifications and consumer
deduplication. These are valuable even with one accepted transfer per invoice.
Do not copy Bitcoin confirmation/RBF rules onto Sui, nor treat a webhook or
manually marked status as independent finality proof. Avoid importing the whole
.NET merchant server merely to emit one simulated receipt.

### GNU Taler

Taler's official merchant API is much richer than a payment URI: contract terms,
claim/payment/status/proof/refund endpoints and merchant session IDs. The fetched
page reports merchant protocol v44 and explicitly lists some clients targeting
older versions; version compatibility must be checked rather than assumed [T1].
Its POS manual confirms QR **or NFC**, with an Android tablet-oriented POS [T2].
That proves a documented NFC use case, not a Linux Sui APDU implementation.

Taler is a different payment system with exchange-issued digital cash and its
own deposit/settlement/trust model. Its native merchant libraries may be useful
for a separate **Taler rail**, but substituting Sui for its exchange is not a
small adapter. Wallet TypeScript/runtime and platform assumptions cannot be
erased by the word “native”. Use its contract/session/refund/replay design as a
reference; do not fork it for this milestone.

Licensing is component-specific: merchant backend AGPL, merchant client library
LGPL, standalone/tests generally GPL; wallet components generally GPL, some
LGPL. The official licensing overview itself warns it can be outdated [T3].
Inspect each source header before copying. AGPL modified network services have
source-offer duties; LGPL libraries have replacement/linking duties. Referencing
the design is not importing copyleft source. Documentation copying also has
separate GPL/GFDL considerations.

### Interledger Open Payments: correct scope

The official project defines wallet-address discovery, resource APIs and a
GNAP authorization server for account-servicing entities [I1–I2]. It is useful
for incoming/outgoing payments, grants and quotes in that ecosystem; it is not
an ISO-DEP AID, merchant-reader/card selection protocol, Sui SDK, secure key
store or proof of trustless onchain settlement. Its repository is Apache-2.0
and points to a separate specifications repository. Do not infer one repository's
license covers every SDK/specification or invent an official Rust implementation.
Keep as a bounded future account-based rail comparison, not the identity or
mandatory backend of this project.

## 8. Hardware signer and transport reuse

### Ledger Sui app: exact compatibility caveat

Reviewed source revision:
`1c7a7ad4e74af6e2b08aa4adc426bd77b1a78d60`, Apache-2.0 app [L1–L5].
The README lists Nano S+, Nano X, Flex and Stax; it says Nano X development
compatibility is checkable only with Speculos because development sideloading
is unavailable. `ledger_app.toml` additionally lists `apex_p` as a build/test
target. That **is not a promise of installed/released device support** for a
particular firmware or app-store version. README installation examples and
manifest target lists do not replace release/device qualification.

App APDUs are a different protocol from checkout NFC:

- CLA `00`; INS `00` version, `01` verify address, `02` public key, `03` sign,
  `22` signed token descriptor, `FE` version string, `FF` quit.
- BIP32 path components are little-endian in this documented app interface.
- The app's block protocol chains 180-byte parameter chunks using SHA256 and
  hash-addressed `GET_CHUNK` requests; signature/result returns to the host.
- Signing accepts transaction bytes/path and sometimes referenced objects for
  clear display. Missing information may cause blind signing or rejection.
- Empty gas payment requires `ValidDuring`. Signed dynamic token descriptors
  supply ticker/magnitude/type with a trusted-source signature; host labels
  alone do not establish trusted token display.

**Concrete gap:** `rust-app/src/parser/tx.rs:1216–1240` handles
`balance::redeem_funds` only when the first argument is `Argument::Result`
pointing to `CommandResult::FundsWithdrawalSplit`. Linux Tap's first call passes
`Argument::Input(0)` directly. `balance::send_funds` then expects that recognized
redeem result. The inspected SUI withdrawal test includes an explicit split;
the SIP-58 empty-gas test transfers SUI with address-balance gas, not our exact
zero-price USDC request. The source therefore shows a **clear-parser mismatch**;
whether the app rejects or falls back to blind signing depends on its handler
and settings and was not executed here. Do not “solve” it by quietly enabling
blind signing or changing the approved transaction. Token type/display behavior
needs its own exact fixture qualification as well.

**Next reuse experiment, separately authorized:** submit the exact public-fixture
SDK-generated transaction to a pinned Speculos Sui app, inspect approval screens,
reject on mismatched terms, and verify returned signature with `sui-crypto`.
If the mismatch is confirmed, propose a narrowly reviewed upstream parser fix
or explicitly qualify a different stable template before adopting it. Record
app revision, model target, firmware profile, descriptor source, derivation path
and actual `ApprovalProfile`. No upstream issue or patch was published here.

### Speculos and Rust transport

Speculos emulates Ledger apps, not RF or the entire hardware security boundary.
Its README explicitly disclaims identical device behavior and warns apps can
make arbitrary Linux syscalls through QEMU/semihosting [L6]. Only run a trusted,
pinned app in a constrained local environment. Its current package metadata
advertises Nano S+/X, Stax, Flex and Apex+; it does **not establish original
Nano S support**.
Emulator/model support and the Sui app's released-device support are separate
facts. The project has GPL/LGPL license files; package metadata says GPLv3.
It is development tooling, not something to embed into an Apache wallet binary
without component-level license review.

Zondax `ledger-rs` supplies Apache-2.0 `ledger-apdu`, `ledger-transport`,
`ledger-transport-hid` and emulator-oriented `ledger-transport-zemu` [H1].
Its `Exchange` abstraction is reusable for a Sui app client. Native HID source
uses `hidapi`, validates Ledger channel/tag/sequence framing, and can open a
specific enumerated device [H2]. But it does **not** implement the Sui app's
SHA256 block protocol or Sui signing policy. Source also exposes integration
hazards: first-device convenience selection, blocking reads behind an async
trait, a very long read timeout, concurrency warnings and info-level APDU hex
logging. Qualify cancellation/device selection and redact/disable sensitive
traffic logs rather than treating this as a custody-safe ready-made client.

`hidapi-rs` is a native Rust wrapper over HIDAPI, MIT wrapper license [H3];
system/backend library licensing and Linux device permissions remain separate.
PC/SC is appropriate for the merchant reader, not automatically Ledger USB HID.
Reuse the correct transport for each role; do not merge signer APDUs and
checkout APDUs merely because both use the word APDU.

## 9. Concrete reuse/adapt/reject plan

| Priority | Reuse unchanged where possible | Thin project-owned part | Reject / defer |
| --- | --- | --- | --- |
| Immediate offline milestone | Existing Mysten types/signatures, minicbor, rusqlite, Rust framing/receiver, Unix sockets | Feed reconstructed APDU invoice bytes into the existing verifier and explicit approval flow; add one realistic lost-receipt scenario | Live RPC, new rail, foundation backend, production keys, RF claims, wallet fork, Node/Android dependency |
| Durable unknown-outcome behavior | Existing first-write-wins `Prepared` journal and merchant transaction/unique constraints | Compare exact BCS + signature + digest across restart; drop ACK after merchant commit; reconnect/replay and check one receipt | Fresh signing, nonce regeneration, quote refresh-and-pay after uncertain outcome |
| Later controlled Sui settlement | Pinned SDK execute-and-checkpoint helper and ledger reads | Bounded submission/reconciliation adapter retaining exact approved artifact and full receipt predicate | Custom BCS/crypto, JSON-RPC migration, helper return value treated as business settlement without checks |
| Later software custody | Gem MIT `core/gem_keystore` candidate; explicit selected native keyring store if useful | Reviewed isolated signer integration, approval binding, recovery/rotation/backup policy | Whole mobile app fork, assumption that file encryption/keyring equals trusted hardware |
| Later hardware custody | Ledger Sui app + Speculos/Ragger fixtures + native HID `Exchange` | Sui block-protocol client, exact shape qualification, SDK verification, declared approval profile | Generic FIDO/TPM/NFC equals Sui; blind-signing fallback; unsupported model/release claims |
| Later UI | Existing selected CXX-Qt/Qt libraries | Thin verified-terms view and explicit approval; keep policy in Rust | Unneeded UI framework switch; QML presented as literally JS-runtime-free |
| Later merchant operations | BTCPay/Taler/OCP requirements and proven external APIs where appropriate | Sui-specific ledger/settled predicate; refund/late/overpayment relationships; deduplicated events | Importing unrelated settlement engines or claiming their HTTP status is Sui finality |
| Future protocol interop | Actual identified initiative's frozen spec/vectors, if supplied | Separate versioned transport/profile adapter | Claiming our private AID implements an unidentified public standard |

### Smallest acceptance scenario (proposal, not executed here)

Use public fixture keys and a synthetic chain only. The merchant sends a signed
invoice through emulated SELECT/OPEN/PUT/COMMIT. The wallet reconstructs those
exact bytes, verifies its separately configured merchant pin and bounded terms,
shows them and requires explicit approval. Persist the signed artifact before
claim delivery. Restart after journaling; then deliver the claim, let the
merchant commit its simulated payment, and intentionally lose the receipt.
Restart/reconnect and resend the saved artifact. Independently inspect both
journals: identical BCS, signature and digest; exactly one merchant simulated
payment; replay receipt for the existing digest; `chain_submission: false`.

This one focused flow closes the integration/lost-ACK proof missing from the
recorded pre-send interruption. It does not require a new generic payment
framework, fresh crypto primitives, a broad unit-test matrix, hardware emulator
installation or live payment. Existing checks should run when product code is
changed; this research-only task intentionally did not build/run them.

## 10. Licensing and outstanding proof checklist

This is engineering triage, not legal advice. For every eventual copied or linked
component preserve exact revision, source origin, license and dependency notices.

- **Apache-2.0:** SDK, Ledger app, Zondax transport and Trust Core need license /
  attribution preservation, applicable NOTICE, changed-file notices and patent
  terms. Their licenses do not grant trademark endorsement or release support.
- **MIT/BSD:** OCP documentation/code, Gem's inspected core, Electrum, GTK Rust
  bindings, PC/SC Rust and nfcd/CXX-Qt pieces require their respective notices;
  BSD non-endorsement terms apply where present. Gem app/source outside the
  licensed core must not be blanket relicensed MIT.
- **GPL/AGPL/LGPL:** Gem apps, Taler, Speculos and Qt/system libraries have distinct
  distribution/network/relinking obligations. Whole-program forks and library
  reuse are different choices. Test tooling can remain separate from the wallet.
- **Hardware/firmware:** open app source is not proof the Ledger OS/device stack,
  controller firmware or issuer asset controls are fully open.
- **Spec documents/assets:** ERC draft CC0 is not an audit; OCP brand assets and
  service terms are not necessarily just protocol code. Keep attribution and do
  not imply affiliation. Taler/Qt documentation has its own copying terms.

Remaining blockers before broader claims:

1. Resolve exact initiative URL/author and obtain missing finalized requirements.
2. Integrated APDU-origin invoice approval and lost-receipt-after-commit proof.
3. Fresh controlled wallet/address-balance qualification, exact asset/network
   policy, and separately authorized submission/reconciliation. No funds or
   custody assumptions are implied by prior public-address simulation.
4. Exact Ledger gasless-USDC clear display/signature behavior and released app /
   model / firmware availability; no hardware run was performed.
5. Gem module license/dependency extraction review and recovery/security review;
   no production custody has been qualified.
6. UI toolkit interpretation of the no-JS requirement; actual Omarchy/mobile
   packaging and accessibility; no UI runtime was started.
7. Physical NFCC listen mode, reader/controller/firmware timing, RF field loss and
   a willing terminal integration. Software APDU success is not RF proof.
8. Independent wallet interoperability and external security review. Neither a
   public repo, audit badge, CI workflow nor an emulator snapshot proves these.

**Owned runtimes: none.** All conclusions above are from read-only source and
documentation inspection, except the explicitly labelled future proposals.

## Primary-source register

Except D2 (explicitly marked discovery-only), links were fetched or their
identified raw source/API/tree was inspected on 28 October 2026. Commit-frozen
links are preferred; unpinned official pages are mutable and must be rechecked
before implementation.

- **O1:** [Open CryptoPay official site](https://opencryptopay.io/) — official documentation and integration links; NFC marketing and POS development qualification.
- **O2:** [OCP wallet integration specification, frozen README](https://github.com/openCryptoPay/landingPage/blob/e97197a76793e60c5ba23b8a34998c69c304865a/README.md) — quote, method, return path, fee and refund semantics.
- **O3:** [OCP actual MIT license](https://github.com/openCryptoPay/landingPage/blob/e97197a76793e60c5ba23b8a34998c69c304865a/LICENSE).
- **O4:** [OCP map API README](https://github.com/openCryptoPay/api/blob/c9d36b959d96e59455960f45b9c1b4fcfb55809f/README.md) — Bun map/admin service, not payment settlement.
- **O5:** [OCP client tests README](https://github.com/openCryptoPay/tests/blob/01410bd6cb78c56dc0385a7d7cc3c7b678a7bde0/README.md) — Node/runtime, invoice side effects and no real payment coverage.
- **E1:** [Ethereum ERCs PR #686 file diff](https://github.com/ethereum/ERCs/pull/686/files) — proposed ERC-7798 Tap to Pay, draft scope, roles, payloads and CC0.
- **S1:** [Solana's Mobile Stack announcement](https://solana.com/news/solana-mobile-stack-reveal) — Android toolkit, Seed Vault, NFC URL capture; announcement, not qualification.
- **S2:** [Current Solana Foundation pay repository](https://github.com/solana-foundation/pay) — observed redirect target and current HTTP agent-payment scope.
- **D1:** [OCPI maintainer README](https://github.com/ocpi/ocpi/blob/master/README.md) — Open Charge Point Interface and EV module/version scope.
- **D2:** [Oracle's own OPI documentation](https://docs.oracle.com/en/industries/retail/retail-eftlink/24.0/eftci/oracle-payment-interface-opi.htm) — distinct Oracle product name; linked search result only, not used as source for Open Payment Initiative protocol details.
- **M1:** [Mysten head revision](https://github.com/MystenLabs/sui-rust-sdk/commit/1313d5112464ecc25d7407f42442adea08589fc5) and [commits API](https://api.github.com/repos/MystenLabs/sui-rust-sdk/commits?per_page=6) — observed default-branch history.
- **M2:** [SDK types manifest](https://github.com/MystenLabs/sui-rust-sdk/blob/1313d5112464ecc25d7407f42442adea08589fc5/crates/sui-sdk-types/Cargo.toml), [RPC manifest](https://github.com/MystenLabs/sui-rust-sdk/blob/1313d5112464ecc25d7407f42442adea08589fc5/crates/sui-rpc/Cargo.toml) — source versions and licenses.
- **M3:** [Transparent retry change](https://github.com/MystenLabs/sui-rust-sdk/commit/db79f3431a479fb765fa57ceb3fb2372a8022697) — nonprocessed-request limits.
- **M4:** [Pinned Rust transaction builder](https://github.com/MystenLabs/sui-rust-sdk/blob/1313d5112464ecc25d7407f42442adea08589fc5/crates/sui-transaction-builder/src/builder.rs) — withdrawal helpers, empty-gas rejection and async simulation behavior.
- **M5:** [Pinned execution/checkpoint helper](https://github.com/MystenLabs/sui-rust-sdk/blob/1313d5112464ecc25d7407f42442adea08589fc5/crates/sui-rpc/src/client/transaction_execution.rs) — duplicate lookup, error variants and waiting semantics.
- **M6:** [Official gasless stablecoin transfers](https://docs.sui.io/develop/transaction-payment/gasless-stablecoin-transfers) — eligibility, no SUI requirement, restrictions and TS-specific automation.
- **G1:** [Gem monorepo README](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/README.md) — mobile apps and top-level GPL statement.
- **G2:** [Gem core MIT license](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/LICENSE).
- **G3:** [Gem Rust workspace manifest](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/Cargo.toml).
- **G4:** [Gem keystore README](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/README.md) and [manifest](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/Cargo.toml).
- **G5:** [Keystore crypto](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/src/storage/crypto.rs), [Unix file I/O](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_keystore/src/storage/file_io.rs).
- **G6:** [Gem Sui manifest](https://github.com/gemwalletcom/wallet/blob/08a6dcd7d3736b2b5cf2c1e3dd8dfe6b2658eba7/core/crates/gem_sui/Cargo.toml) — SDK reuse and provider dependencies.
- **W1:** [Trust Core README](https://github.com/trustwallet/wallet-core/blob/master/README.md) — languages, platform and Apache license statement.
- **W2:** [Trust Sui signer](https://github.com/trustwallet/wallet-core/blob/2a8aa02011ec42b43689667682fa99ae905418d5/rust/chains/tw_sui/src/signer.rs) and [Keystore source directory](https://github.com/trustwallet/wallet-core/tree/2a8aa02011ec42b43689667682fa99ae905418d5/src/Keystore).
- **W3:** [Electrum README](https://github.com/spesmilo/electrum/blob/master/README.md), [actual MIT license](https://github.com/spesmilo/electrum/blob/master/LICENCE).
- **W4:** [Monero GUI README](https://github.com/monero-project/monero-gui/blob/master/README.md), [actual license](https://github.com/monero-project/monero-gui/blob/master/LICENSE).
- **K1:** [Freedesktop Secret Service specification](https://specifications.freedesktop.org/secret-service/latest/).
- **K2:** [Keyring-rs README](https://github.com/open-source-cooperative/keyring-rs) — retrieved through its historical `hwchen/keyring-rs` raw URL; API/store split and MIT OR Apache-2.0.
- **Q1:** [CXX-Qt README](https://github.com/KDAB/cxx-qt/blob/main/README.md) — FFI, tested platforms, maturity and file-specific licenses.
- **Q2:** [Qt supported platforms](https://doc.qt.io/qt-6/supported-platforms.html).
- **Q3:** [Qt's open-source obligations](https://www.qt.io/licensing/open-source-lgpl-obligations).
- **Q4:** [gtk4-rs official API documentation](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/) — native bindings, threading and MIT license.
- **B1:** [BTCPay source tree](https://github.com/btcpayserver/btcpayserver/tree/f60549da343751e031a6edc383716aa4c9b65ec5), [actual MIT license](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/LICENSE) — license and language context.
- **B2:** [BTCPay InvoiceWatcher](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer/HostedServices/InvoiceWatcher.cs).
- **B3:** [BTCPay WebhookSender](https://github.com/btcpayserver/btcpayserver/blob/f60549da343751e031a6edc383716aa4c9b65ec5/BTCPayServer/Plugins/Webhooks/WebhookSender.cs).
- **B4:** [BTCPay invoice/refund documentation](https://docs.btcpayserver.org$HOME/).
- **T1:** [GNU Taler merchant API](https://docs.taler.net/core/api-merchant.html).
- **T2:** [GNU Taler POS manual](https://docs.taler.net/taler-merchant-pos-terminal.html).
- **T3:** [GNU Taler licensing overview](https://docs.taler.net/global-licensing.html).
- **I1:** [Open Payments official site](https://openpayments.dev/).
- **I2:** [Interledger Open Payments README](https://github.com/interledger/open-payments/blob/main/README.md), [actual Apache license](https://github.com/interledger/open-payments/blob/main/LICENSE).
- **L1:** [Ledger Sui README](https://github.com/LedgerHQ/app-sui/blob/1c7a7ad4e74af6e2b08aa4adc426bd77b1a78d60/README.md), [app manifest](https://github.com/LedgerHQ/app-sui/blob/1c7a7ad4e74af6e2b08aa4adc426bd77b1a78d60/ledger_app.toml).
- **L2:** [Sui app APDU interface](https://github.com/LedgerHQ/app-sui/blob/1c7a7ad4e74af6e2b08aa4adc426bd77b1a78d60/docs/apdu.md).
- **L3:** [Sui app block protocol](https://github.com/LedgerHQ/app-sui/blob/1c7a7ad4e74af6e2b08aa4adc426bd77b1a78d60/docs/block-protocol.md).
- **L4:** [Sui app transaction parser](https://github.com/LedgerHQ/app-sui/blob/1c7a7ad4e74af6e2b08aa4adc426bd77b1a78d60/rust-app/src/parser/tx.rs#L1216-L1240).
- **L5:** [SUI withdrawal test](https://github.com/LedgerHQ/app-sui/blob/1c7a7ad4e74af6e2b08aa4adc426bd77b1a78d60/tests/standalone/test_sign_sui_funds_withdrawal.py), [SIP-58 address-balance-gas test](https://github.com/LedgerHQ/app-sui/blob/1c7a7ad4e74af6e2b08aa4adc426bd77b1a78d60/tests/standalone/test_sign_sui_funds_withdrawal_sip58_gas.py).
- **L6:** [Speculos README](https://github.com/LedgerHQ/speculos/blob/39e6fb1503d2de2c0bc599ffec095470f46097a3/README.md), [package metadata](https://github.com/LedgerHQ/speculos/blob/39e6fb1503d2de2c0bc599ffec095470f46097a3/pyproject.toml).
- **H1:** [Zondax ledger-rs README](https://github.com/Zondax/ledger-rs/blob/master/README.md).
- **H2:** [Native Ledger HID transport source](https://github.com/Zondax/ledger-rs/blob/master/ledger-transport-hid/src/lib.rs).
- **H3:** [hidapi-rs README](https://github.com/ruabmbua/hidapi-rs/blob/master/README.md).
