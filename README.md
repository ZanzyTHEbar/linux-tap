# Linux Tap

A small native Linux wallet and merchant reference implementation for open NFC
payments, with Sui as the first settlement adapter. Working name, independent
Omarchy contributor project. No affiliation or endorsement is implied.

This bootstrap contains working native Rust invoice signing, validation,
transaction preparation, merchant verification predicates, durable journals and
an experimental NFC framing implementation. Its checkout demo is a simulation:
it uses real Sui transaction signatures but never submits funds to a chain.

## Run the tested checkout

For a guided terminal recording, see the [Omarchy demo](docs/demo.md):

```sh
cargo build --locked -p tap-cli
python3 scripts/demo.py
```

This uses separate native Rust processes over a private Unix socket. Review the
terms, type `APPROVE SIMULATION`, then follow the restart/replay prompts. It is
an offline software proof of concept, **not a live NFC payment demo**.

Install the Rust toolchain named in `rust-toolchain.toml`, a C compiler, Git and
Python 3. Then run from this directory:

```sh
cargo build --locked -p tap-cli
python3 scripts/smoke.py
```

The smoke test starts separate native wallet and merchant processes over pipes,
interrupts the wallet after journaling, restarts the merchant, retries the exact
signed bytes and checks there is one recorded simulated payment. It also tests
APDU reconstruction in software. It needs no RPC, credentials, NFC hardware or
Google service. Expected final result is `PASS` with `chain_submission: false`.

```sh
bash scripts/check.sh
```

That additionally runs formatting, Clippy and unit tests. The first build needs
network access to fetch pinned upstream code and locked crates.

## Read-only real chain access

```sh
cargo run --locked -p tap-cli -- probe --endpoint https://fullnode.testnet.sui.io:443
```

Use an endpoint you explicitly trust. `verify-on-node` checks a prepared
transaction against a signed invoice, the node chain identity, successful
execution, exact transaction bytes and checkpoint inclusion. This trusts the
chosen node; it is not a light-client proof. No live RPC success is implied by
the offline test results.

`prepare-draft` signs an experimental address-balance transfer offline, using an
explicit development seed file and typed approval. It does not simulate network
eligibility, submit the transaction or provide production key storage. Its key
file is 32 raw bytes, mode 0600. Run `linux-tap prepare-draft --help` for inputs.
Use fresh development keys. The demo's fixed fixture keys are public and must
never receive funds.

`qualify-testnet` reads protocol configuration, the exact pinned USDC metadata
and sender address balance, then simulates the existing transaction with checks
enabled and gas selection disabled. It needs public addresses only, not keys.
The [testnet qualification](docs/testnet-qualification.md) failed: the pinned
asset is not allowed on testnet. No asset substitution or broadcast is provided.

## NFC stepping stones

On Arch or Omarchy, the optional merchant reader build needs `pkgconf` and
`pcsclite`, plus a suitable reader driver such as `ccid` or a vendor-supported
alternative. Confirm the driver for your actual reader before installing it.

```sh
cargo build --locked -p tap-cli --features pcsc-reader
cargo run --locked -p tap-cli --features pcsc-reader -- readers
```

With a qualified card-emulation device running our laboratory application:

```sh
cargo run --locked -p tap-cli --features pcsc-reader -- nfc-push \
  --input fixtures/invoice.json --reader 'EXACT READER NAME'
```

Success means invoice transport acknowledgement only. The fixture is expired
and is not a request to pay. The private AID and APDUs are experimental.

`adapters/nfcd/bridge.py` exports Sailfish's existing LocalHostApp D-Bus interface
and forwards APDUs to the Rust receiver. It contains no keys or payment logic.
It requires a functioning nfcd hardware adapter; it does not magically connect
nfcd to the NXP vendor stack. See [hardware qualification](docs/hardware.md).

## What is deliberately unfinished

- Successful live gasless qualification and submission. The checked testnet
  simulation rejects the currently pinned asset.
- Hardware signer implementations and recovery, encrypted production keystore.
- Physical NFC qualification and a native phone driver path.
- Native Qt UI, commercial POS integration, production refunds and accounting.
- Cryptographic verification of network finality independent of an RPC provider.

There is no broadcast command. There is no foundation account or central registry.
Unmodified card terminals cannot accept this protocol without an acceptance
integration. Open software does not make USDC issuance or NFC firmware decentralized.

## Architecture and next work

- [Reuse audit and fork decisions](docs/reuse.md)
- [Architecture and trust boundaries](docs/architecture.md)
- [Experimental invoice and APDU specification](docs/protocol.md)
- [Ordered backlog](docs/backlog.md)
- [Hardware and bridge instructions](docs/hardware.md)
- [Validation record](docs/validation.md)
- [Native SDK gap](docs/adr/0002-empty-gas-builder.md)

Cargo.lock pins transitive dependencies; deps/upstreams.lock.json pins source
stepping stones. `python3 scripts/fetch_upstreams.py nfcd pn7160` downloads those
specific revisions for inspection without building, flashing or installing them.
Selected official Mysten skills are bundled unmodified under `.agents/skills`.
See [THIRD_PARTY.md](THIRD_PARTY.md) for their provenance and license boundaries.

New project code is Apache-2.0. Third-party source and system dependencies retain
their own licenses. This is a development bootstrap, not an audited wallet.
