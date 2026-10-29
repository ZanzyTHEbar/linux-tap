# Transport, durability and signer qualification notes

> **29 October 2026, 10:40:33 UTC update:** native `storage-info` SQL queries now
> verify patched bundled SQLite 3.51.3; real fixture checkpoint/reopen/replay also
> passed. [Current evidence](../evidence/sqlite-wal-qualification-2026-10-29.json)
> supersedes the runtime/fix gap below, not the remaining wire/host/power-durability limits.

**28 October 2026; research only.** Companion to the [decision map](README.md),
[payments audit](open-payments-reuse.md) and [phones/NFC report](linux-phones-nfc.md).
Sections 1–6 incorporate **coordinator-verified source findings supplied on this
date**; they are not tests performed by this writer. Section 7 adds this writer's
bounded primary-source Keycard inspection. No dependency or product code changed.

## 1. Persistence: WAL + FULL is a policy, not complete durability evidence

Coordinator inspected [Cargo.toml](../../Cargo.toml), [Cargo.lock](../../Cargo.lock)
and `Store::open` in [store.rs](../../crates/tap-cli/src/store.rs): rusqlite 0.37
uses `bundled`; libsqlite3-sys 0.35.0 supplies SQLite **3.50.2**. The cached
amalgamation's line 468 identifies that version/source date 2025-06-28.
`Store::open` selects WAL and `synchronous=FULL`; first-signed artifact and
merchant unique constraints remain useful safeguards.

SQLite's [WAL documentation, section 11](https://sqlite.org/wal.html) describes
a WAL-reset race affecting versions through 3.51.2, fixed in 3.51.3 with backports
including **3.50.7** and 3.44.6. The [3.50.7 backport](https://sqlite.org/src/info/c7facf7ac58d2cda)
links the parent race fix `268c9da287`. The triggering situation involves unusual
concurrent writer/checkpointer behavior. **No evidence shows this demo triggers
the bug.** Treat it as source qualification before multiwriter durability claims:
confirm the effective embedded runtime version, then review a narrow dependency
upgrade separately. Missing system pkg-config SQLite does not diagnose a bundled
build. No version or lockfile was changed here.

WAL/FULL still depends on OS, storage and correct synchronization behavior;
[atomic commit documentation](https://sqlite.org/atomiccommit.html) explains
those assumptions. Restart recovery is not power-cut proof. Do not separate a
live database from its WAL or copy only the main file as a reliable backup.
Use a coordinated/online backup procedure; keep persistent backups outside
temporary storage under the project's approved backup policy. The WAL warning
does not justify weakening synchronous commits or deleting journals.

## 2. Wire the existing paths; distinguish three acknowledgements

Coordinator source map at `f6ea6a8`:

| Existing path | What it proves or supplies |
| --- | --- |
| [apdu.rs](../../crates/tap-core/src/apdu.rs), receiver 17–85 / frames 89–107 | Private AID, 8,192-byte message and 220-byte chunk limits; bounded transport reconstruction |
| [main.rs](../../crates/tap-cli/src/main.rs), `apdu_stdio` 605–662 | Stores **unverified** JSON invoice and synchronizes before transport ACK |
| `pay_exchange` 332–411 | Receives invoice directly on socket/pipe, verifies/approves, loads or persists signed artifact |
| `merchant_exchange` 269–294 | Accepts simulated payment before sending receipt: useful lost-reply injection seam |
| `nfc_push` 479–499 | Real optional merchant PC/SC transmit path; not physical validation by research |
| [demo.py](../../scripts/demo.py), 107–113 | Separate APDU step occurs **after payment**, not integrated invoice ingress |

**Transport ACK:** invoice bytes persisted/reconstructed, not verified or paid.
**Payment ACK:** merchant simulator recorded the claim; still not chain proof.
**Finality:** separately qualified execution/checkpoint predicate on the right
chain; today's simulator does not produce it.
Document which ACK was lost. On reset/reselect discard transient APDU assembly,
not the approved payment journal. No human or network wait belongs inside the
APDU callback. Keep the simulator's private-socket claim/receipt return channel
explicit; current checkout APDUs transport an invoice, not a payment return API.
Reuse the same verifier/signer/store, not a second emulator-only wallet.

## 3. Optional QR and animated UR: transports, not approval standards

Static QR is a possible fallback, **not the next-scope requirement**.
[Nayuki QR-Code-generator](https://github.com/nayuki/QR-Code-generator) is MIT;
its Rust `qrcodegen` manifest is 1.8.0 with real `encode_text`/`encode_binary`
APIs in `rust/src/lib.rs`. [ZXing-C++](https://github.com/zxing-cpp/zxing-cpp)
is Apache-2.0 with C++20 core/C++17 API and Qt/Rust/C bindings. Its current
writer-backend Zint choice means actual selected build dependencies and licenses
need review. [ZBar](https://github.com/mchehab/zbar) is LGPL-2.1-or-later;
older Qt widgets are not automatic Qt 6 integration. No camera/phone test ran.

QR should carry the same verified invoice or a tightly constrained fetch profile,
not silently introduce a new rail/trust source. Never auto-open arbitrary URIs.
Bound decoded size and encoding; verify exact terms and require the same approval.

[UR specification](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md)
2.1.0 (revised 2023-08-21) currently requires dCBOR and says **`ur:bytes` MUST NOT
be used outside testing/validation**. A library README's bytes example does not
override that requirement. Linux Tap's current CBOR profile is not automatically
the required UR application profile merely because both use deterministic CBOR.

[ur-rs](https://github.com/dspicher/ur-rs), MIT `ur` 0.5.2, supplies native Rust
fountain encoding/decoding: custom `Encoder::new`, decoder `receive`, `complete`
and `message` APIs exist in `src/ur.rs`. [bc-ur](https://github.com/BlockchainCommons/bc-ur)
is another implementation; README says late alpha, with BSD-2-Clause-Patent and
mixed file licensing. Neither automatically defines Sui invoice/signing semantics.
Later adoption needs a named application type/profile, canonical payload rules,
interoperability vectors and explicit signer/return semantics. Bound fragments,
declared message size, active sessions, memory, CPU and assembly lifetime; handle
duplicate/mixed/malformed fragments. Preserve exact signed bytes, not merely
equivalent decoded objects. Animated QR is unnecessary to finish the emulator.

## 4. Schema and signature envelopes: avoid gratuitous protocol replacement

[RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html) defines CBOR, including
deterministic encoding considerations; [RFC 8610](https://www.rfc-editor.org/rfc/rfc8610.html)
defines CDDL schema notation. CDDL alone does not enforce every canonical-byte
rule or local amount/expiry/merchant policy. Preserve the existing strict decoder
and byte re-encoding comparison rather than infer equivalence from a schema.

[RFC 9052, section 10](https://www.rfc-editor.org/rfc/rfc9052.html#section-10)
requires application profiling and trust/key/authentication context. COSE is not
an invoice, settlement, merchant-identity or explicit-approval standard.
[google/coset](https://github.com/google/coset) is Apache-2.0, version 0.4.2 in
its manifest; its types/builders invoke caller-provided signing callbacks.
`examples/signature.rs` explicitly uses **`FakeSigner`**: never copy it as real
cryptography. Consider COSE only if the actual identified interoperability
specification calls for it; no envelope rewrite is justified by library existence.

Optional naming caution: the [Sui CAIP-2 namespace](https://namespaces.chainagnostic.org/sui/caip2)
was fetched as **Draft**, created 2025-06-18, with network-name/limited-digest
resolution examples. [CAIP-19](https://chainagnostic.org/CAIPs/caip-19) is a generic
asset-identifier proposal in Review, not automatically a canonical Sui mapping.
Never replace the existing full genesis digest/type pin with a mutable network
name, short prefix, token symbol or unqualified external identifier.

## 5. NFC threat model and acceptance are separate from transport success

[NFCGate](https://github.com/nfcgate/nfcgate) v2 is an Apache-2.0 **Android**
capture/relay/replay laboratory tool, including PCAPNG support; several modes
require Xposed-compatible hooks. It is not a native Linux wallet, a reason to
root a personal phone or authorization to touch others' credentials/readers.
Coordinator inspected the [WOOT 2020 relay/replay abstract](https://www.usenix.org/conference/woot20/presentation/klee)
and [USENIX Security 2025 “More is Less” abstract](https://www.usenix.org/conference/usenixsecurity25/presentation/pavlides),
including the latter's overlays/security-failure/formal-model claims, **not their
artifacts**. Do not report artifact reproduction or a protocol audit.

NFC proximity, a UID and status `9000` cannot establish merchant identity or
authorize payment. Retain independently configured merchant trust, displayed
exact asset/amount/recipient and bounded transaction reconstruction. A transport
secure channel is not protection against a malicious approved signing host.

[EMVCo contactless](https://www.emvco.com/emv-technologies/emv-contactless-chip/)
distinguishes acceptance kernels and approvals. [NFC Forum Type 4](https://nfc-forum.org/build/specifications/type-4-tag-specification/)
describes ISO-DEP/NDEF and offers the specification for purchase. Neither means
an OSS wallet gets EMV kernel acceptance, a public payment AID or a right to deploy
on commercial terminals. Available specifications are not necessarily OSS code.
Phone/controller selection remains in the separate phones report.

## 6. Optional native supply-chain checks

[cargo-deny](https://github.com/EmbarkStudios/cargo-deny) offers license, bans,
advisory and source checks; [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
checks RustSec advisories. Both advertise Apache-2.0 OR MIT. They are candidates,
not new mandatory frameworks, and **neither command ran here**. A clean result
would not be an independent security audit. Pin/review transitive native code
and effective features as well: bundled SQLite shows why a locked Rust wrapper
version or crate badge is insufficient evidence for the embedded C version.

## 7. Keycard / Shell: native transport exists; Ed25519 is a source-level blocker

This writer followed [Keycard's official developer page](https://keycard.tech/developers)
to its actual repositories, rather than assuming generic NFC/secp256k1 support.
The local `keycard-shell/README.md` was read only as public provenance; no
deployment directory, private key, card, PIN or build artifact was inspected.

| Source revision observed | Reusable portion / actual limitation |
| --- | --- |
| [keycard-rs `142ab915…`](https://github.com/keycard-tech/keycard-rs/tree/142ab915eda6bc4c1dc10078028f0d7b0748b218) | MIT Rust SDK 0.1.0; `CardChannel`, default optional PC/SC, V1/V2 secure-channel implementations; hardware integration tests explicitly ignored by default |
| [keycard-go `b08c0f5…`](https://github.com/keycard-tech/keycard-go/tree/b08c0f5f516e03c15671038426f3c700504454ba) | Go APDU/channel/command reference; MPL-2.0 repository metadata; README advertises Ed25519 alongside ECDSA/BLS/Schnorr, not proof the card implements it |
| [status-keycard `37393a2…`](https://github.com/keycard-tech/status-keycard/tree/37393a2cbe9a54352cc8d69e97fc7ea08a840625) | JavaCard applet; actual LICENSE is Apache-2.0, despite broad MIT website marketing |
| [Shell `97a349bd…`](https://github.com/keycard-tech/keycard-shell/tree/97a349bd1eb6a94442400b6e0bba40d6c5cd3cfe) | MIT top-level firmware; actual `hardware/license.txt` is CERN-OHL-S v2; vendored components retain separate licenses |

Native Linux reuse is concrete: [`src/pcsc.rs`](https://github.com/keycard-tech/keycard-rs/blob/142ab915eda6bc4c1dc10078028f0d7b0748b218/src/pcsc.rs)
establishes a user PC/SC context, connects with T=0/T=1 and transmits serialized
APDUs. Defaults select the first reader; provided names use **prefix matching**,
not exact identity. `is_connected` checks a stored handle, not fresh card presence.
Review exact selection, reconnect/card identity, blocking/cancellation and session
reset before adopting it. Manifest has release-candidate crypto dependencies;
version 0.1.0 and README claims are not maturity/security qualification.

The decisive algorithm evidence is [`SECP256k1.java::signHash`](https://github.com/keycard-tech/status-keycard/blob/37393a2cbe9a54352cc8d69e97fc7ea08a840625/src/main/java/im/status/keycard/SECP256k1.java#L201-L213):
`SIGN_ED25519` and `SIGN_BLS12_381` fall through to
`ISO7816.SW_FUNC_NOT_SUPPORTED`; ECDSA and BIP340 have implementation branches.
Constants/host methods for Ed25519 do **not** establish implemented Ed25519.
[`KeycardApplet.java::sign`](https://github.com/keycard-tech/status-keycard/blob/37393a2cbe9a54352cc8d69e97fc7ea08a840625/src/main/java/im/status/keycard/KeycardApplet.java#L1041-L1089)
derives a secp256k1 key, accepts a precomputed hash and delegates to that switch.
Rust [`parsing/signature.rs`](https://github.com/keycard-tech/keycard-rs/blob/142ab915eda6bc4c1dc10078028f0d7b0748b218/src/parsing/signature.rs)
supplies secp256k1 recovery/Schnorr parsing, not a proven Sui Ed25519 signer.

Sui supports other signature schemes, but **Linux Tap's current predicate is
Ed25519-only**. A hypothetical secp256k1 adapter still needs reviewed derivation,
Sui intent/prehash semantics, address/signature serialization and SDK verification;
it cannot silently change existing accounts or policy. Card PIN/secure channel
does not display the invoice. Hash signing alone is host approval, not trusted
Sui amount/asset/recipient display. Do not use the available pinless signing path.

Shell adds its own display/camera/firmware policy. Its inspected
[`app/ur/ur.h`](https://github.com/keycard-tech/keycard-shell/blob/97a349bd1eb6a94442400b6e0bba40d6c5cd3cfe/app/ur/ur.h)
enumerates Ethereum/Bitcoin QR types and has 128-part bounds; that is not a Sui
signing profile. Official docs distinguish immutable card applet versions, so
new source does not upgrade older shipped cards. Secure-element certification
is not certification of the wallet integration or Shell's trusted Sui display.

**Decision:** shortlist MIT Rust PC/SC/channel code and Shell's bounded QR
architecture as later references; reject “ready Sui/Ed25519 hardware signer”.
Do not port secure-element cryptography, flash a card or replace its firmware to
finish the emulator. Pin a chosen revision, review all licenses and qualify
exact hardware/app/approval profile only in a separately authorized task.

## Validation boundary

Only public-source inspection and local prose/link/whitespace/Git checks were
performed. No builds, cargo tests, demo, scanner, emulator, card, service, browser,
live RPC, account, funds request, key generation or signing operation ran.
Coordinator evidence is attributed above, not relabelled as this writer's tests.
**Owned runtimes: none.**
