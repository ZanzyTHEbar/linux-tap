# Linux-phone NFC: implementation and reuse map

**Source inspection: 2026-10-28.** This is implementation evidence for the next
architecture pass, not another phone survey. Read alongside the
[device/stack report](linux-phones-nfc.md) and [research decisions](README.md).
Local product baseline: `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`.
Only this report was written in this pass; existing dirty documentation was preserved.
No installation, compilation, daemon, simulator, physical NFC or payment test ran.

## Recommendation and evidence boundaries

Keep the Rust verifier, signer and durable payment journal authoritative.
The immediate demonstration remains **emulated invoice APDUs → verified terms →
explicit approval → durable signed bytes → lost receipt/restart → identical replay
→ one simulated receipt**. A transport ACK is neither approval nor settlement.

There are now two concrete native application references, not just daemon APIs:
nfcd's `ndef-share` C utility and Sailfish's Qt/QML `sailfish-nfcshare` plugin.
Reuse their registration/resource lifecycle patterns, not their NDEF application
semantics. LocalHostApp is the selected native-phone seam **on a qualified adapter**.
vpcd is a useful subsequent merchant PC/SC boundary; Casimir remains optional NCI
work. Neither should delay integrating the existing APDU receiver with the wallet.

“Inspected” below means public source was read. No upstream test was executed here.
An application's existence proves neither its installation on a phone nor working
HAL listen routing, RF timing, suspend recovery or Linux Tap interoperability.
No upstream code was copied into product code or added as a dependency in this pass.

## Revision ledger: original upstreams and immutable snapshots

Dates are observed committer timestamps, not support promises. Repository links
identify original sources; snapshot links are immutable file/commit anchors.

| Original upstream | Inspected snapshot / activity | License boundary |
| --- | --- | --- |
| [sailfishos/nfcd](https://github.com/sailfishos/nfcd) | [`6fc7aa906df5b6f2c508aab1832c51827095586e`](https://github.com/sailfishos/nfcd/commit/6fc7aa906df5b6f2c508aab1832c51827095586e), 2026-10-09, 1.2.8 | BSD-3-Clause source headers |
| [sailfish-nfcshare](https://github.com/sailfishos/sailfish-nfcshare) | [`0122b7e825fde33fc9802d65abe208e894f6c666`](https://github.com/sailfishos/sailfish-nfcshare/commit/0122b7e825fde33fc9802d65abe208e894f6c666), 2026-08-04 | BSD-3-Clause LICENSE; retain notices |
| [nfcd-binder-plugin](https://github.com/mer-hybris/nfcd-binder-plugin) | [`ee270922a39d3730226936d7e78896dd508400be`](https://github.com/mer-hybris/nfcd-binder-plugin/commit/ee270922a39d3730226936d7e78896dd508400be), 2026-09-07 | BSD-3-Clause inspected plugin sources; HAL/vendor dependencies separate |
| [nfcd-pn54x-plugin](https://github.com/monich/nfcd-pn54x-plugin) | [`2d95c87fdb03776ef638d3dc37cb996c9cbeb82a`](https://github.com/monich/nfcd-pn54x-plugin/commit/2d95c87fdb03776ef638d3dc37cb996c9cbeb82a), 2021-01-08 | BSD plugin; kernel driver has separate GPL obligations |
| [nfcd-mce-plugin](https://github.com/mer-hybris/nfcd-mce-plugin) | [`87e762e063f9c645762ee7e4bb6f9d1df80a052b`](https://github.com/mer-hybris/nfcd-mce-plugin/commit/87e762e063f9c645762ee7e4bb6f9d1df80a052b), 2022-01-20 | BSD-style plugin; MCE dependency |
| [nfcd-policy-plugin](https://github.com/sailfishos/nfcd-policy-plugin) | [`9543ed3307c3c82ae3b2603fd6d6135459da8944`](https://github.com/sailfishos/nfcd-policy-plugin/commit/9543ed3307c3c82ae3b2603fd6d6135459da8944), 2026-09-23 | BSD-3-Clause SPDX |
| [FuriLabs/nfcd](https://github.com/FuriLabs/nfcd) / [binder fork](https://github.com/FuriLabs/nfcd-binder-plugin) | `7079570c9fffc6f687095f3d179af23c950f1be9` / `e0e9565d3e055f17a043a67054224f0c9c9dadc2`, both 2026-07-12 | nfcd Debian copyright: BSD-3-Clause; audit bundled components separately |
| [FuriLabs/gnome-control-center](https://github.com/FuriLabs/gnome-control-center) | [`c6bf9317d4f5aeeda43eae2128dc67d6d9ed3f4f`](https://github.com/FuriLabs/gnome-control-center/commit/c6bf9317d4f5aeeda43eae2128dc67d6d9ed3f4f), 2026-10-01 | NFC panel SPDX GPL-3.0-or-later |
| [FuriLabs/furios-meta](https://github.com/FuriLabs/furios-meta) | [`2928303ce2d7f7e14c8f460d6baf4fff389fa03b`](https://github.com/FuriLabs/furios-meta/commit/2928303ce2d7f7e14c8f460d6baf4fff389fa03b), 2026-08-28 | Packaging evidence, not a reusable wallet module |
| [linux-nfc/neard](https://github.com/linux-nfc/neard) | [`f4b6db9ec482208079f684a0ed6762fd37371d2b`](https://github.com/linux-nfc/neard/commit/f4b6db9ec482208079f684a0ed6762fd37371d2b), 2026-10-26 | Primarily GPL-2.0-only; inspect per-file exceptions |
| [AOSP NFC/Casimir](https://android.googlesource.com/platform/packages/modules/Nfc/) | [`b01e3991865799e55d095a30a0b86b5f4030b929`](https://android.googlesource.com/platform/packages/modules/Nfc/+/b01e3991865799e55d095a30a0b86b5f4030b929), 2025-03-24 | Inspected Python/Rust/Soong headers Apache-2.0 |
| [AOSP Cuttlefish](https://android.googlesource.com/device/google/cuttlefish/) | [`a1162ca7a4e6297f1699b65052a8c2dd466fd518`](https://android.googlesource.com/device/google/cuttlefish/+/a1162ca7a4e6297f1699b65052a8c2dd466fd518), 2025-03-26 | Inspected control sources Apache-2.0 |
| [frankmorgner/vsmartcard](https://github.com/frankmorgner/vsmartcard) | [`93286a90d2411120ae17299f9902e1fdd8954185`](https://github.com/frankmorgner/vsmartcard/commit/93286a90d2411120ae17299f9902e1fdd8954185), 2026-10-15 | vpcd/virtual ICC GPL-3.0-or-later; umbrella tree mixed |

The AOSP pins resolve the inspected `refs/heads/main`, not the newest release
branch. Their 2025 dates must not be described as 2026 Casimir development.
Auxiliary libncicore/libnciplugin source below was read at moving `master`;
API rate limits prevented acquiring full revision anchors in this pass. Pin before reuse.

## 1. Concrete LocalHostApp application source

### nfcd `ndef-share`: small C reference, not a payment application

Read [`tools/ndef-share/ndef-share.c`][ndef-share] and its [Makefile][ndef-build].
`ndef_share_run_with_daemon` connects generated `handle-*` callbacks, exports a
`GDBusInterfaceSkeleton`, then calls `RegisterLocalHostApp` with the NDEF AID and
`NFC_HOST_APP_FLAG_ALLOW_IMPLICIT_SELECTION` (lines 587–604).
It watches daemon disappearance and runs a GLib main loop. Lifecycle handlers
implement Start/Restart/Stop, selection and Process; NDEF file/read tracking is
application state, not an invoice verifier. Successful final sharing is controlled
by response/read completion and `--keep-sharing`, not payment finality.

The tool optionally calls `RequestMode(CARD_EMULATION, READER_WRITER)` for
`--reader-off`; it requests NFC-A via `RequestTechs(A, -1)` by default.
This is concrete evidence for suppressing reader polling during a listen-only
experiment, **not** justification to turn other phone functionality off silently.
Source-derived command, **not run**: `ndef-share --text 'laboratory' --reader-off --keep-sharing`.
Other parser options are `--uri`, `--media`, and `--verbose` (lines 814–827).

Build: C/GNU Make; `gio-unix-2.0`, `gio-2.0`, `libnfcdef`, `libglibutil`,
`libmagic`, `libdl`; `gdbus-codegen` generates Daemon and LocalHostApp stubs from
nfcd XML. `make release` builds `build/release/ndef-share` in its tool directory.
These are inspected buildfile targets, not verified build instructions for this host.
**Adapt:** export/register/cleanup and daemon-loss handling. **Reject:** NDEF AID,
implicit-selection defaults and file-read semantics as Linux Tap payment policy.

### Sailfish `sailfish-nfcshare`: native Qt/QML application integration

Read [`qmlplugin/ndefapp.cpp`][qt-app], `qmlplugin/nfcshare.cpp`,
`shareplugin/qml/NfcShare.qml`, [`qmlplugin.pro`][qt-build] and RPM spec.
`NdefApp::Private` is a `QDBusAbstractAdaptor` exporting LocalHostApp with explicit
introspection and slots, including `Process` and `ResponseStatus` (lines 281–354).
Its asynchronous sequence is **RegisterLocalHostApp → RequestMode → RequestTechs**;
completion handlers retain resource IDs. The destructor releases tech/mode requests,
unregisters the app and unexports its object (lines 413–546).

`Start`, `Restart`, `Stop`, selection and deselection manage transient NDEF state.
`Process` dispatches its Type-4 file commands; `ResponseStatus` confirms a read
only for a successful matching response ID (lines 737–773). This is a particularly
useful example of why “response sent” and “application operation completed” differ.
`NfcShare` creates a `NdefApp` from NDEF bytes; the QML/share-plugin layers provide
Sailfish sharing UX. There is no Sui signing, transaction approval or durable journal.

Build: qmake, Qt DBus/QML, pkg-config `libnfcdef`; QML module installation at
`org/sailfishos/nfcshare`. The full share integration also depends on Sailfish UI
and sharing packages identified by its RPM, so it is not a portable standalone UI.
**Reuse pattern:** async D-Bus registration, owned request IDs, cleanup and read
acknowledgement separation. **Do not fork the app into a wallet** or require its UI
framework merely to obtain LocalHostApp. BSD licensing permits selective adaptation
with notices; no code was imported here.

### Existing upstream tests: exact meaning and limits

Read [`unit/plugins_dbus_service_local_app/test_plugins_dbus_service_local_app.c`][app-tests].
`test_data_init` constructs `TestAdapter`, advertises reader/card modes, and
`test_activate` creates `TestInitiator`; the harness uses a private test D-Bus.
`test_basic` exercises registration, restart and implicit selection;
`test_process` supplies SELECT/file APDUs, verifies Process arguments and a nonzero
response ID/ResponseStatus; `test_switch` exercises switching apps and deselection.
These are useful daemon contract references with synthetic adapter traffic.
They are **not physical tests, a ready NFCC emulator, or evidence of phone HCE**.
Keep them upstream rather than copying a large mocked suite into Linux Tap.

## 2. Adapter ownership, routing, power and permission boundaries

Actual path: **hardware/HAL I/O → libncicore → libnciplugin → nfcd NfcHost →
LocalHostApp → transport adapter → Rust bounded APDU receiver**.
[`LocalHostApp.xml`][host-xml] supplies lifecycle and
`Process(host,CLA,INS,P1,P2,data,Le) → response,SW1,SW2,response_id`.
An explicitly selected lab AID and flags zero remain preferable to NDEF's implicit
selection. `RequestMode(0x08, 0)` requests card mode; it does not grant approval.

[`libnciplugin/src/nci_adapter.c`][nci-adapter] contains CE/listen mode transitions,
ISO-DEP host creation and a 1500 ms CE-reactivation timer/state machine.
[`nci_initiator.c`][nci-initiator] handles listen A/B ISO-DEP data and sends through
`nci_core_send_data_msg(NCI_STATIC_RF_CONN_ID, …)`. Reuse those lower layers;
do not duplicate reactivation in the wallet. libncicore implements NCI state/control
but requires an `nci_hal.h` implementation; it does not access hardware itself.

**Binder:** [`binder_nfc_adapter.c`][binder] tracks `need_power`, `power_on`,
pending operations and close/reopen completion. `binder_nfc_adapter_power_check`,
`submit_power_request`, cancellation and remote-death handling show that HAL ownership
is an asynchronous lifecycle, not “open a device and forward APDUs.” The plugin
contains HIDL and AIDL implementations; its RPM needs nfcd headers, libncicore,
libnciplugin and libgbinder. A native app can use this without an Android wallet,
but the board's Android-derived HAL/service, vendor configuration and firmware remain
dependencies. A HAL death or daemon restart requires requalification/re-registration,
not automatic payment retries or a new signature.

**Direct character device:** [`pn54x_io.c`][pn54x] uses raw device read/write and
`PN54X_SET_PWR = _IOW(0xe9,0x01,unsigned int)`; power control calls that ioctl.
Its cancellability workaround forks a blocking read process and tears it down on
stop. The README defaults to `/dev/pn54x`, allows `/dev/pn547`, and warns about
32-bit userspace/64-bit-kernel `compat_ioctl`. Preserve framing and lifecycle below
nfcd; don't transplant this old I/O code into the wallet. Device-node access and
matching ioctl ABI must be established on the actual system. Mainline `nxp-nci`
AF_NFC/netlink is **not this character ABI**; a matching NXP chip name is insufficient.

**Phone power policy:** [`mce_plugin.c`][mce] has `RequireUnlock=false` and
`AlwaysOn=false` defaults. It tracks display validity/state and optionally tklock;
`AlwaysOn` disables those tracking conditions. Thus screen/display policy can defeat
an apparently successful application mode request. Don't infer secure approval from
screen unlock, or enable always-on to hide a power/timing defect.
[`policy_plugin.c`][policy] monitors `/var/lib/policy/policy.conf`, group `policy`,
key `NfcEnabled`; a valid false value obtains an `NfcBlockRequest`. Missing/invalid
values do not create a block in the inspected implementation. This is device policy,
not wallet authorization, and should not be silently rewritten by an app.

**D-Bus permissions:** inspected [`org.sailfishos.nfc.daemon.conf`][bus-policy]
allows daemon ownership/callback sending for its daemon identities and public
interface calls under default policy. Separately, `dbus_service_plugin.c` stores
host apps and mode/tech/block requests by client and watches client disappearance.
Its libdbusaccess block/unblock policy defaults to deny except group `privileged`;
without that optional build feature the access helper returns true. Do not confuse
that block policy with a universal privileged-only RegisterLocalHostApp rule.
Validate installed bus policy, daemon build options and service credentials; do not
ship a blanket allow-all system-bus override as an integration shortcut.

## 3. FuriOS: concrete packaging and UI, plus an ownership gap

FuriLabs' pinned [LocalHostApp XML][furi-xml] preserves the useful native callback
contract. Its binder [Debian control][furi-binder] concretely packages dependencies
on libncicore/libnciplugin/libgbinder/nfcd development interfaces.
[`furios-meta/debian/control`][furi-meta] includes NFC packages including a
`nfcd-batman-plugin` reference. This establishes packaging intent, not installation
on every model or successful card emulation. The plugin repository lookup returned
404, so its display/power policy and deployment cannot be established here.

The native [`cc-nfc-panel.c`][furi-panel] is stronger application evidence than a
package list. `cc_nfc_panel_enable_nfc` removes/creates `~/.nfc_disable` and calls
start/stop helpers for **system** `nfcd.service`. [`cc-systemd-service.c`][furi-service]
uses systemd `StartUnit`/`StopUnit` with `replace`; it is not a host-app registration
tool. Panel init checks `/usr/sbin/nfcd` and current service activity.
`ping_andromeda` calls `io.furios.Andromeda.SessionManager.Ping` on the session bus;
when active the panel says **“NFC is not available while Android is running.”**
This is source evidence of intended mutual exclusion, not a verified arbitration
mechanism or proof that simply stopping Andromeda establishes host routing.

Build: NFC panel Meson sources/resources within GNOME Control Center, GTK/Adwaita,
GIO/systemd helpers; GPL-3.0-or-later panel code. Reuse as a behavioral reference,
not Apache wallet source. Its switch updates before error handling, so the visible
state alone is not authoritative evidence of actual hardware readiness.
The [FLX1 adaptation control][furi-board] pins downstream API32/Hybris/vendor/kernel
dependencies. Exact NFCC, HAL version, routing/config, installed policy, firmware,
lock behavior and host SELECT trace remain missing. No native FuriOS custom-AID
application example was established beyond its inherited LocalHostApp stack.

## 4. neard utilities: useful, but on the other stack

Read [`tools/nfctool/main.c`][nfctool], [`test/test-adapter`][near-adapter],
`test/test-tag` and `tools/meson.build` at the pinned neard revision.
Source-derived diagnostic commands, **not executed**: `nfctool --list`,
`nfctool --device nfc0 --enable --poll`, `test/test-adapter list`,
`test/test-adapter powered nfc0 on`, `test/test-adapter poll nfc0 on Initiator`.
The Python helper uses system D-Bus `org.neard.Adapter`, sets `Powered`, and calls
`StartPollLoop`; `test-tag` enumerates/dumps/writes NDEF tags, not LocalHostApp APDUs.
`nfctool` uses kernel netlink for adapters/power/discovery and offers **LLCP** sniffing,
service-name lookup and LLCP link parameters; `--poll=Target` is not generic HCE.
Meson builds `nciattach` and `nfctool`; `snep-send` is an uninstalled internal tool.
Dependencies include GLib, D-Bus and libnl; Python helpers need dbus bindings.
Use these to diagnose kernel reader/LLCP bring-up, not nfcd HAL routing or wallet
acceptance. Reject neard/SNEP as a substitute payment transport or a second daemon
competing for the same controller. Its GPL internals need not enter wallet code.

## 5. Casimir: peer skeleton versus functioning control path

Read pinned [`scripts/t4at.py`][t4at], [`Android.bp`][casimir-bp],
[`Cargo.toml`][casimir-cargo] and the Cuttlefish control files below.
`T4AT.listen` answers NFC-A polling and Type-4 selection; `poll` discovers/selects
a peer. **`T4AT.active` handles `rf.Data` with `pass`: it sends no APDU response.**
Therefore `python3 scripts/t4at.py --mode listen --address 127.0.0.1 --rf-port 7001`
is only a source-derived peer invocation, not a complete invoice emulator.
It imports generated `rf_packets` plus Python modules including `requests`;
packet-code generation/environment must be resolved before even that invocation.
Its `_read` uses ordinary `read(2)`/`read(length)`, not exact-length reads: adapt
stream handling, EOF, bounds, sender selection and deadlines in any real peer.

Synthetic RF frames use **u16 little-endian length + generated RF packet**, not raw
APDUs and not vpcd framing. The server defaults to loopback NCI 7000/RF 7001;
Cuttlefish defaults/configuration differ. A new nfcd integration would require an
NCI socket HAL/I/O plugin and NCI initialization/credits/reset/discovery qualification.
A T4AT RF peer alone does not provide that bridge or validate nfcd routing.

Cuttlefish [`casimir_controller.cpp`][controller] provides a more complete reader:
`Poll → SelectNfcA → SelectT4AT`, then `SendApdu` puts bytes into ISO_DEP `DataBuilder`
and waits for a response. Inspected waits are 10 s poll, 1 s selection and 3 s APDU;
these generous simulator deadlines are not physical NFC timing guarantees.
`Write` uses little-endian lengths; `ReadExact`/`ReadRfPacket` manage partial reads
and deadlines. Broadcast CRC/framing controls are separate from ordinary SendApdu.
[`casimir_control.proto`][control-proto] exposes SendApdu (hex-string list), PollA,
radio, power, broadcast, Init and Close. Its [Soong build][control-build] requires
Cuttlefish libraries, generated PDL C++ packets, protobuf and `libgrpc++_unsecure`.
Do not expose an unauthenticated lab control service to a shared network.

Casimir Soong explicitly supplies `librustutils`; `main.rs` imports its inherited-fd
helper but the inspected Cargo manifest omits that dependency. Cargo also includes
PDL/protobuf/gRPC generation dependencies. Standalone is an upstream goal, not a
verified one-command build here. README routing/config stubs also prevent claims
of complete eSE/routing emulation. **Adapt later:** generated RF/NCI seams and
reader control. **Reject now:** a large AOSP build as prerequisite for wallet demo.

## 6. vpcd: minimal PC/SC seam and mandatory isolation

Read [`ifd-vpcd.c`][ifd], [`reader.conf.in`][reader-conf], [`vpcd.c`][vpcd],
[`VirtualSmartcard.py`][vicc] and [`configure.ac`][vpcd-build].
`IFDHCreateChannel` derives port from channel plus slot; default two slots are
35963/35964. `IFDHCreateChannelByName` treats `/dev/null:0x8C7B` as server mode;
a real hostname such as `127.0.0.1:0x8C7B` means **reverse mode**, connecting outward.
It is not a setting to bind the default server to loopback. `reader.conf.in` also
requires a built/substituted `LIBPATH`; copying `@TARGET@` literally will not work.
Autotools configure supports vpcd host/slot options and pcsc-lite headers; use the
external IFD driver with pcscd, not its optional replacement PC/SC implementation.

Wire contract: **u16 big-endian payload length**, then payload; length-one controls
OFF=0, ON=1, RESET=2, ATR=4. Longer payloads are command APDUs; replies contain full
response APDUs including status bytes. ATR requests return ATR bytes; power/reset
controls change transient card state. Implement exact reads, bounded lengths and
timeouts independently around Rust `apdu-stdio`; don't import its card filesystems,
SAM, passport or crypto. RESET/power loss must never erase approved-payment history.

`opensock` binds **INADDR_ANY**. There is no authentication/TLS at this inspected
wire boundary. Before future startup, use a private VM/network namespace with no
external interface, isolated pcscd socket/config and laboratory keys only; or review
a loopback-bind patch. Reverse mode still requires isolating the virtual ICC listener.
Loopback alone does not authenticate other local processes. No listener was started.
Use GPL-3.0-or-later vpcd as an external component with its license/source obligations;
do not copy GPL transport code into an Apache-only wallet. This is a narrow protocol
adaptation recommendation, not a blanket legal conclusion about any distribution.

## Architect handoff: reuse/adapt/reject and remaining gates

| Piece | Decision / minimum new glue | Still unproven |
| --- | --- | --- |
| Rust receiver/verifier/signer/journal | Reuse; connect invoice consumption to actual approval/replay flow | Integrated emulated lost-receipt milestone |
| LocalHostApp app patterns | Adapt lifecycle/async cleanup only; keep lab AID | Qualified D-Bus/adapter timing and recovery |
| Existing [nfcd bridge](../../adapters/nfcd/bridge.py) | Retain transport-only boundary; later bound subprocess stalls and handle daemon loss | Synchronous callback has no read deadline; no extended APDU support |
| Binder/pn54x + NCI libraries | Reuse externally on a matching board; no new wallet controller stack | Exact HAL/device ABI, routing, firmware, power, permissions |
| FuriOS settings/meta | Reference installed ownership/policy contract; don't copy GPL UI | Andromeda arbitration, batman policy, model-specific HCE |
| neard utilities | Diagnostics only | No LocalHostApp-equivalent path established |
| vpcd | Later thin virtual ICC around same Rust receiver, isolated pcscd | ATR/reset/connect/transmit flow; no RF evidence |
| Casimir | Optional later NCI socket HAL + completed RF peer | Build metadata, generated packets, routing, native nfcd integration |

Bridge-specific caution: its Process reconstructs only the project's short-APDU
profile and does not preserve general Le semantics; it returns response ID zero.
Its registration mode request does not override device blocks/power policy. Add no
human approval/network wait inside Process; handle those after invoice transport.
Reset transient assembly on field/session loss, never the signed-byte replay journal.

Access gaps: GitHub API later returned 403 rate-limit; no bypass/retry loop was used.
Raw authoritative pinned files remained readable. `sailfishos/nfcd-mce-plugin` was
404; the actual inspected source is **mer-hybris/nfcd-mce-plugin**. FuriLabs batman
lookup and `tools/nfctool/README` returned 404. These are path/access observations,
not proof that code does not exist elsewhere. No physical phone or runtime was used.
Local validation: report-relative file links and whitespace only; no runtime resources owned.

[ndef-share]: https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/tools/ndef-share/ndef-share.c
[ndef-build]: https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/tools/ndef-share/Makefile
[qt-app]: https://github.com/sailfishos/sailfish-nfcshare/blob/0122b7e825fde33fc9802d65abe208e894f6c666/qmlplugin/ndefapp.cpp
[qt-build]: https://github.com/sailfishos/sailfish-nfcshare/blob/0122b7e825fde33fc9802d65abe208e894f6c666/qmlplugin/qmlplugin.pro
[app-tests]: https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/unit/plugins_dbus_service_local_app/test_plugins_dbus_service_local_app.c
[host-xml]: https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/plugins/dbus_service/org.sailfishos.nfc.LocalHostApp.xml
[nci-adapter]: https://github.com/mer-hybris/libnciplugin/blob/master/src/nci_adapter.c
[nci-initiator]: https://github.com/mer-hybris/libnciplugin/blob/master/src/nci_initiator.c
[binder]: https://github.com/mer-hybris/nfcd-binder-plugin/blob/ee270922a39d3730226936d7e78896dd508400be/src/binder_nfc_adapter.c
[pn54x]: https://github.com/monich/nfcd-pn54x-plugin/blob/2d95c87fdb03776ef638d3dc37cb996c9cbeb82a/src/pn54x_io.c
[mce]: https://github.com/mer-hybris/nfcd-mce-plugin/blob/87e762e063f9c645762ee7e4bb6f9d1df80a052b/src/mce_plugin.c
[policy]: https://github.com/sailfishos/nfcd-policy-plugin/blob/9543ed3307c3c82ae3b2603fd6d6135459da8944/src/policy_plugin.c
[bus-policy]: https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/plugins/dbus_service/org.sailfishos.nfc.daemon.conf
[furi-xml]: https://github.com/FuriLabs/nfcd/blob/7079570c9fffc6f687095f3d179af23c950f1be9/plugins/dbus_service/org.sailfishos.nfc.LocalHostApp.xml
[furi-binder]: https://github.com/FuriLabs/nfcd-binder-plugin/blob/e0e9565d3e055f17a043a67054224f0c9c9dadc2/debian/control
[furi-meta]: https://github.com/FuriLabs/furios-meta/blob/2928303ce2d7f7e14c8f460d6baf4fff389fa03b/debian/control
[furi-panel]: https://github.com/FuriLabs/gnome-control-center/blob/c6bf9317d4f5aeeda43eae2128dc67d6d9ed3f4f/panels/nfc/cc-nfc-panel.c
[furi-service]: https://github.com/FuriLabs/gnome-control-center/blob/c6bf9317d4f5aeeda43eae2128dc67d6d9ed3f4f/panels/nfc/cc-systemd-service.c
[furi-board]: https://github.com/FuriLabs/adaptation-furiphone-krypton/blob/cff8378ddd9d5d55ce86c3bbf5a16b1cf143fdd7/debian/control
[nfctool]: https://github.com/linux-nfc/neard/blob/f4b6db9ec482208079f684a0ed6762fd37371d2b/tools/nfctool/main.c
[near-adapter]: https://github.com/linux-nfc/neard/blob/f4b6db9ec482208079f684a0ed6762fd37371d2b/test/test-adapter
[t4at]: https://android.googlesource.com/platform/packages/modules/Nfc/+/b01e3991865799e55d095a30a0b86b5f4030b929/libnfc-nci/tools/casimir/scripts/t4at.py
[casimir-bp]: https://android.googlesource.com/platform/packages/modules/Nfc/+/b01e3991865799e55d095a30a0b86b5f4030b929/libnfc-nci/tools/casimir/Android.bp
[casimir-cargo]: https://android.googlesource.com/platform/packages/modules/Nfc/+/b01e3991865799e55d095a30a0b86b5f4030b929/libnfc-nci/tools/casimir/Cargo.toml
[controller]: https://android.googlesource.com/device/google/cuttlefish/+/a1162ca7a4e6297f1699b65052a8c2dd466fd518/host/commands/casimir_control_server/casimir_controller.cpp
[control-proto]: https://android.googlesource.com/device/google/cuttlefish/+/a1162ca7a4e6297f1699b65052a8c2dd466fd518/host/commands/casimir_control_server/casimir_control.proto
[control-build]: https://android.googlesource.com/device/google/cuttlefish/+/a1162ca7a4e6297f1699b65052a8c2dd466fd518/host/commands/casimir_control_server/Android.bp
[ifd]: https://github.com/frankmorgner/vsmartcard/blob/93286a90d2411120ae17299f9902e1fdd8954185/virtualsmartcard/src/ifd-vpcd/ifd-vpcd.c
[reader-conf]: https://github.com/frankmorgner/vsmartcard/blob/93286a90d2411120ae17299f9902e1fdd8954185/virtualsmartcard/src/ifd-vpcd/reader.conf.in
[vpcd]: https://github.com/frankmorgner/vsmartcard/blob/93286a90d2411120ae17299f9902e1fdd8954185/virtualsmartcard/src/vpcd/vpcd.c
[vicc]: https://github.com/frankmorgner/vsmartcard/blob/93286a90d2411120ae17299f9902e1fdd8954185/virtualsmartcard/src/vpicc/virtualsmartcard/VirtualSmartcard.py
[vpcd-build]: https://github.com/frankmorgner/vsmartcard/blob/93286a90d2411120ae17299f9902e1fdd8954185/virtualsmartcard/configure.ac
