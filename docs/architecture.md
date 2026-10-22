# Architecture

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
invoice is a deliberately smaller draft v0 schema. Successful network eligibility
remains unproven and submission is unimplemented. A trusted RPC predicate exists,
but no chain result has been manufactured to make the simulator look live. The separate
read-only `qualify-testnet` command now checks protocol configuration, exact asset
metadata, address-balance funding and checked simulation of `build_direct`.
It constructs diagnostic unsigned terms, not an authenticated payment request;
it does not relax invoice pins, signing or receipt predicates. Qualification
failed for the pinned asset on testnet, so submission remains unimplemented.
Hardware profiles, fresh merchant custody addresses, production recovery and
Qt approval remain explicit backlog items.

USDC is a centralized-issuer asset on a decentralized settlement network. The
project can remove mandatory wallet/cloud dependencies; it cannot remove issuer
controls, network availability requirements or terminal owners' deployment rights.
