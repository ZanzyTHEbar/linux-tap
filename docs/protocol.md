# Experimental protocol v0

This is a deliberately small executable draft, not the complete production
invoice schema in engineering specification v0.2. Do not freeze it or assign a
public compatibility mark before hardware and independent implementation tests.

## Invoice bytes

The signed core is a definite CBOR map with exactly ten fields, ascending integer
keys and shortest integer encodings. No floats, indefinite containers, duplicate
keys, unknown fields or trailing bytes are accepted. Decode through minicbor,
then re-encode and compare bytes to reject alternate encodings. Core limit is
2048 bytes; JSON envelope limit for NFC is 8192 bytes.

| Key | Value |
| --- | --- |
| 0 | unsigned version 0 |
| 1 | 32 byte chain identity, genesis checkpoint digest bytes |
| 2 | 16 random invoice bytes |
| 3 | 32 byte recipient address |
| 4 | full Sui asset type string, at most 512 UTF-8 bytes |
| 5 | positive u64 amount in base units |
| 6 | u64 creation time in UTC milliseconds |
| 7 | u64 expiry in UTC milliseconds; positive lifetime at most 15 minutes |
| 8 | merchant label, 1 to 128 UTF-8 bytes, no control characters |
| 9 | unsigned direct profile identifier 0 |

Sign ASCII `linux-sui-tap/invoice/v0`, one zero byte, then the exact CBOR core,
using Ed25519 through sui-crypto. Invoice hash is BLAKE2b-256 of those same bytes.
The JSON envelope has core_hex, merchant_key_hex and signature_hex. Its JSON
format is transport only and is not itself the signed representation. Unknown
JSON fields are rejected. A key pin must be provided by local policy. A valid
signature is not sufficient to prove a business identity.

The supplied fixture uses public test keys and expired timestamps. Its chain
identity is 32 repetitions of byte 42, not a live deployment configuration.
Regenerate it with `cargo run --locked -p tap-cli --example fixture`.

## Laboratory NFC application

Private lab AID is F04C54415000. This is not a registered public payment AID and
does not impersonate a card-network application. Transport uses short APDUs.

| Command | Header | Data and result |
| --- | --- | --- |
| SELECT | 00 A4 04 00 Lc | Lab AID; resets session; draft version byte in reply |
| OPEN | 80 10 00 00 22 | u16 big-endian message length and BLAKE2b-256 hash |
| PUT | 80 11 P1 P2 Lc | P1 P2 are u16 big-endian byte offset; up to 220 bytes |
| COMMIT | 80 12 00 00 00 | Verify complete length and hash; return hash |

9000 means transport success. 6A80 means rejected input and aborts session.
The receiver accepts exact repeated chunks but rejects changed or out-of-order
chunks. Removing the RF field requires a new selected session; the same invoice
can be sent again. COMMIT can be repeated. Extended APDUs and negotiated command
timing are outside this draft.

The nfcd bridge stores only an unverified invoice. Its 6581 response means the
request could not be persisted. It will not overwrite a different existing
request file. Signed invoice verification and explicit payment approval happen
later. No APDU response is proof of settlement.

## Sui draft

The native adapter accepts only the exact native USDC type in tap-core::USDC.
Amounts are at least 10000 base units. Its source is an address balance; coin
selection and conversion are not implemented. Its two calls redeem a sender
withdrawal and send it to the approved recipient. All gas fields are empty or
zero. ValidDuring binds chain, epoch and a journaled nonce. It is not a custom
checkout contract and does not add an invoice ID to chain state.

The wallet must persist exact signed bytes before submission. No broadcast
operation is delivered in this bootstrap. Future submission must first qualify
protocol eligibility, available funds, simulation and retry behavior. Invoice
expiry stops new approvals; it is not a millisecond cancellation of a signed
transaction. Separately signed direct transfers can pay the same invoice twice.
