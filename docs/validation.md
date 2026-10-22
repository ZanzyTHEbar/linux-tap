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

## Earlier bootstrap environment limitations

The earlier bootstrap runner rejected Unix socket binding with EPERM, so that
run covered pipes only. The recording-demo increment above successfully exercised
socket binding on its runner. Test the socket path on the intended Omarchy host;
the demo deliberately fails rather than silently falling back to pipes.

The read-only testnet gRPC probe was attempted and failed at DNS resolution in
this environment. No live chain identity, balance, transaction eligibility,
simulation, execution or settlement was verified. The RPC predicate has unit
fixtures and compiles, but must be qualified against a real node.

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
