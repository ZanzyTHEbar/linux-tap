# Architecture

**Current evidence, 29 October 2026:** one complete native private-PC/SC emulated
real testnet checkout is independently verified, including human-approved saved
bytes, checkpoint execution, merchant credit and wallet association.
[Sanitized first-checkout record](evidence/emulated-testnet-checkout-2026-10-29.json).
A second human-approved payment verified both application-boundary faults and
exact keyless restart/receipt recovery, preserving the authoritative first credit:
[recovery evidence](evidence/emulated-testnet-recovery-2026-10-29.json).
Wallet/merchant used the same official fullnode, not independent operators or a
light client. Physical RF, wire-level unknown outcomes, host failure, custody and power-loss
qualification remain open; this does not implement the full proposed architecture.

For the proposed target beyond this bootstrap, see [product architecture](product-architecture.md),
[product design](product-design.md) and [ADR 0003](adr/0003-reuse-first-product-boundaries.md).
Those proposals do not upgrade the implementation or validation facts below.

This bootstrap implements the first laboratory milestone of the existing native
Linux Sui engineering specification. It does not replace the foundation charter
or implement the full production service decomposition.

## Modules

tap-core owns invoice encoding, merchant signatures, local policy checks and a
bounded APDU receiver. It has no database, RPC, UI or hardware access.

tap-sui uses the pinned Mysten SDK for an experimental address-balance USDC
transaction, signing and verification. The template is reconstructed exactly
before it is accepted, so appended commands or substituted terms are rejected.
PaymentSigner provides an adapter boundary; only DevelopmentSigner is delivered.
The caller must obtain user approval. No hardware approval is implied.

tap-cli provides development wallet and merchant commands, SQLite journals and
the optional PC/SC reader client. The simulator has a synthetic chain and public
fixture keys; its tables and receipts are labelled simulated. Real-node reads
use a different command and cannot write simulated or production paid status.

The optional Python nfcd bridge contains transport glue only. It implements an
existing D-Bus interface and delegates bytes to Rust. It cannot sign or broadcast.
Python is not a chain or wallet engine. Replace this bridge with a Rust D-Bus
adapter if a tested device path justifies it; the existing daemon remains shared.

## Trust and process boundaries

- An invoice signature authenticates a key. The expected merchant key must come
  from local trust configuration, not the untrusted envelope itself.
- Approval displays exact asset, amount and recipient. The development signer
  verifies the template again before signing.
- The merchant independently checks execution through an explicitly chosen node.
  A checkpoint result here is a trusted-node assertion, not a verified light-client proof.
- The NFC receiver stores an unverified request and returns transport status.
  Neither the reader nor the daemon has spending authority.
- Production custody must be isolated from UI, NFC parsing and merchant adapters.
  CLI fixture code and raw development seed files are not that custody solution.

## Persistence and retry semantics

The wallet stores exact signed bytes and digest before sending its claim. It
loads the existing record on restart. It never overwrites that record with a
fresh transaction after an unknown outcome. The merchant transactionally records
one simulator receipt and accepts identical replays. Different receipts for the
same invoice are rejected for reconciliation.

This database rule is not onchain invoice deduplication. A separately signed
direct payment can still arrive. Live acceptance needs the full late-payment,
overpayment and refund ledger from the engineering specification. Structured
checkout with onchain deduplication is a separate paid or sponsored profile.

## Scope changes from the paper design

The initial workspace has three crates rather than ten services. The signed
invoice is a deliberately smaller draft v0 schema. Controlled funding/conversion,
eligibility and one actual testnet checkout are now verified. A trusted RPC predicate exists,
but no chain result has been manufactured to make the simulator look live. The separate
read-only `qualify-testnet` command now checks protocol configuration, exact asset
metadata, address-balance funding and checked simulation of `build_direct`.
It constructs diagnostic unsigned terms, not an authenticated payment request;
it does not relax invoice pins, signing or receipt predicates. The default
pinned asset fails on testnet. Explicit Circle testnet qualification uses the
same bounded template and reconstruction with a separate exact type/genesis
policy. The original qualification-only increment did not expose it through
signing or receipt verification. The explicit preparation checkpoint now does,
without changing legacy defaults; see the current README and validation record. A successful
simulation from a pre-existing funded public address is not fresh-wallet or
settlement evidence. Separate native conversion and controlled submission were
subsequently implemented and exercised by the human; the milestone above records
their narrow real-testnet qualification, including the scoped application-boundary
recovery above, not wire-level/host/power-failure proof.
Hardware profiles, fresh merchant custody addresses, production recovery and
Qt approval remain explicit backlog items.

Explicit testnet preparation adds bounded versioned candidate/reservation/artifact
tables under the existing cooperative owner lock, without promoting simulated
receipts. Its saved candidate binds the exact envelope, independent merchant pin,
fixed policy/profile, endpoint, signer, BCS and digest. The unique issuance key
binds merchant pin, chain and invoice ID. Approval and its qualification summary
are committed before key loading; recovery approvals append without replacing
the reservation. The subsequent controlled-dispatch slice adds a transactional
first intent/attempt barrier before polling the SDK execution helper, append-only
attempt approvals and typed node observations. It never rebuilds or signs a
transaction. A missing result after this barrier means possibly sent; restart
queries the saved digest first. Only explicit same-artifact replay is supported.

The independently configured merchant ledger adds one immutable trust namespace,
one receiver reservation per invoice (unique chain/receiver), independent node
observations, and transactional credit/receipt insertion with unique chain/digest.
Wallet confirmation rechecks the chosen merchant node before durable association.
All loaded approval, attempt, observation, credit and association records are
validated against their saved terms and profile; no simulator rows are promoted.
The node is trusted for checkpoint inclusion, not a light client. File receipts
are not authenticated remote business acknowledgements. These local commands
have no merchant private-key dependency. Full custody/backup, global receiver
freshness, remote delivery, refunds and broader wire/host/power-failure qualification remain open.

USDC is a centralized-issuer asset on a decentralized settlement network. The
project can remove mandatory wallet/cloud dependencies; it cannot remove issuer
controls, network availability requirements or terminal owners' deployment rights.
