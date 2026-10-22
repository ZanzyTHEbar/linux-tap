# Validation record

Bootstrap assembled on 22 October 2026 on x86-64 Linux using Rust 1.99.1 and
the SDK commit pinned in Cargo.toml. ARM64 compilation is a future qualification
step. Cargo.lock is committed; dependency license metadata is recorded separately.

## Checks completed

- Rust compilation and Clippy with warnings denied.
- Fifteen unit tests across invoice validation, signatures, APDU framing,
  template checks, trusted-node receipt predicates and durable journal recovery.
- Separate wallet and merchant process smoke test over OS pipes, including an
  interrupted wallet, merchant restarts, identical retries, and exactly one
  prepared record and simulated payment.
- Optional pcsc-reader feature compilation and unit tests against the installed
  libpcsclite shared library. No reader was attached. This environment required
  a private development linker symlink because the development pkg-config file
  was absent; ordinary Arch builds should use the actual pcsclite package.
- nfcd bridge import, upstream D-Bus method signatures and Rust subprocess APDU
  roundtrip with exact output persistence. No system D-Bus registration occurred.
- Twenty-five included upstream guidance files checked against their recorded
  SHA256 hashes. Cargo metadata inventory includes 198 resolved packages.

## Recording-demo increment (22 October 2026)

On this increment's x86-64 Linux runner, using pinned Rust 1.99.1:

- `bash scripts/check.sh`: passed formatting, Clippy with warnings denied, all
  15 existing tests, CLI build and the separate pipe smoke test.
- `python3 scripts/demo.py --approve-simulation`: passed against real private
  Unix sockets with separate native Rust merchant and wallet processes. Before
  sending, the merchant had zero payment records. Both processes restarted;
  the entire stored signed payload (not only its digest) matched on resume and
  replay. Exactly one prepared transaction, simulated payment and matching wallet
  receipt remained. APDU reconstruction passed in software (seven frames).
- A temporary PTY harness ran `python3 scripts/demo.py` with actual terminal
  input, approved the displayed terms and continued both restart prompts: passed.
  A second run typed `NO`: exited nonzero before creating a wallet journal, with
  no PASS result. This verifies host-terminal approval, not a hardware display.
- `python3 -m py_compile scripts/demo.py` and `git diff --check`: passed.

The demo retains per-run journals, merchant log and result JSON in its printed
private `/tmp/ltap-*` directory; these are temporary software artifacts, not
durable database backups. PTY transcripts for this run are local evidence only,
not a published recording. No new test suite, dependency, reader or vendor code
was added. The prior PC/SC compilation record above is not a new hardware result.

This host is not an Omarchy desktop qualification. No real display/recorder UX,
ARM64 build, live-node request, funds movement or physical NFC was exercised by
this increment. Missing finalized charter/specification traceability and remaining
mandatory capabilities are listed in [the demo gap analysis](demo.md).

## Testnet qualification increment (22 October 2026)

- `bash scripts/check.sh`: passed formatting, Clippy with warnings denied, all
  15 existing tests, CLI build and native pipe smoke (`chain_submission: false`).
  No new helper unit tests or mock eligibility claims were added.
- `cargo build --offline -p tap-cli`: passed after reusing the already-locked
  serde_json dependency in tap-sui; no dependency versions changed.
- Live `probe --endpoint https://fullnode.testnet.sui.io:443`: succeeded, genesis
  `69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD`, epoch 1260.
- Live `qualify-testnet` with that full chain pin and diagnostic `--sender 0x0
  --receiver 0x1 --amount 10000`: **failed qualification as intended**, exit 1.
  Protocol 139 enabled gasless, but the pinned asset was absent from the token
  allowlist, coin metadata lookup returned NotFound, address-balance funding
  was unconfirmed, and checked simulation rejected the asset as not allowed.
  The [qualification guide](testnet-qualification.md) records the exact command
  and [saved live evidence excerpt](evidence/testnet-qualification-2026-10-22.json).
- The same live command with full all-zero digest
  `11111111111111111111111111111111`: rejected with `chain pin mismatch`, exit 1,
  before protocol, balance or simulation calls.
- The saved evidence excerpt was compared field-for-field with the full local
  live report, including the retained protocol fields and exact BCS: matched.
- `git diff --check`: passed.

This is real gRPC/protocol/rejection evidence, **not a successful simulation or
execution**. No signatures, user keys, faucets, asset substitution, funding or
broadcasts were used. Submission/reconciliation work stopped at the failed gate.
Successful simulation response validation compiles but lacks live success
evidence. The earlier mock receipt predicates remain mock evidence only.
Reader/vendor code was unchanged; PC/SC and physical hardware were not retested.

## Earlier bootstrap environment limitations

The earlier bootstrap runner rejected Unix socket binding with EPERM, so that
run covered pipes only. The recording-demo increment above successfully exercised
socket binding on its runner. Test the socket path on the intended Omarchy host;
the demo deliberately fails rather than silently falling back to pipes.

The earlier read-only testnet gRPC probe failed at DNS resolution. The increment
above supersedes that connectivity limitation, not the missing live execution
or settlement evidence. The RPC receipt predicate still has only unit fixtures
and compilation evidence; no live receipt has been verified.

## Not tested or implemented

No mainnet or testnet funds were sent. No physical NFC exchange, trusted hardware
approval, phone driver, firmware flash, production key recovery, UI, commercial
POS integration or live refund was tested. The two wallets required for an
independent interoperability milestone do not exist yet: the current demo has
one wallet implementation and one separate merchant process.

The native USDC gasless shape is experimental. Offline encoding and signing do
not establish live eligibility or prove that a customer holds usable address
balances. The captured offline-builder gap is reproducible, but no upstream
issue or patch was published.

Run scripts/check.sh to repeat the default checks. With PC/SC development files
installed, run cargo test --workspace --features pcsc-reader --locked. For the
optional bridge, install adapters/nfcd/requirements.txt into a venv and run
that venv's Python on scripts/test_nfcd_bridge.py.
