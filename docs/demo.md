# A native Linux checkout worth showing

Record an interrupted checkout that safely resumes the **same signed transaction**
instead of creating another one. This is a software simulation with real Mysten
SDK signatures, not a live payment or physical tap. X here means the social
platform; there is no X-specific hardware target.

## Requirements versus implementation

The checked-in architecture and backlog describe a laboratory milestone of a
larger engineering specification. That finalized specification and foundation
charter are not included in this checkout or available project memory. This is
a source-backed gap analysis, **not certification against missing requirements**.
The coordinator must supply those documents before full requirement acceptance.

| Requirement / capability | Current evidence | Smallest demo treatment |
| --- | --- | --- |
| Native Linux wallet and merchant; no JS chain runtime | Rust `tap-core`, `tap-sui`, `tap-cli`; separate processes | Existing binaries; Python only orchestrates and reads SQLite evidence |
| Merchant authentication and narrow approved terms | Invoice signature checked against a pinned key; exact transaction reconstruction | Display the public laboratory pin, full asset, amount and recipient before typed approval; no verified-business claim |
| Durable artifact retry in the simulator | Exact signed bytes journaled before sending; cooperative DB-file owner lock, not a durable issuance reservation | Exit before send, then lose receipt after merchant commit; restart both processes, compare BCS/signature/digest, replay and retain one simulated payment |
| Reproducible Omarchy demonstration | Existing pipe smoke test; socket transport lacked runtime evidence | Guided private-socket runner and these setup/recording instructions; actual Omarchy host qualification still required |
| Real Sui gasless USDC checkout and settlement | Default asset rejected; explicit Circle testnet profile simulates with an existing funded public address | **Blocked live capability.** See [fresh-wallet prerequisites](testnet-qualification.md); no settlement or broadcast |
| Production custody, recovery and trusted approval | Public development fixture signer only | **Missing.** Terminal approval is host software, not isolated custody or trusted hardware display |
| Native Qt Quick confirmation | Selected in B21, not implemented | **Missing.** This slice is a native CLI, not the finalized graphical wallet |
| APDU invoice-to-wallet ingress | Separate native frame/receiver processes; exact persisted inbox is the wallet input | Verify/approve that invoice and bind the private claim channel to the same envelope; transport ACK is not approval/payment |
| Physical NFC and commercial acceptance | Emulated APDU ingress; optional PC/SC client | **Missing.** No physical tap, timing, qualified adapter or POS acceptance claim |
| Independent wallet interoperability | One wallet implementation and one merchant | **Missing.** Separate processes are not two independently implemented wallets |
| Omarchy packaging / merchant SDK product | Rust reference modules; no system package or stable SDK distribution | **Incomplete.** Local source build only; no daemon install, permissions changes or desktop integration |

This implements only the runnable software-demo slice. Subsequent testnet-only
qualification failed for the default asset. A separately approved explicit
Circle testnet configuration now supports read-only qualification, not a changed
demo or signing policy. A specific live trial still needs separate authorization
after fresh-wallet qualification and review. The missing
production and hardware requirements remain blockers to a full wallet launch.

## Set up on Arch / Omarchy

Use the repository's pinned Rust toolchain through rustup. If development tools
are missing, review and install the standard host prerequisites yourself:

```sh
sudo pacman -S --needed base-devel git python rustup
```

Do not install rustup alongside a conflicting system Rust package without
reviewing your host's package policy. In a terminal, from the repository root:

```sh
cargo build --locked -p tap-cli
python3 scripts/demo.py
```

The first build fetches pinned SDK source and locked crates from the Internet.
After building, the demo itself needs no network, credentials, funded account,
NFC hardware, PC/SC service, Google service or JavaScript runtime. No packages
or system services are installed by the demo. Other Linux hosts need a C compiler,
Git, Python 3 and rustup. The runner expects `target/debug/linux-tap`; do not set
a custom `CARGO_TARGET_DIR` for these instructions.

## Record a 45–60 second walkthrough

Use your normal Omarchy terminal and screen recorder. Enlarge the text, use a
terminal at least 100 columns wide and build before recording. Keep the
`SIMULATION ONLY` label and final limitations visible in the clip.

1. Run `python3 scripts/demo.py`. A fresh invoice passes through Rust-generated
   SELECT/OPEN/PUT/COMMIT frames and a separate `apdu-stdio` receiver. Only after
   receiver exit and every transport ACK does the wallet read that exact inbox.
   Rust verifies the invoice and shows **0.010000 USDC**, full asset/recipient,
   public merchant pin, full genesis, expiry and the frozen candidate digest.
   Type `APPROVE SIMULATION` within two minutes; any other response declines.
2. Point out `JOURNALED` and `SIMULATED_INTERRUPT_BEFORE_SEND`: the wallet exited,
   but its signed bytes survived; the merchant has zero payment records.
3. Press Enter. Fresh processes resume saved bytes with `--replay-only`. The
   merchant commits a simulated payment, then closes without its receipt.
   The wallet deliberately exits nonzero with `SIMULATED_OUTCOME_UNKNOWN`;
   read-only SQLite inspection confirms a merchant payment but no wallet receipt.
4. Press Enter again. Restart both processes and replay identical saved bytes.
   The receipt says `SIMULATED_PAID_NOT_ON_CHAIN` and `replay: true`. A final
   duplicate replay leaves **one signed artifact / one simulated payment / one
   wallet receipt association**. No RF or physical tap is emulated.

Resends use the already approved journal entry; they never sign a fresh payment.
This demonstrates local simulated-ledger idempotency, not onchain invoice
deduplication. Both interruptions are deterministic: pre-send exit and post-commit
lost merchant reply. Neither is power-loss proof or a live-network-outcome test.

The narrow CLI seams are `apdu-frames --input`, `apdu-stdio --out`,
`pay-demo --invoice-inbox` and `serve-demo --drop-receipt-after-commit`.
The inbox is bounded to 8,192 bytes and must match every signed-envelope field
offered on the private return channel before approval/signing. This does not
replace the independent laboratory merchant pin. No file watcher or partial-file
consumption is used; concurrent/power-safe inbox publication remains unqualified.
Legacy pipe/socket checkout commands still work without the optional inbox.

`pay-demo --replay-only` refuses new signing when no saved artifact exists.
Interactive new issuance uses `APPROVE SIMULATION`; interactive saved-artifact
resend uses `REPLAY SIMULATION` and remains available after invoice expiry.
Fresh signing rechecks expiry after approval. A cooperative OS file lock is held
before candidate review/signing; decline can leave an empty journal, but no signed
artifact or receipt. There is no durable approval/reservation/dispatch state or
claim of exactly-once signing across a crash between signing and persistence.

For unattended verification of that same socket flow:

```sh
python3 scripts/demo.py --approve-simulation
bash scripts/check.sh
```

The explicit flag permits only this public-fixture simulation, not real signing
or spending. `check.sh` also retains the separate pipe-based smoke scenario.

Each demo run creates a new private mode-0700 `ltap-*` directory under Python's
temporary directory (`TMPDIR`, normally `/tmp`) and prints its location. It retains
mode-0600 wallet/merchant journals, wallet/merchant logs, APDU frame/receiver output,
source/inbox invoice, child PID/start identities and `result.json`; it does not
overwrite earlier runs or erase their evidence. For this workspace use
`TMPDIR=/tmp/opencode python3 scripts/demo.py`.
These are synthetic fixture artifacts, not database backups. `/tmp` is temporary:
copy the printed directory somewhere durable yourself if you need to retain it.
`result.json` is written only after all evidence checks pass. A failed/declined
run exits nonzero, retains `failure.json` and has no PASS result. The intentional
lost-receipt wallet error is expected only when all surrounding journal checks pass.
Owned child processes are waited/reaped, including on failure; only those children
can be terminated by the runner. Inspect retained output and logs on failure.

Unix socket binding must be allowed. On a sandbox denying it, this demo fails
instead of silently substituting pipes. `python3 scripts/smoke.py` is the separate,
explicit pipe-only check, not evidence for the socket demo. If approval fails,
rerun the script for a fresh checkout; no existing socket or database is reused.

## Suggested X / community caption

> Linux Tap: native Rust checkout for Linux/Omarchy. Approve → interrupt → restart
> → replay the same signed bytes: one simulated payment. Offline demo; no funds,
> live settlement or physical NFC. Independent project, not affiliated with Omarchy
> or Sui.

Attach your own recording and repository URL when ready. Nothing in this task
publishes to X or the community. Do not describe this clip as “tap to pay works,”
“gasless USDC live,” a production wallet, or commercial-terminal compatibility.

## Branch and review policy

`CONTRIBUTING.md` requires evidence and review but specifies no default target
branch or direct-merge permission. `main` exists; that alone does not authorize
targeting or merging into it. The qualification increment stays on
`feat/testnet-qualification`; the coordinator has named local `main` as the
integration target after independent review. No merge, push or PR is authorized.
