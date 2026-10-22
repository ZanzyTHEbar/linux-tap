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
| Durable retry without duplicate signing | Exact signed bytes journaled before sending; simulated merchant ledger | Exit before send, restart both processes, compare stored bytes, replay and retain one simulated payment |
| Reproducible Omarchy demonstration | Existing pipe smoke test; socket transport lacked runtime evidence | Guided private-socket runner and these setup/recording instructions; actual Omarchy host qualification still required |
| Real Sui gasless USDC checkout and settlement | Experimental offline template; read-only RPC predicate | **Missing mandatory live capability.** No eligibility, balance or settlement claim; no broadcast added |
| Production custody, recovery and trusted approval | Public development fixture signer only | **Missing.** Terminal approval is host software, not isolated custody or trusted hardware display |
| Native Qt Quick confirmation | Selected in B21, not implemented | **Missing.** This slice is a native CLI, not the finalized graphical wallet |
| Physical NFC and commercial acceptance | APDU software reconstruction; optional PC/SC client | **Missing.** No physical tap, timing, qualified adapter or POS acceptance claim |
| Independent wallet interoperability | One wallet implementation and one merchant | **Missing.** Separate processes are not two independently implemented wallets |
| Omarchy packaging / merchant SDK product | Rust reference modules; no system package or stable SDK distribution | **Incomplete.** Local source build only; no daemon install, permissions changes or desktop integration |

This implements only the runnable software-demo slice. Live checkout cannot be
completed under this task's no-broadcast/no-spending authorization. The missing
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

1. Run `python3 scripts/demo.py`. The Rust wallet verifies the invoice and shows
   **0.010000 USDC**, the full asset type, recipient and public merchant pin.
   Type `APPROVE SIMULATION` within two minutes; any other response declines.
2. Point out `JOURNALED` and `SIMULATED_INTERRUPT_BEFORE_SEND`: the wallet exited,
   but its signed bytes survived; the merchant has zero payment records.
3. Press Enter. Fresh wallet and merchant processes resume the stored bytes.
   The receipt explicitly says `SIMULATED_PAID_NOT_ON_CHAIN` and `replay: false`.
4. Press Enter again. Show the identical digest, `replay: true` and final **one
   signed transaction / one simulated payment**. The APDU step reconstructs
   invoice bytes in software only; it does not emulate RF or show a physical tap.

Resends use the already approved journal entry; they never sign a fresh payment.
This demonstrates local simulated-ledger idempotency, not onchain invoice
deduplication. The interruption is a controlled exit before send, not a power-loss
or uncertain live-network-outcome test.

For unattended verification of that same socket flow:

```sh
python3 scripts/demo.py --approve-simulation
bash scripts/check.sh
```

The explicit flag permits only this public-fixture simulation, not real signing
or spending. `check.sh` also retains the separate pipe-based smoke scenario.

Each demo run creates a new private mode-0700 `/tmp/ltap-*` directory and prints
its location. It retains wallet/merchant SQLite journals, merchant log, invoice
and `result.json`; it does not overwrite earlier runs or erase their evidence.
These are synthetic fixture artifacts, not database backups. `/tmp` is temporary:
copy the printed directory somewhere durable yourself if you need to retain it.
`result.json` is written only after all evidence checks pass. A failed/declined
run exits nonzero and has no PASS result; inspect its terminal output and log.

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
targeting or merging into it. This work stays on `feat/omarchy-poc-demo` pending
independent review and explicit coordinator merge/target authorization.
