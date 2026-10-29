# linux-tap

Native Rust wallet and merchant reference tools for exploring contactless
stablecoin payments on Linux, with Sui as the first settlement network.
This is an experimental implementation, not a polished wallet or stable merchant SDK.

## Why it exists

A contactless payment should be inspectable from the request through settlement.
linux-tap explores that path with open source, explicit user approval, native
execution and separate merchant verification. The wallet and chain core use Rust;
they don't require a JavaScript runtime or a project-operated payment service.

The aim is to make the boundaries clear: receiving an invoice is not permission
to spend, and a successful NFC exchange is not proof that a payment settled.

## How a payment works

For an explicitly approved testnet payment:

1. The merchant creates a signed invoice with the asset, amount and recipient.
2. The wallet checks the invoice against a separately pinned merchant key,
   shows the exact terms and asks the user to approve them.
3. The wallet saves the signed transaction before submitting it to Sui.
4. The merchant verifies checkpointed execution through its chosen node,
   records one credit and issues a receipt.

If the result is uncertain, recovery follows the saved transaction rather than
creating another payment. Transport acknowledgements never stand in for approval
or settlement. **The quickstart below simulates this flow without a blockchain.**

## Try the local demo

You'll need Linux, Git, Python 3, a C compiler and the Rust toolchain specified in
[`rust-toolchain.toml`](rust-toolchain.toml). Rustup can select the pinned toolchain.
The first build may need network access to fetch the locked dependencies.

```sh
git clone https://github.com/ZanzyTHEbar/linux-tap.git
cd linux-tap
cargo build --locked -p tap-cli
python3 scripts/demo.py
```

Run the demo in a terminal. It starts separate native wallet and merchant
processes, delivers an invoice through the software APDU receiver, and asks you
to type `APPROVE SIMULATION`. It then walks through interruption, a lost receipt
and recovery using the same saved transaction bytes.

**Simulation only:** after dependencies are fetched, the demo runs offline with
no RPC, real funds, real wallet keys or NFC hardware. It uses public laboratory
keys; **never fund them**. It doesn't open your wallet or require a seed phrase.
Its receipts are labelled `SIMULATED_PAID_NOT_ON_CHAIN`.

See the [demo guide](docs/demo.md) for the steps and expected output.

## What works now

- Signed invoices, independent merchant-key pins and strict checks of the
  network, asset, amount, recipient, expiry and exact transaction template.
- Native Sui integration using pinned Mysten SDK types, serialization,
  signatures and checkpoint verification rather than custom chain cryptography.
- SQLite journals that retain exact signed bytes, signatures and transaction
  digests for restart and reconciliation, with unique merchant-credit records.
- An emulated PC/SC path connecting the real native wallet and merchant tools.
- Human-approved **0.01 Circle USDC payments on Sui testnet**, including
  application-interruption recovery and byte-identical receipt regeneration.

The [checkout evidence](docs/evidence/emulated-testnet-checkout-2026-10-29.json)
and [recovery evidence](docs/evidence/emulated-testnet-recovery-2026-10-29.json)
record those bounded results. The [validation record](docs/validation.md) explains
what was tested and what each result does—and doesn't—establish.

## What is not ready

- Production custody, encrypted-key recovery and a security-audited wallet.
- Physical NFC/RF, commercial-terminal acceptance and independent public-protocol
  interoperability qualification.
- Host/power/filesystem-failure durability and live execution-response-loss
  qualification.

Verification currently trusts an explicitly selected RPC node. It is not a light
client or proof from independent node operators. Hardware-held keys alone would
not establish a trusted transaction display.

## Working on the code

The implementation has three crates:

| Crate | Role |
| --- | --- |
| `tap-core` | Signed invoices, merchant trust and policy checks, bounded APDU framing. |
| `tap-sui` | Sui transaction construction, signing interfaces and selected-node verification. |
| `tap-cli` | Wallet/merchant commands, approval flows, journals and the optional PC/SC client. |

Run the existing developer checks:

```sh
bash scripts/check.sh
```

This runs formatting, Clippy, the workspace tests, a native build and the local
smoke scenario. The default checks don't require RPC access or a reader. PC/SC
development requires the platform dependencies described in the
[hardware guide](docs/hardware.md).

## Read more

- [Architecture](docs/architecture.md) — implemented modules and trust boundaries.
- [Testnet qualification](docs/testnet-qualification.md) — controlled testnet
  prerequisites and safety gates; not part of the quickstart.
- [Validation](docs/validation.md) — evidence, checks and remaining gaps.
- [Protocol](docs/protocol.md) — experimental invoice and transport format.
- [Backlog](docs/backlog.md) — outstanding implementation and qualification work.
- [Contributing](CONTRIBUTING.md) — development expectations.
- [Security](SECURITY.md) — reporting concerns and current security limits.

## License

New project code is [Apache-2.0](LICENSE). Third-party code and system components
retain their own licenses; see [third-party provenance](THIRD_PARTY.md).
