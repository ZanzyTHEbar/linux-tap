# Private virtual PC/SC lab

Stage B uses the **existing** Rust merchant `nfc-push`, real pcsc-lite resource
manager and upstream vpcd IFD driver. A thin Python VICC forwards APDUs to the
existing `apdu-stdio` Rust receiver. It never builds an invoice, verifies terms,
signs, touches the wallet journal or implements a second wallet. The existing
`demo.checkout` then consumes the exact inbox and checks approval, lost receipt
after merchant commit, restart and identical signed-artifact replay.

**Qualified profile:** Ubuntu 26.04 x86-64, Rust 1.99.1, virtual single-slot T=1,
ATR `3b800181`, public development fixtures / `DevelopmentSoftware` approval.
No reader hardware or firmware; no RF, HCE, production custody, power-cut or live
settlement qualification. Claim/receipt transport remains a private Unix socket.
Default CI/checks remain offline and never build/download/start this lab.

## Reproduce explicitly

Requires existing Python 3, C compiler, flex, Ninja, pkg-config, patch, curl,
dpkg-deb, tar, flock and the project's pinned Rust toolchain. No sudo or system
installation. Choose a new, private, absolute cache path without whitespace,
colons or symlink aliases (short enough for Unix socket/reader-name limits).

```sh
bash scripts/pcsc-lab-build.sh $PCSC_CACHE
TMPDIR=/tmp/opencode CARGO_NET_OFFLINE=true \
  PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig \
  cargo build --locked -p tap-cli --features pcsc-reader
python3 scripts/pcsc-lab.py --prefix $PCSC_CACHE/prefix
```

The last command uses the existing interactive Rust approval prompt. For explicit
public-fixture-only unattended acceptance add `--approve-simulation`. This is
not silent production approval. The runner never downloads. Dependency setup is
the separate opt-in build command. If the locked optional pcsc crates are not
cached, one normal `cargo build --locked -p tap-cli --features pcsc-reader` with
the same `PKG_CONFIG_PATH` needs network access; no lock/toolchain upgrade.
Running default `scripts/check.sh` rebuilds the default-feature binary; rebuild
`--features pcsc-reader` before running this lab again.

## Public invoice delivery only (new seam; review completed)

The existing cached prefix can deliver an arbitrary public `SignedInvoice`
without running the synthetic checkout or accessing a wallet:

```sh
python3 scripts/pcsc-lab.py --prefix $PCSC_CACHE/prefix --invoice /absolute/private-operation/invoice.json --out /absolute/private-operation/inbox.json
```

`--invoice` excludes `--approve-simulation` and requires `--out`; no terminal
approval is needed for transport. The output must not exist, including as a
symlink, and its canonical absolute parent must be owned mode 0700. The runner
takes a bounded byte snapshot; native `apdu-frames`, `nfc-push` and the real
Rust receiver retain parsing/framing authority. The checked-in native bound is
8192 bytes, with 220-byte chunks (not a newly introduced 2048-byte limit).
Signature trust, chain, date/expiry and payment eligibility are not transport
policy; the later `testnet prepare` uses the independently configured pin.

The same build-hash checks, lock, peer-UID protection, Unix-only daemon/driver,
reset probe, native `nfc-push` and APDU transcript checks are reused. After ACK
and exact-byte comparison, VICC closes the receiver's stdin, waits for exit 0
and records `receiver-completed.json`. Only after child/daemon cleanup succeeds
are complete bytes atomically published without replacement. A private sibling
staging file is fsynced, linked without overwrite and unlinked, then the parent
directory is fsynced; the final mode-0600 regular file has link count 1.
This is not power-cut certification or hostile same-user path protection.

The sole success status is `APDU_DELIVERED_NOT_APPROVED_NOT_PAID`, with
`chain_submission=false`, `wallet_approval=false` and `payment=false`. No wallet
or merchant DB is created, no approval/signing/submission command is invoked,
and no artifact-survival assertion is made by this mode. Reset qualifies only
partial assembly discard. Failure retains private evidence and gives no success
status; an existing output is never replaced. The default fixture checkout and
its artifact/replay assertions remain separate and unchanged.

Public-fixture delivery and the original Stage B acceptance passed locally;
independent delivery/workflow review completed 28 October 2026
(`ses_e7d4e67afffeE9pFbLkvW2fFdS`), with no blocking/material P1/P2. This remains
virtual public-fixture evidence, not actual merchant payment, RF, custody or
power-cut proof. See the [human preparation recipe, steps 1–3 only](testnet-qualification.md#private-pcsc-testnet-checkout-human-only-review-completed).

## Private IPC decision and source review

The user-owned requirement is real PC/SC invoice delivery with **no externally
accessible listener, host service or host reader discovery**. A VM/TCP/VICC OS
stack was an implementation proposal, not that requirement. Prior user-namespace
creation failed `uid_map: EPERM`; no denial workaround or namespace retry was used.

Pinned upstream `virtualsmartcard/src/ifd-vpcd/ifd-vpcd.c` supports reverse mode
via `DEVICENAME hostname:port`; `src/vpcd/vpcd.c::connectsock` normally connects
TCP, while `opensock` binds unauthenticated `INADDR_ANY`. Merely selecting
localhost or a random port would not authenticate the endpoint.

The small GPL-3.0-or-later [patch](../scripts/pcsc-lab-vpcd.patch) instead makes
this lab driver **Unix-only**: server mode fails closed, reverse mode requires an
absolute AF_UNIX path and checks `SO_PEERCRED` UID. VICC binds inside a fresh 0700
directory with a 0600 socket and also checks the daemon peer UID. No TCP branch
is usable; no LD_PRELOAD, proxy, custom PC/SC implementation, VM, AOSP, full VICC
OS or new service is needed. Single-slot upstream framing/power/ATR/IFD logic is
otherwise retained. This trusts the same OS user/root, not a compromised host.
Independent read-only review completed on 28 October 2026
(`ses_e7d4e67afffeE9pFbLkvW2fFdS`) with no blocking or material P1/P2 findings;
see the [Stage B review record](validation.md#private-virtual-pcsc-stage-b-28-october-2026).
It qualifies only private virtual PC/SC, not RF/HCE, hardware approval, power-loss
durability, custody or live chain behavior. Source inspection and socket snapshots
are not full kernel-level proof; hash checks are not signature verification.

pcsc-lite **2.3.3's Meson build**, not its stale Autotools version declaration,
is authoritative here. `meson.build` declares 2.3.3; generated `pcscd.h` sets
`PCSCLITE_IPC_DIR`, `PCSCLITE_RUN_PID` and `PCSCLITE_CSOCK_NAME` to the private
prefix. `pcscdaemon.c` uses those compiled paths; `--config` selects only our
fresh reader directory. USB/libudev/libusb/systemd/polkit are compiled out;
serial reader configuration is enabled solely for the explicit vpcd config.
The private client library is selected with `LD_LIBRARY_PATH`, with its exact
socket set by `PCSCLITE_CSOCK_NAME`; that client variable alone would **not**
relocate a host daemon. No host daemon or reader inventory is queried.

Upstream pcscd explicitly chmods its runtime directory 0755, PID 0644 and socket
0666 despite umask. The enclosing owned prefix remains 0700 throughout, so none
becomes externally reachable. The runner restores runtime 0700 and files/socket
0600 before clients, and runtime 0700 again on cleanup. It refuses an existing
daemon socket/PID, verifies build hashes/configuration, and holds a private lock
to prevent rebuild or competing runs. Do not manually remove stale runtime files
without establishing their ownership/status.

## Sources, checksums and licenses

Only these official read-only downloads are used by the build helper:

| Input | Pin / SHA-256 | Provenance / license retained in private cache |
| --- | --- | --- |
| [pcsc-lite source](https://deb.debian.org/debian/pool/main/p/pcsc-lite/pcsc-lite_2.3.3.orig.tar.xz) | 2.3.3; `cdff7d7153a0b37aa74e26dfec89ec7dc5c5286aa21b91b903e38739d227e8e7` | Matches official [Debian source descriptor](https://deb.debian.org/debian/pool/main/p/pcsc-lite/pcsc-lite_2.3.3-1.dsc); `COPYING`: BSD-3-Clause main code, BSD-2-Clause auth, ISC simclist; examples/spy/tests GPL-3.0-or-later |
| [vsmartcard archive](https://codeload.github.com/frankmorgner/vsmartcard/tar.gz/93286a90d2411120ae17299f9902e1fdd8954185) | revision `93286a90d2411120ae17299f9902e1fdd8954185`; `9f55b62246e745d87e5ce21b6c83e135773906cfa2c9fd02d6b00ff178c55763` | Official upstream immutable revision; vpcd files/patch GPL-3.0-or-later, full source/copyright/COPYING retained; umbrella tree mixed |
| [Meson Debian package](https://deb.debian.org/debian/pool/main/m/meson/meson_1.7.0-1_all.deb) | 1.7.0-1; `e92d04e60e784336883d537755d09b7dcb856809fa318a0931df3898e84ded95` | Extracted locally only; no package scripts/install; Apache-2.0, package copyright retained |

All hashes are checked before extraction. pcsc-lite's source hash was compared
with the official HTTPS `.dsc`; its PGP signature was **not** independently
verified. vsmartcard/Meson hashes pin the observed official HTTPS downloads, not
an independent signing-key verification. `build.sha256` additionally pins the
local patch, generated headers, daemon, client libraries and driver; build logs,
complete source archives/extracted sources and licenses remain under the private
cache. No upstream source tree or binary is vendored into the checkout.

## Evidence, failure and lifetime

Each invocation retains `<PRIVATE_VALIDATION_ROOT>` with `configuration.json`,
build hashes/headers, reader config, process PID/start identities, real pcscd
APDU log, VICC u16BE/control/ATR/APDU transcript, expected Rust frames, exact
invoice/inbox, wallet/merchant logs, journals and cleanup/result JSON. The runner
captures only its daemon's socket inode rows and asserts all are AF_UNIX.
Files are 0600, directories 0700. These are synthetic lab artifacts, not backups.

The native reset probe calls the real library's `SCardReconnect(SCARD_RESET_CARD)`
through a small Linux-only ctypes binding: SELECT/OPEN/first PUT, reset, then
COMMIT must return `6a80`. Invoice delivery itself is **only** the unchanged Rust
`nfc-push`. After checkout a second reset verifies that the ready invoice and
exact approved artifact survive. ATR requests do not reset assembly; OFF/ON/RESET
do. Reads require exact two-byte big-endian lengths with 260-byte request bounds;
receiver work and subprocess operations are deadline-bounded by the lab runner.

`result.json` is written only after the full flow, exact APDU comparison and
owned-child cleanup pass. `application-result.json` covers only the reused
application assertions, not the full Stage B gate. Failures retain `failure.json`
and no final PASS. Cleanup only signals/reaps owned `Popen` children, then checks
all recorded PID/start identities for orphans and private daemon socket removal.
Neither broad PID killing nor host service changes are used.
