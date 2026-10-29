# Linux Tap

A small native Linux wallet and merchant reference implementation for open NFC
payments, with Sui as the first settlement adapter. Working name, independent
Omarchy contributor project. No affiliation or endorsement is implied.

**Resume here:** [29 October emulation/testnet checkpoint](docs/checkpoints/2026-10-29-emulation-testnet.md)
summarizes completed evidence, remaining gaps, research reuse and the recommended next work item.

**Read-only query-response loss qualified:** a real local HTTP/2 reset produced
durable Unknown, then fresh-process recovery of the same saved checkpoint, in
[reconstructed lab state](docs/evidence/query-response-loss-2026-10-29.json).
No new payment; execution-response loss and host/power durability remain open.
Independent query-loss source/post-run review and the existing heavy checks are complete.
The [separate execution-loss source candidate](docs/execution-response-loss-lab.md)
preserves SDK recovery; compilation/offline native guards pass, not live fault qualification.

**B14 SQLite foundation qualified, 29 October:** the native `storage-info` API
reports patched bundled SQLite 3.51.3; real fixture checkpoint/reopen/replay
preserves exact artifacts and one simulated credit. [Scoped evidence](docs/evidence/sqlite-wal-qualification-2026-10-29.json).
This closes the known SQLite source/runtime gap, not wire-loss, host/power-failure
or production durability qualification. Post-validation independent review is
complete with no blocking/material P1/P2; B14 remains partial.

**29 October 2026: first native private-PC/SC emulated real testnet checkout
verified complete.** Human invoice/register/delivery/prepare/submit/accept/confirm
settled **0.01 Circle testnet USDC** at checkpoint **400344020**. Independent
keyless verification confirmed the saved transaction and local idempotency:
[sanitized public evidence](docs/evidence/emulated-testnet-checkout-2026-10-29.json).
This is one testnet milestone, not production readiness or physical NFC/RF proof.
The receiving address is now used: preserve custody/journals; do not rerun the
recorded recipe or reset receiver reuse policy with a new ledger. Hardware stays deferred.

**Application-boundary recovery also verified, 29 October 2026:** one new human-
approved **0.01 USDC** payment at checkpoint **400484678**, with both native
fault exits 86, keyless recovery and byte-identical receipt regeneration.
[Sanitized recovery evidence](docs/evidence/emulated-testnet-recovery-2026-10-29.json)
distinguishes received-success-before-persistence loss from credit-before-export
interruption. The authoritative ledger retains the first credit plus exactly one
new credit (**two total**). One additional independent reconcile supplied fresh
node proof; duplicate accept/confirm used cached credit/association, not fresh queries.
Both receivers are now used: preserve custody/journals and authoritative history;
do not rerun the [completed recovery recipe](docs/testnet-qualification.md#opt-in-application-boundary-recovery-review-completed)
or authorize a third payment from this evidence. Execution-response loss, host/power failure,
RF, independent node operators and production readiness remain unqualified.
Review found one signal-cleanup P2; the scoped fix now unwinds on SIGTERM/SIGHUP/
SIGINT, reaps only owned children and retains STOPPED evidence. Focused re-review
completed 29 October 2026 (`ses_e7d355137ffeaXiT10YXbCLGve`): **P2 RESOLVED,
no remaining material findings. The subsequent human trial is complete;
no further funds activity is authorized.** SIGKILL/root crash is outside this
guarantee; after possible broadcast, reconcile the saved digest, never re-sign.

This bootstrap contains working native Rust invoice signing, validation,
transaction preparation, merchant verification predicates, durable journals and
an experimental NFC framing implementation. Its default checkout demo remains a simulation:
it uses real Sui transaction signatures but never submits funds to a chain.

## Run the tested checkout

For a guided terminal recording, see the [Omarchy demo](docs/demo.md):

```sh
cargo build --locked -p tap-cli
python3 scripts/demo.py
```

This feeds a fresh laboratory invoice through separate Rust APDU frame/receiver
processes into the actual wallet verification and approval path. Review the terms,
type `APPROVE SIMULATION`, then follow the restart/replay prompts. After the
merchant commits, its receipt is deliberately lost; restart replays the same
BCS, signature and digest with one simulated payment. Claims/receipts use a private
Unix socket, not RF. This is **not a live NFC payment demo**.

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
The default pinned asset is not allowed on testnet. An explicit
`--circle-testnet-usdc` flag selects Circle's official testnet type, bound to the
fixed testnet genesis. See
[qualification and fresh-wallet prerequisites](docs/testnet-qualification.md).
Legacy signing/verification defaults and the offline demo retain their existing
asset policy. No automatic asset substitution, conversion or broadcast is provided.

`testnet conversion inspect --sender <PUBLIC_ADDRESS>` now discovers the smallest
eligible owned Circle testnet coin and checks an unsigned whole-coin conversion
to that **same owner's** address balance. It cannot load keys, sign or submit.
The human reports private setup verification and Circle faucet completion;
public inspection found a 20 USDC coin, not merchant-payment eligibility.
The reviewed false blocker is fixed: only the singleton read-only deny-list
`PerEpochConfig` at `0x403` is allowed, with no changed-object exceptions. One
new checked unsigned inspection passed. Native conversion `prepare`, `submit`
and `reconcile` now implement separate durable approval/recovery and exact-artifact
dispatch. Independent lifecycle review completed on 28 October 2026
(`ses_e7c57274cffe7qT1NSnKx1gggB`), with no demonstrated P1/P2 finding;
the prior read-only `0x403` P2 is resolved. The human subsequently completed the
20 USDC same-owner conversion: saved keyless reconciliation verifies checkpoint
`400301849`, epoch 1267. Saved public qualification shows 20,000,000 units of
address balance and a successful checked 10,000-unit payment simulation.
The subsequent actual merchant checkout is verified in the milestone above;
conversion and eligibility alone were not settlement. See the
[conversion workflow and evidence](docs/testnet-qualification.md#native-conversion-lifecycle-review-completed).

The new `scripts/pcsc-lab.py --prefix ... --invoice PUBLIC_FILE --out NEW_INBOX`
mode delivers only: native `nfc-push` → private PC/SC/vpcd → thin VICC → Rust
receiver. It publishes exact bytes only after ACK, receiver exit and cleanup,
with status `APDU_DELIVERED_NOT_APPROVED_NOT_PAID`. No signing, RPC or wallet
automation is added. Independent delivery/workflow review completed 28 October
2026 (`ses_e7d4e67afffeE9pFbLkvW2fFdS`), with no blocking/material P1/P2.
The [recorded human checkout](docs/testnet-qualification.md#private-pcsc-testnet-checkout-human-only-review-completed)
is now complete; its supplied receiving address must not be reused.

## Disposable testnet setup (source reviewed; human verification reported)

`testnet init` now provides the local setup rather than requiring an installed
Sui CLI. Focused init-only review completed on 28 October 2026
(`ses_e7d355137ffeaXiT10YXbCLGve`) with no blocking/material P1/P2 finding.
This is **not** a production security audit or positive generation proof.
Human init and private `verify-setup` have now been reported successful;
the agent did not verify private SDK key matches. Read-only `verify-setup` review also
completed on 28 October 2026 (`ses_e7d355137ffeaXiT10YXbCLGve`), with no
blocking/material P1/P2 finding. Source and public-fixture evidence covered key
matches, tamper rejection, immutability and absence of key payload in output;
the user's wallet was not accessed. This is not freshness, funding, eligibility
or settlement proof. The following is the human-private **unfunded** workflow,
with recording/agent access disabled;
the absolute parent must already be owned, canonical and mode 0700, and the new
leaf must not exist:

```sh
target/debug/linux-tap testnet init --directory /absolute/private-parent/fresh-testnet
target/debug/linux-tap testnet verify-setup --directory /absolute/private-parent/fresh-testnet
cat /absolute/private-parent/fresh-testnet/READY.json
```

Type `CREATE DISPOSABLE TESTNET WALLETS` when prompted. The command creates three
SDK/OS-entropy Ed25519 keystores (sender, merchant invoice signer, receiver),
mode 0600 inside the new mode-0700 directory. Only public addresses, merchant pin
and paths are printed/published; **never print or share the keystore contents**.
`READY.json` means provisioned only, not funded, eligible or paid. On STOPPED or
interruption retain the partial directory, do not fund/use it or overwrite/retry
it. The receiver keystore remains yours; the merchant observer does not need it.

**Already completed init and verify-setup? Do not rerun them or regenerate keys.**
For an existing setup still awaiting private verification, run only
`verify-setup` against that directory, privately.
It reads the three fixed role files under checked process dump protection, uses
the official decoder and compares all public keys/addresses with READY. It writes
no files, signs nothing and calls no RPC. Output is public-only
`KEY_MATCHES_ONLY_NOT_FUNDED_OR_ELIGIBLE`; READY itself is unchanged.

**Before any funding**, the human must verify successful init exit status,
owned mode-0700 directory/mode-0600 regular files (no symlinks; link count 1), and locally decode
each keystore through a successful `verify-setup` result to match its public
address and merchant pin against READY. Share only public/pass information,
never key contents. Actual user-wallet verification is human-private and human-reported;
the agent must not run this command on the user's real wallet directory.

Only after those checks, privately open [Circle's faucet](https://faucet.circle.com/), select **USDC /
Sui Testnet**, use only the public sender address and complete CAPTCHA yourself.
Usable **address balance**, not just coin objects, is required; conversion is
still a human-reviewed blocker, never automatic. No blanket SUI requirement is
added. [Setup details and public-manifest checkout configuration](docs/testnet-qualification.md#fresh-disposable-wallet-setup-human-operated-not-performed-by-the-agent).
The generator was compiled and negative guards tested, but its fresh-entropy
path was deliberately **not run by the agent**. Unencrypted software custody,
memory erasure, backups and live settlement are not qualified.

## Explicit testnet preparation checkpoint

`linux-tap testnet invoice --help` and `linux-tap testnet prepare --help` expose
the fixed Circle Sui testnet profile without changing draft-v0 wire bytes.
Preparation consumes the completed APDU inbox plus an independently supplied
merchant public-key pin. It freezes one candidate, requires typed terminal
approval, qualifies those exact bytes against an explicitly trusted testnet node,
and commits an issuance reservation **before** loading/signing with a dedicated
legacy Sui Ed25519 keystore. Candidate, approval, recovery approvals and immutable
artifact are separate versioned journal records; the exported `Prepared` shape
is unchanged. `verify-on-node --circle-testnet-usdc` uses the same explicit policy.

`--offline-fixture-epoch` is restricted to the existing public merchant/sender
fixtures and still requires terminal approval. It never claims live eligibility.
An unresolved reservation requires explicit `--recover-saved-candidate` approval;
only the saved candidate and same deterministic software key can be used.
An existing artifact is verified/re-exported without new signing or freshness
checks, not replaced with a new nonce/epoch. It is **PREPARED_NOT_PAID**, never
ready to submit; any future dispatch must requalify separately.

The original preparation checkpoint had only offline public-fixture evidence;
the 29 October milestone additionally verifies actual human-approved preparation.
Independent re-review resolved its record-integrity P2 with no remaining finding.
Do not use agent-accessible existing wallets or real secrets
for this exercise. Keystore paths must be explicit/absolute, owned regular files
mode 0600, with an owned canonical mode-0700 parent; no default wallet search,
symlinks, encryption or production custody is supplied. The
official SDK decodes legacy entries; memory erasure of JSON/SDK temporaries is
**not guaranteed**. Two controlled testnet payments and scoped application-boundary
recovery are verified; full receiving custody, backups and wire/host/power-failure
qualification remain pending. Hardware
remains deferred; the first emulated end-to-end milestone does not close those gates.

## Controlled testnet dispatch and merchant reconciliation

The controlled implementation provides `testnet register`, `submit`, `reconcile`,
`accept` and `confirm`. Its bounded independent review is complete, with the
qualification-record and self-transfer P2s resolved; this is not live proof.
Init-only source review is also complete and human init reported successful;
verify-setup source/public-fixture review is complete, and human-private
verification has been reported successful. Same-owner conversion and public
payment eligibility and one actual merchant checkout are evidenced.
See [the runnable, human-operated flow](docs/testnet-qualification.md#controlled-submission-and-independent-merchant-flow)
for actual APDU ingress → preparation → dispatch → independent merchant credit →
wallet association commands. The recorded run completed payment, merchant credit
and wallet association; do not create another transfer from that historical recipe.

Submission is keyless and uses only the approved immutable journal artifact.
Typed `TESTNET SUBMIT` approval, fresh exact-candidate qualification and a committed
dispatch-intent/attempt barrier precede the official SDK execution helper, whose
whole operation has a 60-second deadline. Restart queries the saved digest before
checking eligibility. No timeout, NotFound or verified failure creates a new
transfer. Explicit `--replay-saved` can only resend identical bytes after another
lookup, qualification and typed approval. Offline/default public fixtures are
never submittable. There is no mainnet submission mode.

A separate keyless merchant ledger reserves a receiver for exactly one invoice,
queries its independently chosen fixed-genesis testnet node, and commits one
credit/receipt per invoice and chain/digest. Late successful observations are
retained, not discarded or automatically refunded. `confirm` independently checks
the caller-pinned merchant node before wallet association. Private receipt files
are not cryptographically authenticated remote business acknowledgements; this
is a local artifact-file workflow, not a new merchant daemon or live PC/SC demo.

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

For real PC/SC without hardware, the separately opted-in [private virtual lab](docs/pcsc-lab.md)
builds pcsc-lite/vpcd locally and uses Unix-only private IPC. It qualifies the
same `nfc-push` → Rust receiver → wallet recovery flow, not RF or live payments.
Default checks never download or start the lab.

`adapters/nfcd/bridge.py` exports Sailfish's existing LocalHostApp D-Bus interface
and forwards APDUs to the Rust receiver. It contains no keys or payment logic.
It requires a functioning nfcd hardware adapter; it does not magically connect
nfcd to the NXP vendor stack. See [hardware qualification](docs/hardware.md).

## What is deliberately unfinished

- Broader asset/network qualification and wire/host/power-failure durability.
  The explicit Circle testnet profile has two actual controlled payments and
  scoped application-boundary recovery, not blanket live-network qualification.
- Hardware signer implementations and recovery, encrypted production keystore.
- Physical NFC qualification and a native phone driver path.
- Native Qt UI, commercial POS integration, production refunds and accounting.
- Cryptographic verification of network finality independent of an RPC provider.

There is a guarded testnet-only broadcast command, not a production-ready wallet.
There is no foundation account or central registry.
Unmodified card terminals cannot accept this protocol without an acceptance
integration. Open software does not make USDC issuance or NFC firmware decentralized.

## Architecture and next work

- [Proposed product architecture](docs/product-architecture.md) and [product design](docs/product-design.md)
- [Reuse audit and fork decisions](docs/reuse.md)
- [28 October research: reuse, Linux phones and transport security](docs/research/README.md)
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
