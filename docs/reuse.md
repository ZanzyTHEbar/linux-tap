# Reuse audit

## Addendum — 28 October 2026

See the [current research reports](research/README.md) for updated reuse,
native Linux phone/NFC and transport-security findings.

This addendum supersedes the historical Gem core license caution below:
the inspected current monorepo `core/LICENSE` is MIT; the app's top-level
GPL license remains separately scoped. Neither license should be applied
indiscriminately across the repository. This clarification is not automatic
adoption, a wallet-fork decision or permission to copy unreviewed files.

The user-selected immediate milestone is an integrated emulated APDU invoice →
wallet-verified invoice and explicit approval → lost payment receipt/restart →
replay of the exact signed bytes → one simulated receipt. This is not live
payment or RF proof and does not change backlog capability states.

## 22 October 2026 review (historical)

Reviewed 22 October 2026. The decision is to assemble a narrow application around
maintained components, rather than fork a complete wallet and replace its chain.
These are engineering selections, not claims that every upstream is audited.

| Area | Reuse | Decision and current integration |
| --- | --- | --- |
| Sui types and serialization | [Mysten sui-rust-sdk](https://github.com/MystenLabs/sui-rust-sdk) | Linked and tested; official BCS, addresses, transaction types and digests |
| Signing | sui-crypto | Linked; Ed25519 fixture signing and verification; hardware provider boundary added |
| Chain reads | sui-rpc | Linked; gRPC service probe and explicit-node receipt verification; no JSON-RPC |
| Transaction building | sui-transaction-builder | Pinned and gap reproduced; zero-gas draft uses SDK structures while empty-gas offline support is unresolved |
| CBOR | [minicbor](https://github.com/twittner/minicbor) | Linked; a narrow fixed integer map with re-encoding checks; no custom CBOR parser |
| Hashing | RustCrypto blake2 | Linked; BLAKE2b-256 domains; no custom hash implementation |
| Persistence | SQLite through rusqlite | Linked; transactional journals, unique constraints and full synchronous commits |
| CLI and runtime | clap and Tokio | Linked; typed commands, framed pipes and Unix socket development transport |
| Merchant reader | [bluetech/pcsc-rust](https://github.com/bluetech/pcsc-rust) and system PC/SC | Optional feature compiled; real reader enumeration and APDU transmit; no physical test |
| Linux card application | [Sailfish nfcd](https://github.com/sailfishos/nfcd) | BSD daemon protocol reused; original XML bundled; development bridge delegates framing to Rust |
| PN7160 integration | [NXP linux_libnfc-nci](https://github.com/NXPNFCLinux/linux_libnfc-nci/tree/NCI2.0_PN7160) | Dedicated branch pinned as a hardware stepping stone, not silently treated as an nfcd plugin |
| Native UI | Qt Quick and [KDAB CXX-Qt](https://github.com/KDAB/cxx-qt) | Selected next-stage UI; no UI code or claimed runtime integration yet |
| Agent guidance | [Mysten skills](https://github.com/MystenLabs/skills) | External upstream guidance; reviewed revision/hash inventory retained, not bundled |
| Hardware Sui app | [LedgerHQ/app-sui](https://github.com/LedgerHQ/app-sui) | Apache-2.0 application reviewed; possible comparison/adapter target; device stack is not fully FOSS |
| Merchant operations | [BTCPay Server](https://github.com/btcpayserver/btcpayserver) | MIT reference for self-hosted checkout operations; no BTCPay fork or Sui support is claimed |

## Wallet fork decisions

**Gem Wallet** is the closest reviewed reusable multi-chain Rust wallet core.
Its current monorepo contains Android Compose and iOS SwiftUI apps, and is
GPL-3.0. The older standalone core repository says it migrated and identifies
itself as MIT. Do not assume that historical license applies to new monorepo
files. A GPL fork would be possible with the applicable obligations, but it does
not supply a native Linux UI, our NFC transport or merchant acceptance. We are
not copying its code in this bootstrap.

Sources: [current wallet](https://github.com/gemwalletcom/wallet),
[historical core](https://github.com/gemwalletcom/core).

**Electrum** is MIT and provides a useful Linux wallet UX reference. Its Bitcoin
transaction, synchronization and wallet model would need substantial replacement.
**Monero GUI** supplies a native Qt example, but its wallet engine is similarly
chain-specific. Neither is selected as the application base. No source is copied.

Sources: [Electrum](https://github.com/spesmilo/electrum),
[Monero GUI](https://github.com/monero-project/monero-gui).

**Trust Wallet Core** is Apache-2.0, primarily C++, with multi-chain functionality
and language bindings. It does not replace the requested Mysten native SDK, and
introducing another chain engine adds qualification work. Reconsider only if a
specific reusable hardware or key-management module removes a demonstrated gap.
Source: [wallet-core](https://github.com/trustwallet/wallet-core).

## Protocol reuse and the small part we own

Use NFC ISO-DEP and standard APDU framing through existing controller stacks;
PC/SC for readers; D-Bus for the existing daemon; CBOR for invoice bytes; and
Sui's transaction and signature formats for settlement. Our custom protocol only
binds merchant invoice terms to the approved chain transaction and carries that
invoice between endpoints. Generic payment links do not supply those signed
merchant, expiry and reconciliation guarantees.

The APDU command set and invoice schema remain an experimental proposal. They
are not an EMV application, card-network token or approved terminal kernel.
NDEF and QR can become additional transports of the same signed core. Hardware
measurements and independent implementations must precede a stable protocol.

## Adoption rule

Add a dependency when it removes more security or maintenance work than it adds.
Inspect the actual license, platform assumptions, source revision and update path.
Use libraries unchanged where possible. Keep hardware or SDK fixes as small,
reviewable patches suitable for upstream. Do not rebrand an unported mobile app
as a native Linux wallet.
