# Native Linux phones, NFC and an emulator-first Linux Tap path

**Read-only research verified 28 October 2026.** Audience: Linux Tap implementers
choosing an APDU integration, not consumers choosing a daily-driver phone.
Local baseline supplied for this work: clean `main`, `f6ea6a8`. No product code,
package installation, build, daemon startup, flashing, physical test, account,
funds movement or network payment was performed for this report.

## Decision in one page

The immediate milestone is **emulated APDU invoice → real wallet verification
and explicit approval → restart/unknown acknowledgement → replay the same
signed bytes → one simulated receipt**. “Real wallet” here means the actual
native verification, signing and journal implementation with laboratory keys,
not a funded production wallet. It does not mean live payment or RF proof.

1. Keep the existing Rust wallet, merchant and bounded APDU receiver. Connect
   the emulated APDU input to the wallet's actual invoice-consumption path;
   don't merely run a separate APDU reconstruction demonstration beside an
   unrelated checkout.
2. For a real merchant API boundary without RF, reuse **pcsc-rust → pcsc-lite →
   vsmartcard's vpcd → a small transport-only virtual ICC adapter → Rust
   `apdu-stdio`**. This adds meaningful PC/SC coverage without a phone or kernel
   driver. Keep vpcd isolated: its inspected C server binds `INADDR_ANY`, despite
   packaging descriptions referring to localhost.
3. Keep **Sailfish nfcd LocalHostApp** as the best already-selected native Linux
   host-APDU interface. Its useful hardware stack includes `libnciplugin`,
   `libncicore` and an actual I/O plugin. Installing the daemon alone does not
   create an NFC adapter.
4. For eventual RF qualification, prefer a **PN7160 development board with the
   pinned NXP Linux stack**, or an already-owned **Sailfish Xperia with a working
   NFC adaptation**. These are candidates, not demonstrated Linux Tap devices.
   OnePlus 6/6T are excellent mainline-oriented Linux application hosts, but
   their upstream NFC driver path is not the Sailfish character-device path and
   does not expose a ready-made generic ISO-DEP host-card application API.
5. Do not buy a PinePhone, PinePhone Pro or Librem 5 expecting built-in NFC.
   PinePhone Pro production is discontinued. Do not pick PN7150 for a new design:
   NXP now marks it **Not Recommended for New Designs**.
6. Investigate **Casimir** only as a second-stage NCI simulator. It is native Rust
   and can run standalone; it is not necessary to deploy Android to reuse it.
   Its current source/build metadata and incomplete routing implementation need
   qualification. Android Studio's ordinary emulator still documents no NFC,
   while **Cuttlefish Android 15+ explicitly supports Casimir HCE APDU injection**.
   “Android emulation cannot do NFC” is therefore too broad and stale.

The evidence behind these decisions follows. No row below establishes working
Linux Tap HCE on a physical phone.

## Evidence rules and revision anchors

“Native Linux” includes conventional Linux userspace on mainline-oriented phone
ports and native Sailfish/Ubuntu Touch applications on Android-derived board
support. The latter must be labelled **downstream/BSP + HAL**, not mainline.
Android, /e/OS, Volla OS, LineageOS and an Android app in Waydroid do not become a
native Linux wallet merely because their kernel is Linux.

This report distinguishes:

- **Source evidence:** a device-tree node, driver, callback or API exists.
- **Upstream-reported testing:** its author reports a test; not reproduced here.
- **Local compilation/runtime:** none for this research. Existing repository
  documentation records earlier checks, not checks performed by this worker.
- **Physical/RF evidence:** none. Controller feature lists are not host-routing,
  RF timing, suspend/resume or terminal interoperability evidence.

GitHub API responses inspected on 2026-10-28 supplied these anchors. Commit dates
are source-reported committer timestamps, not release promises or independent
proof of activity. `pushed_at` can include other branches/tags: nfcpy illustrates
why it must not be substituted for the default-branch commit date.

| Source | Inspected revision/date | Significance |
| --- | --- | --- |
| [Linux](https://github.com/torvalds/linux/commit/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d) | `65ba8f34ec886ec72b2c07ddc8cf3d815c32869d`, 2026-10-28 | Mainline source snapshot, not a claim about the distro kernel on a phone |
| [Sailfish nfcd](https://github.com/sailfishos/nfcd/commit/6fc7aa906df5b6f2c508aab1832c51827095586e) | `6fc7aa…`, 2026-10-09, version 1.2.8 | Matches repository lock |
| [nfcd binder plugin](https://github.com/mer-hybris/nfcd-binder-plugin/commit/ee270922a39d3730226936d7e78896dd508400be) | `ee2709…`, 2026-09-07, version 1.2.2 | Android HAL I/O adaptation remains active |
| [nfcd pn54x plugin](https://github.com/monich/nfcd-pn54x-plugin/commit/2d95c87fdb03776ef638d3dc37cb996c9cbeb82a) | `2d95c8…`, 2021-01-08, version 1.0.3 | Old direct-character-device integration; needs ABI review |
| [NXP PN7160 branch](https://github.com/NXPNFCLinux/linux_libnfc-nci/commit/d5af4ef88244b9f962986b39eb4126a8c1c0aead) | `d5af4e…`, 2024-08-27 | Matches repository lock; do not silently use master |
| [neard](https://github.com/linux-nfc/neard/commit/f4b6db9ec482208079f684a0ed6762fd37371d2b) | `f4b6db…`, 2026-10-26 | Calling neard abandoned is not supported by current source |
| [vsmartcard](https://github.com/frankmorgner/vsmartcard/commit/93286a90d2411120ae17299f9902e1fdd8954185) | `93286a…`, 2026-10-15 | Current tree differs from Debian's 3.3 source package |
| [libnfc](https://github.com/nfc-tools/libnfc/commit/3fa0751ad58fb0053d3de2e61bbbd4066259104c) | `3fa075…`, 2025-03-05 | Mature but comparatively slow-moving |
| [nfcpy](https://github.com/nfcpy/nfcpy/commit/8a195262fe86167845d282ba5bc9cdaadf6441c4) | `8a1952…`, 2022-03-10, release 1.0.4 | API reported pushes in 2024, but inspected default HEAD is 2022 |
| [pcsc-rust lock](../../deps/upstreams.lock.json) | `822b83e15bb0cfc0f492a17ec9eeb4c4355b3089` | Keep the project's pin; upstream API reported repository push 2024-12-14 |

Moving web pages and unpinned auxiliary source links below were inspected on
2026-10-28; pin them before reuse.
Search snippets were discovery aids only. For example, an August 2026 Fairphone
patch describes Linux 7.1.2; that is the author's reported test environment, not
a kernel version installed or tested here. No later-than-host-date search
result is used as evidence of present support.

## Capability ladder: what “NFC works” must actually mean

The required phone role is a **PICC/card listening over ISO-DEP**, receiving
merchant command APDUs and returning response APDUs. A merchant is the reader/
PCD. A useful qualification ladder is:

1. Correct controller, antenna, clock, power, IRQ and bus are identified.
2. Driver can reset and exchange NCI/vendor frames; required firmware/config is
   available for that exact board and legally distributable where necessary.
3. Reader polling and reading a tag work.
4. ISO-DEP **listen** activation and host routing work, not just NFC-DEP peer mode
   or routing to a UICC/embedded secure element.
5. Native userspace receives arbitrary application APDUs for the lab AID.
6. Response timing, chaining, field loss, reselect, screen lock and power states
   are measured with the complete adapter.
7. Those bytes reach the wallet's verifier; approval and journal replay retain
   their security properties.

A green NFC box in a distro wiki often establishes only step 3. Hardware HCE
capability establishes neither step 4 on a new kernel nor step 7. An embedded
secure element is not required for software HCE and does not establish trusted
transaction approval or Sui signing compatibility. NDEF Type 4 uses ISO-DEP
APDUs, but a fixed NDEF service is not automatically an arbitrary application
APDU service. LLCP/NFC-DEP is a different protocol, not another name for ISO-DEP.

## Device capability matrix

Availability below is a dated public-page observation, not inventory reserved
for this project. A vendor Android support promise does not promise support for
the native Linux port, proprietary NFCC firmware, or a particular Linux ABI.

| Device / native OS option | Availability and lifecycle | Kernel and actual NFC evidence | Reader versus host ISO-DEP / exposed path | Decision |
| --- | --- | --- | --- | --- |
| **PinePhone** / Mobian, postmarketOS | PINE64 maintains a product/store listing; no fixed vendor OS end date established. Mobian lists first tier. | Mainline-oriented Allwinner A64. [PINE64 specifications](https://pine64.org/devices/pinephone/) mention NFC only as a possible pogo-pin extension, not an installed NFCC. | No built-in NFC reader/HCE path. External I²C or USB board requires its own qualification. | Good native UI/application host if already owned; not a built-in NFC purchase. |
| **PinePhone Pro** / Mobian, postmarketOS | [PINE64 blog](https://pine64.org/blog/) explicitly announces discontinuation on 2025-08-14; software/community support is separate. Mobian still lists first tier. | Mainline-oriented RK3399S. [Hardware documentation](https://pine64.org/documentation/PinePhone_Pro/) and inspected [DTS](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/arch/arm64/boot/dts/rockchip/rk3399-pinephone-pro.dts) do not establish an onboard NFCC. | External controller needed; no generic built-in HCE path established. | Reuse an existing one, not a new lifecycle-dependent target. |
| **Librem 5 / USA** / PureOS, Mobian | [Purism product page](https://puri.sm/products/librem-5/) lists Librem 5 in stock, $799+, ten business days, and promises lifetime updates without a fixed calendar date. Mobian lists first tier. | Mainline-oriented i.MX8MQ with distribution patches. Vendor explicitly says **NFC: No**. Its 3FF smartcard reader is a contact interface, not NFC. [USA specifications](https://puri.sm/products/librem-5-usa/) also say no NFC. | No built-in NFC reader or HCE; USB controller possible. | Strong native Linux application host; reject as a built-in tap solution. |
| **OnePlus 6 / 6T** (`enchilada` / `fajita`) / postmarketOS, Mobian | Older secondary-market devices; no current OEM security lifecycle verified here. [Mobian devices](https://wiki.debian.org/Mobian/Devices) places both in first tier. Linux support does not renew vendor firmware maintenance. | Mainline-oriented SDM845 ports. Both include [sdm845-oneplus-common.dtsi](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/arch/arm64/boot/dts/qcom/sdm845-oneplus-common.dtsi): I²C `nfc@28`, `nxp,pn553`, fallback `nxp,nxp-nci-i2c`, enable GPIO12, firmware GPIO62, IRQ GPIO63. | Kernel `nxp-nci` → Linux NFC subsystem → netlink/AF_NFC; not `/dev/pn54x`. Reader-oriented driver evidence, not physical reader testing here. HCE needs a qualified userspace/raw-device adaptation or kernel work. | Best low-cost Linux wallet host candidate; **not** a proven built-in HCE phone. Prefer already-owned unlocked unit. |
| **SHIFT6mq** (`axolotl`) / postmarketOS, Mobian | [SHIFT page](https://www.shift.eco/en/start-page-2-2/) says sold out, with future second-life stock; old marketing still mentions Android 13 “soon,” so don't infer current support from it. Mobian second tier. | [Mainline DTS](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/arch/arm64/boot/dts/qcom/sdm845-shift-axolotl.dts) has the same PN553/fallback at 0x28 and GPIO12/62/63 pattern. | Same kernel NFC versus raw-character-device distinction as OnePlus. No physical HCE evidence acquired. | Alternative if repairability/ownership makes it convenient; not lower-risk HCE integration. |
| **SHIFTphone 8** (`otter`) / emerging mainline port | Current successor platform; exact stock and native-port support contract not verified. | [qcm6490-shift-otter.dts](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/arch/arm64/boot/dts/qcom/qcm6490-shift-otter.dts) exists, but no NFC node matched in inspected file. QCM6490 alone says nothing about NFCC identity. | Exact NFC chip, firmware, downstream DT and native userspace path unresolved. | Watchlist; don't extrapolate PN553 from SHIFT6mq or ST21NFCD from FP5. |
| **Fairphone 4** / postmarketOS, experimental Mobian | Mobian third tier. [Fairphone support page](https://support.fairphone.com/hc/en-us/articles/9979180437393-How-long-will-my-Fairphone-receive-software-and-security-updates) currently guarantees vendor software support at least through 2028. | Mainline-oriented Qualcomm port; inspected [sm7225-fairphone-fp4.dts](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/arch/arm64/boot/dts/qcom/sm7225-fairphone-fp4.dts) has no NFC node match. Exact controller/firmware not established here. | Android NFC specification is not native port qualification. Neither native reader nor host APDU path established by inspected sources. | Not an evidence-backed first HCE target. |
| **Fairphone 5** / postmarketOS, experimental Mobian | Mobian third tier; vendor's [FP5 longevity terms](https://www.fairphone.com/legal/fairphone-5-tagline-explained) promise support through end of 2031. New stock not established; do not confuse long support with current production. | QCM6490 mainline-oriented port. [2026-08-19 patch v2](https://lwn.net/Articles/1089751/) identifies **ST21NFCD**, raw NCI over I²C; existing ST driver NDLC framing is wrong for it. Author reports NTAG215/NDEF URI reading with patches on postmarketOS. Inspected upstream [FP5 DTS](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/arch/arm64/boot/dts/qcom/qcm6490-fairphone-fp5.dts) contains no NFC node match. | Patch author explicitly says **SE/HCE not tested**. Proposed kernel path is NFC subsystem, not nfcd LocalHostApp. Treat as unmerged in this inspected snapshot, not “NFC supported upstream.” | Useful upstream reader-enablement candidate; high-cost first HCE choice. |
| **Fairphone Gen. 6** / developing postmarketOS port | Vendor support page promises 2033; that is stock OS support. | [milos-fairphone-fp6.dts](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/arch/arm64/boot/dts/qcom/milos-fairphone-fp6.dts) exists but has no NFC match. FP5 patch references FP6 NFC work; that reference is not inspected FP6 integration evidence. | Exact controller variant, firmware/config and tested native HCE path remain unqualified. | Watchlist; no HCE purchase recommendation. |
| **Volla Phone Quintus** (`algiz`) / Ubuntu Touch | [Volla](https://volla.online/en/volla-phone-quintus/) offers Volla OS or Ubuntu Touch and advertises NFC. However [UBports device page](https://devices.ubuntu-touch.io/device/algiz/) currently says **Old Releases**, not supported by current Ubuntu Touch release. | Ubuntu Touch is native userspace but commonly Halium/Android BSP adaptation, not evidence of upstream kernel NFC. Exact Quintus NFCC, DT and firmware not established here. Volla OS is Android and must be assessed separately. | Native reader/HCE path unresolved. Vendor NFC listing is not Ubuntu Touch HCE evidence. | Do not choose based on “Linux + NFC” marketing alone; require exact supported OS image and trace first. |
| **Other Volla 22/X23/Plinius and Ubuntu Touch devices** | [UBports promoted devices](https://devices.ubuntu-touch.io/promoted) and per-device pages vary; check current release, not percentage of unrelated features. | Model-specific downstream/Halium versus mainline path must be recorded. No common Volla NFCC identity established. | No universal Ubuntu Touch host-APDU guarantee was found. | Separate device qualification, not a family-level compatibility claim. |
| **Sony Xperia 10 II/III/IV/V with Sailfish** | [Sailfish supported-device table](https://docs.sailfishos.org/Support/Supported_Devices/) lists precise variants. IV/V free trial images are identified in the inspected table; procurement, licences and updates need separate review. Older Xperia X is in unsupported devices, last release 4.6.0. | Genuine native Sailfish userspace with downstream BSP: II kernel 4.14, III 4.19, IV/V 5.4 in official table. [`nfcd-binder-plugin`](https://github.com/mer-hybris/nfcd-binder-plugin) talks Android NFC HAL via binder; older [`pn54x` plugin](https://github.com/monich/nfcd-pn54x-plugin) uses raw character I/O. Exact NFCC differs by phone/variant and was not established from this table. | nfcd has real ISO-DEP host objects and LocalHostApp APDU dispatch. This is the closest software integration fit, but a daemon API is not a tested claim for every Xperia model. Obtain adapter/config/firmware inventory and host SELECT trace. | Prefer an existing working Sailfish NFC device as the first **phone** qualification candidate if BSP/HAL dependencies are acceptable. |
| **Jolla C2** / Sailfish | Official table lists C2/S19 Max Pro S, first release 5.0.0 in 2025, kernel/BSP 5.4/13. | Native Sailfish with BSP, not a mainline proof. Exact NFCC/physical NFC presence not established by inspected table. | Do not infer NFC or HCE just because it runs nfcd-capable Sailfish. | Unqualified; require model hardware evidence before considering it. |
| **Google Pixel 3a / XL** / Mobian, postmarketOS | Mobian first-tier Android-origin device. Old hardware/firmware lifecycle, no renewed OEM support claim. | [sdm670-google-common.dtsi](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/arch/arm64/boot/dts/qcom/sdm670-google-common.dtsi) identifies **PN557**, 0x28, enable GPIO12, firmware GPIO43, IRQ GPIO44, fallback nxp-nci. | Same kernel NFC HCE gap as OnePlus; do not assume PN553 stack identity. | Sensible existing Linux host; not proven HCE target. |
| **Pocophone F1** / Mobian, postmarketOS | Mobian first tier. | Broad Linux support does not imply an NFC-equipped variant or controller. No qualified NFC path established here. | No HCE evidence. | Application demo host only; verify actual hardware before NFC consideration. |
| **FuriLabs FLX1 / FuriOS** | [Official shop](https://furilabs.com/shop/flx1), inspected 2026-10-28: $550, **out of stock**. No fixed support end date verified. | Native Debian-based userspace with downstream Hybris/API32 adaptation. Vendor advertises an NFC/wireless-charging combination; public adaptation pins kernel baseline 4.19.325. Exact NFCC/DT/firmware not established. | FuriLabs publishes nfcd + binder-plugin packaging and LocalHostApp (details below). Reader/host RF operation on FLX1 not verified here. | Material additional **existing-phone** qualification candidate if BSP/HAL is acceptable; don't treat software presence as tested HCE. |
| **FuriLabs FLX1s / FLX1S+ / FuriOS** | [FLX1s](https://furilabs.com/shop/flx1s): $499 sale, **out of stock**; [FLX1S+](https://furilabs.com/shop/flx1splus): $499 sale, **in stock**, on 2026-10-28. | Native Linux family with downstream adaptation; the inspected specifications for these models do **not** advertise NFC, unlike FLX1. That omission is not conclusive electrical absence. | No model-specific NFC controller, installed HAL or host APDU evidence established. Shared OS NFC packages do not prove this hardware has NFC. | Application-host options, not built-in NFC recommendations; do not inherit FLX1's hardware claim. |
| **Jolla Phone (2026), distinct from C2** / Sailfish OS 5 | [Official current batch page](https://commerce.jolla.com/products/jolla-phone-october-2026), inspected 2026-10-28, says **shipping since 2026-07-08**, minimum five years OS support, EU/UK/Norway/Switzerland sales. Main text advertises November batch while FAQ still says October; HTML also shows unavailable variants. Confirm allocation rather than claiming immediate stock. | Vendor explicitly advertises NFC and MediaTek platform. Exact kernel/BSP version, NFCC, DT and firmware not established from inspected primary sources; the supported-device table still documents C2/Xperia, not this new phone's adaptation. | Sailfish's public nfcd host interface is relevant reuse, but no new-phone-specific plugin configuration or physical HCE trace was established. AppSupport payment behavior is a separate question. | Add to current native-phone shortlist, not merely announcements; require device-specific adapter/host SELECT evidence before preferring it over a known-working Xperia. |
| **Liberux NEXX** (sometimes shortened to NEX) / proposed LiberuxOS | [Official site/FAQ](https://liberux.net), inspected 2026-10-28, says it is evaluating next steps after Indiegogo and **cannot confirm a relaunch date**. No shipping/customer-delivery evidence established. | Advertised RK3588S and Debian 13-based OS; source publication is phrased as a future commitment. Exact shipping kernel, NFC hardware, DT, firmware and public NFC adaptation not established. | No reader/HCE/userspace path established. Renderings/specifications are explicitly subject to change. | Announced/development watchlist, **not a shipping qualification candidate**. |
| **F(x)tec Pro1-X (QX1050)** / Ubuntu Touch | [UBports current page](https://devices.ubuntu-touch.io/device/pro1x/) says unsupported by current Ubuntu Touch release; [Focal page](https://devices.ubuntu-touch.io/device/pro1x/release/focal/) remains available. Vendor buy link returned 404; current new stock/support term not established. | UBports documents Halium 11, downstream kernel 4.19.157. Public adaptation README/deviceinfo confirm Android 11/Halium, not mainline. Exact NFCC/firmware unresolved. | Focal feature table explicitly marks **NFC untested**; no host APDU path established. Do not substitute original Pro1 hardware/port evidence. | Relevant existing keyboard/Linux application host, not a first NFC target. |

For Mobian tiers the direct current source is
[Mobian/Devices](https://wiki.debian.org/Mobian/Devices), read successfully via
HTTP. It explicitly distinguishes lightly patched mainline Linux from Android
devices newly supported by the community. postmarketOS wiki requests hit an
Anubis challenge, and current pmaports raw paths attempted during this research
returned 404. Consequently this report **does not invent current per-device
postmarketOS NFC status, kernel package versions or support categories**.
The historical [v24.06 release](https://postmarketos.org/blog/2024/06/16/v24.06-release/)
lists OnePlus 6/6T, FP4 and Pine devices, but is not current NFC evidence.
**Naming scope:** the project's directly inspected
[“Project rebrand: Nura” announcement](https://postmarketos.org/blog/2026/09/27/nura-rename/),
dated **2026-09-27**, explicitly renames the operating-system project formerly
called postmarketOS to Nura and says migration of user-facing strings will take
time. This does not prove every repository/package/URL has changed. This report
retains postmarketOS when naming existing ports and historical sources; resolve
current package sources directly rather than inferring URLs from either name.

### Followup source notes: additional current phones

All observations in these added rows/notes were checked **2026-10-28**, against
the same local product baseline `f6ea6a8`; no runtime validation was added.

**FuriOS is a substantive additional nfcd reuse candidate.** Public
[`adaptation-furiphone-krypton/debian/control`](https://github.com/FuriLabs/adaptation-furiphone-krypton/blob/cff8378ddd9d5d55ce86c3bbf5a16b1cf143fdd7/debian/control)
identifies the FLX1 API32 adaptation and depends on
`linux-{boot,image}-furiphone-krypton >= 4.19.325+furios5`, vendor image packages
and `adaptation-hybris-api32-phone`. Commit `cff8378…` is dated 2026-10-01;
`etc/flash-bootimage/01-kernel-ver` says `4.19.325-furiphone-krypton`.
This is a concrete downstream/BSP boundary, not an inference from Android-app
support. The separate radon adaptation also depends on API32/Hybris and vendor
packages; its copied package description still says FLX1, so it is **not** used
here to assert a precise FLX1s/+ mapping or kernel version.

FuriLabs' [`nfcd-binder-plugin/debian/control`](https://github.com/FuriLabs/nfcd-binder-plugin/blob/e0e9565d3e055f17a043a67054224f0c9c9dadc2/debian/control)
(commit `e0e9565…`, 2026-07-12) requires libncicore, libnciplugin, libgbinder and
nfcd development interfaces. Its
[`nfcd LocalHostApp XML`](https://github.com/FuriLabs/nfcd/blob/7079570c9fffc6f687095f3d179af23c950f1be9/plugins/dbus_service/org.sailfishos.nfc.LocalHostApp.xml)
(commit `7079570…`, 2026-07-12) contains the same lifecycle and
`Process(CLA,INS,P1,P2,data,Le)` response boundary as the selected bridge.
Inspected nfcd Debian copyright is BSD-3-Clause. This provides **public Debian
packaging reuse** as well as an API; it does not establish those packages are
installed/configured on each model, HAL listen routing, or Linux Tap timing.
Small next step on an already-owned NFC-equipped FLX1: inventory running adapter,
HAL and firmware, then prove the lab AID SELECT before changing wallet code.
No exact chip, firmware freedom or trusted approval claim is inferred.

**New Jolla Phone:** the official shop's shipping statement is stronger than old
preorder announcements, but it is still a vendor report, not a delivery observed
here. Its minimum-five-year promise supplies no exact security-update end date
in the inspected text. No model-specific public NFC adaptation revision was
identified in this bounded followup; the generic Sailfish code audited below
must not be presented as that missing evidence. **Liberux:** the inspected site
promises future source availability but does not supply a public NFC code path;
no bounded search result established one. These are source gaps, not claims that
no such code exists or that the project has stopped.

**Pro1-X public code:** inspected UBports
[`README.md` and `deviceinfo`](https://gitlab.com/ubports/porting/community-ports/android11/fxtec-pro1x/fxtec-pro1x/-/tree/3e119fed31ee1dbca9a3cf92b093278104df3474)
at `3e119fed…` (2026-10-19, Apache-2.0 license addition) confirm Halium 11,
`vendor/pro1x-perf_defconfig`, Android boot-image version 11 and downstream kernel
source branch `halium-11.0`. The boot-image patch-level string `2021-10-01` is
build metadata, not proof of the currently installed phone's security state.
UBports' feature table is the direct evidence for **NFC untested**. A generic
kernel source pointer does not justify guessing PN553/ST/NXP driver identity or
HCE support. No flashing/build instructions from that README were executed.

## Why mainline NFC support is not a Linux HCE application stack

### Kernel NCI, digital, sockets and secure elements

At the Linux revision above, inspect
[`net/nfc/nci/core.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/net/nfc/nci/core.c),
[`ntf.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/net/nfc/nci/ntf.c),
[`rawsock.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/net/nfc/rawsock.c),
[`digital_core.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/net/nfc/digital_core.c)
and [`include/uapi/linux/nfc.h`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/include/uapi/linux/nfc.h).

- `nci_discover_map_req` maps ISO-DEP to both poll and listen. That isolated line
  looks promising but is **not sufficient evidence of end-to-end HCE**.
- Actual discovery adds passive A/F listen configurations when
  `tm_protocols & NFC_PROTO_NFC_DEP_MASK`; `nci_start_poll` configures listen
  parameters under that same NFC-DEP condition.
- `nci_rf_intf_activated_ntf_packet`'s listen branch calls `nfc_tm_activated`
  for **NFC-DEP**, not a generic host ISO-DEP APDU server.
- The digital stack similarly adds target listening for
  `NFC_PROTO_NFC_DEP_MASK`. ISO14443 initiator support is not a corresponding
  userspace card-emulation registration API.
- AF_NFC exposes raw/LLCP protocols. The `SOCK_RAW` operations use
  `sock_no_sendmsg` and require `CAP_NET_RAW`: packet-monitor access is not raw
  controller takeover. The sequenced-packet initiator exchange API does not
  create a host ISO-DEP application simply by reversing send/receive order.
- Secure-element commands and off-host routing are separate from software HCE.

Thus the narrow conclusion is **no ready-made generic ISO-DEP host application
path through the inspected kernel NFC APIs**. This is not a claim that no Linux
program can perform HCE. Sailfish and NXP explicitly implement it in different
userspace stacks, using suitable raw I/O or HAL transports.

### Controller binding and firmware are separate concerns

The upstream [`nxp-nci/i2c.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/drivers/nfc/nxp-nci/i2c.c)
matches `nxp,nxp-nci-i2c`, requests enable/firmware GPIOs, and registers with the
NFC subsystem. It is **not** the vendor pn5xx miscdevice driver. A DT property
called `firmware-gpios` selects a hardware operating mode; it is not evidence
that a correct firmware blob was found, uploaded or redistributable.

The inspected NXP [`nxp-pn5xx/pn5xx_i2c.c`](https://github.com/NXPNFCLinux/nxp-pn5xx/blob/master/pn5xx_i2c.c)
instead contains a miscdevice, raw read/write and `PN544_SET_PWR` ioctl, with DT
compatibles `nxp,pn547` and `nxp,pn544`. Its old comment says pn548/pn547/pn544
support. Renaming a node or seeing “PN5xx” in two repositories does not prove
PN553/PN557/PN7160 compatibility. The driver source is GPL-2.0-or-later; modern
kernel API adaptation and exact board power/clock handling must be reviewed.
Never bind both owners to one I²C device. Replacing a phone's working driver is
a separate risky hardware task, not part of the software-demo milestone.

For ST, inspected [`st-nci/i2c.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/drivers/nfc/st-nci/i2c.c)
matches ST21NFCB/C forms, while
[`st-nci/core.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/drivers/nfc/st-nci/core.c)
sends through `ndlc_send`. ST21NFCA HCI support lives in another driver family.
The FP5 patch author explains that **ST21NFCD uses raw NCI, not NDLC**; changing
only a compatible string leaves the controller unusable. Its proposed DT adds
clock/supply handling; GPIO39 CLK_REQ and some power rails/SWP use remain
unmodelled or untested. `ese-present`/`uicc-present` in that proposal do not prove
working secure elements, much less HCE.

ST's [Android vendor extensions](https://github.com/STMicroelectronics/ST54-android-vendor-st21nfc)
are another stack, not a portable native-Linux `st-nci` replacement. Only its
README was inspected here; no ST54 phone adaptation or firmware was qualified.
ST25 reader frontends/RFAL are not interchangeable with ST21 smartphone NFCCs.

## Component audit: reuse, adapt or reject

### Sailfish nfcd: reuse the host-application boundary

The [README](https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/README)
explicitly supports reader/writer, peer and card emulation **and requires an
adapter plugin**. The source is BSD-3-Clause; GitHub's API returned
`NOASSERTION`, showing why a badge/API license field is not a substitute for the
actual license and source headers.

Concrete source path:

`raw character device or Android HAL → hardware plugin → libncicore →
libnciplugin NciAdapter/NciInitiator → nfcd NfcHost → LocalHostApp D-Bus →
Linux Tap receiver`

- [`Daemon.xml`](https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/plugins/dbus_service/org.sailfishos.nfc.Daemon.xml)
  introduces `RegisterLocalHostApp(path,name,aid,flags)` at interface version 4,
  since daemon 1.2.0. Card emulation is mode bit `0x08`; `RequestMode` returns
  a request handle which must be released. Mode selection is not signing consent.
- [`LocalHostApp.xml`](https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/plugins/dbus_service/org.sailfishos.nfc.LocalHostApp.xml)
  gives lifecycle `Start/Restart/Stop`, `Select/ImplicitSelect/Deselect`, then
  `Process(host,CLA,INS,P1,P2,data,Le) → response,SW1,SW2,response_id`.
  A nonzero response ID requests `ResponseStatus`; this reports response
  transmission status, **not receipt settlement or wallet approval**.
- [`libncicore README`](https://github.com/mer-hybris/libncicore/blob/master/README)
  implements NCI 1.0 states and a limited NCI 2.0 subset; the caller implements
  `nci_hal.h`. It does not access hardware itself.
- [`libnciplugin/src/nci_adapter.c`](https://github.com/mer-hybris/libnciplugin/blob/master/src/nci_adapter.c)
  creates `NfcHost` on ISO-DEP activation and sets CE + listen when requested.
  Its explicit CE reactivation state machine has a 1500 ms reactivation timer,
  preserving a host across compatible reactivation. This is useful reuse, not
  behavior Linux Tap should reimplement.
- [`nci_initiator.c`](https://github.com/mer-hybris/libnciplugin/blob/master/src/nci_initiator.c)
  handles listen A/B and ISO-DEP, subscribes to RF data connection packets, and
  sends responses via `nci_core_send_data_msg(NCI_STATIC_RF_CONN_ID, …)`.
  This is concrete host-APDU source evidence, unlike a protocol bitmap.
- [`nfcd-binder-plugin`](https://github.com/mer-hybris/nfcd-binder-plugin/blob/ee270922a39d3730226936d7e78896dd508400be/README)
  talks Android 8+ NFC HALs over binder. Its tree has HIDL and AIDL
  implementations. Its RPM spec requires `libncicore`, `libnciplugin`,
  `libgbinder` and nfcd plugin headers. It does not require an Android wallet app,
  but does require a working Android-derived HAL/service adaptation.
- [`nfcd-pn54x-plugin README`](https://github.com/monich/nfcd-pn54x-plugin/blob/2d95c87fdb03776ef638d3dc37cb996c9cbeb82a/README)
  defaults to `/dev/pn54x`, configurable to `/dev/pn547`; warns about
  32-bit userspace versus 64-bit kernel `compat_ioctl`. This is not an AF_NFC
  backend and cannot simply open the OnePlus mainline nxp-nci adapter.

The [nfcd RPM spec](https://github.com/sailfishos/nfcd/blob/6fc7aa906df5b6f2c508aab1832c51827095586e/rpm/nfcd.spec)
also needs GLib, libglibutil, libnfcdef, file-devel and normally libdbusaccess;
it packages system D-Bus policy and systemd service ownership. On Arch/Debian/
Alpine this is a small ecosystem to package, not one standalone C file.
The permissive BSD licenses of the inspected Sailfish core/plugin headers are
compatible with preserving Linux Tap's Apache-2.0 code; retain notices and
review all packaged transitive components.

**Local fit:** `adapters/nfcd/bridge.py` already registers the lab AID with flags
zero and requests `0x08`. It resets/selects the Rust subprocess on lifecycle
callbacks and returns `response_id=0`. It reconstructs only the project's short
APDU profile, rejecting data over 255 and Le over 256. It is not a general
extended-APDU bridge. Its synchronous subprocess round trip inside D-Bus
callbacks is a timing/failure boundary to measure later. Do not wait for human
approval or network calls in `Process`; persist unverified invoice transport,
respond promptly, and approve afterward. No bridge change was made here.

**Cost:** low for application glue on an already-working nfcd phone; medium/high
for packaging the stack elsewhere; high if a new hardware I/O plugin is needed.

### NXP PN7160 / PN7150: reuse vendor HCE, not a guessed daemon adapter

The [PN7160 branch README](https://github.com/NXPNFCLinux/linux_libnfc-nci/blob/d5af4ef88244b9f962986b39eb4126a8c1c0aead/README.md)
explicitly says `NCI2.0_PN7160`; PN7150/PN7120 use master. Release R1.1 notes
include T5T/MIFARE fixes and old 64-bit-install caveats. Pinning is essential.

[`linux_nfc_api.h`](https://github.com/NXPNFCLinux/linux_libnfc-nci/blob/d5af4ef88244b9f962986b39eb4126a8c1c0aead/src/include/linux_nfc_api.h)
exposes actual HCE APIs: `nfcHce_registerHceCallback`,
`nfcHce_deregisterHceCallback`, `nfcHce_sendCommand`, and discovery controls with
host-routing enablement. [`libnfc-nci.conf`](https://github.com/NXPNFCLinux/linux_libnfc-nci/blob/d5af4ef88244b9f962986b39eb4126a8c1c0aead/conf/libnfc-nci.conf)
defines `HOST_LISTEN_TECH_MASK`: A=1, B=2, F=4; default 7. Neither that default
nor function availability proves correct board RF settings, permissions or
reader interoperability. A thin transport subprocess can forward callbacks to
the Rust APDU engine without linking chain/signing code into the vendor stack.

[PN7160 vendor page](https://www.nxp.com/products/PN7160), inspected 2026-10-28:
active product, integrated firmware, I²C/SPI variants, NCI 2.0 and Type 4 A/B /
Type 3 card emulation. It explicitly says **not intended for EMVCo compliance**.
The page lists firmware release notes v12.50.11 dated 2026-01-07, Linux guide
[AN13287](https://www.nxp.com/docs/en/application-note/AN13287.pdf) rev 1.1 dated
2021-09-13 and [card-emulation note AN13861](https://www.nxp.com/docs/en/application-note/AN13861.pdf)
dated 2023-04-26. These document titles/dates were observed on the product page;
the PDFs were not parsed in this research. Integrated firmware remains a vendor
dependency, not source freedom or proof of installed firmware version.

[PN7150 page](https://www.nxp.com/products/PN7150) now says **Not Recommended for
New Designs**, recommends PN7160 and says Android maintenance stopped at Android
11. It still explicitly advertises host Type 4 A/B emulation at 106 kbit/s and
NCI 1.0. An existing PN7150 is a reasonable lab reuse target; new design choice
is different. NXP's P2P maintenance retirement statement is another reason not
to base new checkout transport on LLCP/SNEP.

License: top-level NXP library is Apache-2.0 as reported by repository metadata;
the project lock correctly warns of mixed per-file licenses. Kernel driver is
GPL; firmware/configuration distribution needs its own review. Do not label the
entire firmware/host/kernel bundle Apache. **Cost:** medium for an exposed
development board; high on an undocumented phone. Start with vendor HCE tools,
then add one adapter. There is no inspected ready-made PN7160→nfcd plugin here.

### neard: retain for kernel reader diagnostics, reject as drop-in HCE daemon

[neard](https://github.com/linux-nfc/neard) is a Linux kernel NFC socket/netlink
daemon, with D-Bus, GLib and libnl. Current README uses Meson ≥1.1, while distro
0.20 recipes still use autotools. Current HEAD dated 2026-10-26 contradicts
blanket claims that it is dead. Debian's inspected package description promises
tag read/write and LLCP target/initiator with SNEP/NPP/handover, not a
LocalHostApp equivalent. Reuse `nfctool` and its reader workflows to establish
controller bring-up; don't port the wallet to LLCP to fit the tool.

License is primarily GPL-2.0-only with file exceptions. D-Bus/process use is a
cleaner integration boundary than copying its internals into Apache wallet
code. **Cost:** low for diagnostics; high/speculative to add generic HCE through
the kernel APIs described above. Keep nfcd and neard names distinct.

### pcsc-lite, CCID and pcsc-rust: reuse on merchant side

[`pcsc-rust/pcsc/src/lib.rs`](https://github.com/bluetech/pcsc-rust/blob/master/pcsc/src/lib.rs)
wraps SCard APIs for reader enumeration, connect, transactions, transmit and
status changes. The crate documents that operations on one context serialize;
use separate contexts for monitoring and APDU traffic, and avoid blocking the
wallet UI. Its MIT source is already pinned by this project. This is a reader
API, not a way to turn a phone's NFCC into a card.

[pcsc-lite COPYING](https://github.com/LudovicRousseau/PCSC/blob/master/COPYING)
is primarily BSD, with BSD-2/ISC files and GPLv3+ spy/test exceptions. A broad
“BSD only” package statement is inaccurate. [CCID](https://github.com/LudovicRousseau/CCID)
is LGPL-2.1-or-later and maps USB reader hardware to pcscd; its repository API
reported 2026-10-25 activity, as did PCSC. A reader may require `acsccid` or a
vendor driver instead. PC/SC APDU access does not imply target/card-emulation
support, even when the controller inside the reader has that feature.

**Cost:** low, existing project feature. The eventual ACR1552U candidate must be
qualified by exact USB ID, firmware, actual driver and ISO-DEP APDU behavior;
this report does not certify that reader or spend on one.

### vsmartcard/vpcd: recommended emulated PC/SC boundary

Inspected [`ifd-vpcd`](https://github.com/frankmorgner/vsmartcard/tree/93286a90d2411120ae17299f9902e1fdd8954185/virtualsmartcard/src/ifd-vpcd)
is a pcsc-lite reader driver. [`vpcd.c`](https://github.com/frankmorgner/vsmartcard/blob/93286a90d2411120ae17299f9902e1fdd8954185/virtualsmartcard/src/vpcd/vpcd.c)
uses a two-byte **network/big-endian** length and socket exchange. Its listen
server sets `sin_addr.s_addr = htonl(INADDR_ANY)`. This is an actionable exposure
hazard: isolate the lab service in a private network namespace/VM with no
external interface, or review a minimal loopback-bind configuration/patch before
running it. “VICC connects to localhost” does not constrain the server bind.

[`VirtualSmartcard.py`](https://github.com/frankmorgner/vsmartcard/blob/93286a90d2411120ae17299f9902e1fdd8954185/virtualsmartcard/src/vpicc/virtualsmartcard/VirtualSmartcard.py)
separates control messages of length one: OFF=0, ON=1, RESET=2, ATR=4. Longer
messages are command APDUs; a SmartcardOS `execute` produces the response.
Default PC/SC slots commonly use TCP 35963/35964. A minimal virtual ICC must
implement valid ATR/power/reset behavior and reset the APDU transport state on
card reset; it must not discard the wallet's durable approved transaction.
The inspected Python helper uses `recv` for header/payload, so a new independent
transport adapter should implement exact-length reads rather than assuming one
TCP read equals one complete message.

Use vpcd as an external lab component and implement only its documented wire
boundary around the existing Rust APDU receiver. Do not fork its passport,
identity-card filesystem, SAM or cryptographic implementations. Debian copyright
and source headers show GPL-3.0-or-later for the vpcd transport and virtual ICC
Python files, with GPL-2+ and other files elsewhere in the umbrella repository.
Copying that implementation into Apache-only code is not the same as using the
unmodified driver as a separate process/component; preserve the correct source
and license distribution obligations.

**Cost:** low/medium: one narrow adapter plus isolated pcscd/vpcd configuration.
No RF timing, NCI, phone firmware or nfcd compatibility is established by a PASS.

### libnfc and nfcpy: useful optional bench tools, not universal phone stacks

[libnfc](https://github.com/nfc-tools/libnfc) is oriented around supported reader
drivers such as PN53x USB/UART and PC/SC; it is not NXP's similarly named
`linux_libnfc-nci`. Its library is LGPL-3.0-or-later; examples/utilities have their
own headers. Inspected [`utils/nfc-emulate-forum-tag4.c`](https://github.com/nfc-tools/libnfc/blob/3fa0751ad58fb0053d3de2e61bbbd4066259104c/utils/nfc-emulate-forum-tag4.c)
explicitly says it works only with PN532 because it relies on firmware handling
ISO14443-4 and WTX. It implements NDEF SELECT/READ/UPDATE, not Linux Tap's lab
application. Reuse the transport API if qualifying an existing PN532, not its
NDEF application parser as wallet policy.

[`nfcpy/src/nfc/clf/pn532.py`](https://github.com/nfcpy/nfcpy/blob/8a195262fe86167845d282ba5bc9cdaadf6441c4/src/nfc/clf/pn532.py)
has unusually concrete limits: Type A listen at 106 kbit/s; a four-byte UID
starting `08`; optional replacement RATS response; Type B listen unsupported;
Type F maximum frame size 64 bytes; host command frame maximum 265. It exposes
`tt4_cmd` after suitable Type A activation. These constraints make it credible
as a narrow laboratory tool, not arbitrary-card cloning or full NFC coverage.
Actual USB bridge firmware and antenna still matter. A low-price PN532 module
advertised as “NFC” need not expose its target mode through that board's USB
firmware.

nfcpy's [license](https://github.com/nfcpy/nfcpy/blob/8a195262fe86167845d282ba5bc9cdaadf6441c4/LICENSE)
is **EUPL 1.1**, not MIT. Keep it optional and outside chain/signing code. Its
default branch was still the 2022 1.0.4 release in inspected API evidence.
**Cost:** low for exploratory Python/PN532 experiments; medium for robust custom
card transport; reject as a shortcut to phone PN553/ST21 driver integration.

[libllcp](https://github.com/nfc-tools/libllcp) is an LGPLv3-era LLCP implementation
for libnfc; API reported its last repository push 2015-03-14. Even without that
maintenance concern, LLCP/NFC-DEP is not the required ISO-DEP reader/card APDU
transport. **Reject as a new dependency for this milestone.**

## Emulation layers: choose the boundary being proved

| Layer | What it can establish | What it cannot establish / cost |
| --- | --- | --- |
| Existing Rust APDU receiver over pipes/private socket | Actual command parsing, bounded reconstruction and wallet handoff when wired together | No pcscd, reader, NCI, NFCC or RF; lowest cost |
| pcsc-lite + vpcd + thin virtual ICC | Actual merchant PC/SC enumeration/connect/transmit boundary with real receiver and wallet | No NFC RF or phone driver; low/medium cost; isolate TCP exposure |
| nfcd host application against a qualified simulator/adapter | D-Bus registration, AID dispatch and daemon lifecycle | Requires actual adapter; LocalHostApp unit/pipe checks alone do not prove live D-Bus/NCI |
| Linux `virtual_nci` + controller peer | Kernel NCI initialization, notifications, polling and reader exchange | It is not a complete simulated NFCC or ready paired HCE service; medium/high work |
| Linux `nfcsim` | Paired digital NFC-DEP/LLCP paths and fault injection | Not a ready ISO-DEP card emulator; wrong default protocol for this project |
| Casimir standalone or Cuttlefish | Simulated NCI and NFC-A ISO-DEP interactions, HCE APDU injection in Cuttlefish | Not physical RF, not automatically an nfcd plugin, build/routing limits need review |
| QEMU guest + CCID device | Guest PC/SC/USB reader boundary, OS/UI packaging | Not a phone NFCC model or RF timing proof; unnecessary for first application demo |

### Linux nfcsim and virtual_nci source details

[`drivers/nfc/nfcsim.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/drivers/nfc/nfcsim.c)
creates two linked digital devices with **only `NFC_PROTO_NFC_DEP_MASK`** at
allocation. Its presence does not mean a ready ISO-DEP APDU card pair. Don't
build a custom ISO-DEP emulator into the kernel merely to demonstrate the wallet.

[`drivers/nfc/virtual_ncidev.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/drivers/nfc/virtual_ncidev.c)
creates `/dev/virtual_nci`, mode 0600. Opening it allocates/registers an NCI
device. User reads receive kernel-generated NCI frames; writes feed
`nci_recv_frame`; ioctl command 0 returns the device index. Only one outbound
send buffer is pending at a time. User code must implement the controller side
of reset/init/config/discovery/activation/credits/data, not send invoice APDUs
directly to this fd. Its protocol mask includes reader tag types, not a claim of
HCE support.

[`tools/testing/selftests/nci/nci_dev.c`](https://github.com/torvalds/linux/blob/65ba8f34ec886ec72b2c07ddc8cf3d815c32869d/tools/testing/selftests/nci/nci_dev.c)
shows real reuse material: opens the fd, obtains index, responds in helper
threads to NCI 1/2 startup, controls the device over netlink, and tests
`t4t_tag_read`. This is stronger than a link to a simulator name, but its T4T test
is **kernel reader against simulated tag**, not phone HCE. Both kernel components
carry GPL source obligations; keep this separate from the application demo.

### Casimir: promising native OSS, with important qualifications

Primary sources inspected:
[AOSP Cuttlefish NFC](https://source.android.com/docs/devices/cuttlefish/nfc),
[`README.rst`](https://android.googlesource.com/platform/packages/modules/Nfc/+/refs/heads/main/libnfc-nci/tools/casimir/README.rst),
[`Cargo.toml`](https://android.googlesource.com/platform/packages/modules/Nfc/+/refs/heads/main/libnfc-nci/tools/casimir/Cargo.toml),
[`src/main.rs`](https://android.googlesource.com/platform/packages/modules/Nfc/+/refs/heads/main/libnfc-nci/tools/casimir/src/main.rs),
and [`casimir_control.proto`](https://android.googlesource.com/device/google/cuttlefish/+/refs/heads/main/host/commands/casimir_control_server/casimir_control.proto).
Gitiles reported directory tree ID `500f3bca4d59ffceefe1e2bbf6d735c92ebe4faf`.

- Native Rust/Tokio server, NCI and simulated RF endpoints, NFC-A,
  ISO-DEP/NFC-DEP. README supplies a Type 4-A Python peer, `scripts/t4at.py`, in
  poll or listen mode. This can be reused without accepting Android as the wallet.
- `main.rs` defaults TCP listeners to **loopback**, NCI port 7000 and RF port
  7001, with inherited Unix listener fd options. Cuttlefish instance defaults
  differ (README: 7100/8100 + instance offset). Record which execution mode is
  actually used rather than copying mismatched defaults.
- Simulated RF packets have a **little-endian** u16 length, unlike vpcd's
  big-endian protocol. Main reads use `read_exact`; it broadcasts synthetic RF
  packets and sends link-loss notifications when peers disconnect. These are
  software events, not electromagnetic conformance evidence.
- Source is Apache-2.0 in inspected Rust/proto headers. README marks listen-mode
  routing commands as stubs/in progress, with several configuration features
  saved but unused. Do not claim eSE policy or full routing validation.
- Current `main.rs` imports `rustutils::inherited_fd`; inspected Cargo.toml does
  not declare rustutils. Its manifest also brings PDL generation and gRPC/protobuf
  build dependencies. Consequently **“standalone supported” is upstream intent,
  not a claim that today's tree builds with one Cargo command**. AOSP build
  metadata and a narrow external build adaptation need inspection before a spike.
- Cuttlefish documentation, updated **2026-09-22**, explicitly supports Android
  15+ HCE injection through `CasimirControlService.SendApdu`, NFC-A/ISO-DEP. The
  inspected proto additionally exposes radio state, power level and broadcast
  controls. Useful independent Android HCE comparison later; no such runtime was
  started here.

In contrast, [Android Studio advanced emulator documentation](https://developer.android.com/studio/run/advanced-emulator-usage),
updated **2026-03-06**, still lists NFC as absent virtual hardware. These are
different products/documented paths, not a contradiction to hide with an
absolute “emulator can't NFC” statement. An Android-specific HCE service remains
an Android test peer, not the required native Linux wallet.

### QEMU and optional RF instrumentation

[QEMU's official CCID documentation](https://www.qemu.org/docs/master/system/devices/ccid.html)
supports a completely virtual reader/card and libcacard-backed emulation or
passthrough. That is a USB smartcard abstraction; it doesn't emulate PN553 I²C,
ST21 firmware, phone suspend or RF. QEMU is useful for an isolated Linux guest
or cross-architecture UI later, but no custom phone model is needed now.

Proxmark3 and Chameleon are **optional measurement tools**, not wallet engines.
[Proxmark3 command source documentation](https://github.com/RfidResearchGroup/proxmark3/blob/master/doc/commands.md)
lists ISO14443-A sniff, APDU, chaining and AID simulation commands. That is useful
for later lab traces, not proof that arbitrary PC-host APDU emulation meets
timing. [ChameleonUltra upstream](https://github.com/RfidResearchGroup/ChameleonUltra)
is a separate firmware/card-emulation project; do not equate MIFARE/UID emulation
with ISO-DEP arbitrary host APDUs. These tools were only scoped, not deeply
qualified, compiled or used here. No recommendation to purchase, flash, clone
credentials or interact with someone else's cards is made.

## Distribution packages: exact names, sources and limits

These are observed metadata/source recipes, **not commands executed** and not a
complete reproducible deployment manifest. Desktop Arch package availability
does not imply Arch Linux ARM builds for a particular phone. Debian pool contents
span suites; a version in the pool is not automatically the installed stable
version. postmarketOS inherits Alpine packages but uses its own kernel/device
ports and overlays.

| Distribution | Verified source/metadata | Integration consequence |
| --- | --- | --- |
| Arch / Omarchy merchant | [`pcsclite` 2.5.3-1](https://archlinux.org/packages/extra/x86_64/pcsclite/), built 2026-10-18; [`ccid` 1.8.4-1](https://archlinux.org/packages/extra/x86_64/ccid/), updated 2026-09-21; [`libnfc` 1.8.0-4](https://archlinux.org/packages/extra/x86_64/libnfc/), built 2026-10-26 | Existing PC/SC feature needs pcsclite and pkgconf for discovery, plus actual reader driver for hardware. libnfc is optional, not a required wrapper around PC/SC. Package source links lead to Arch packaging GitLab. |
| Debian / Mobian | Inspected official [vsmartcard pool](https://deb.debian.org/debian/pool/main/v/vsmartcard/), including `vsmartcard_3.3+dfsg-2.debian.tar.xz`, control and copyright read in memory | Packages `vsmartcard-vpcd`, `vsmartcard-vpicc`, `python3-virtualsmartcard`; vpcd depends on `pcscd`. Build requirements include libpcsclite-dev/libnfc-dev, autotools, gengetopt and Python. Use upstream current revision or Debian 3.3 deliberately; don't mix APIs silently. |
| Debian / Mobian kernel diagnostics | Official [`neard_0.20-3.debian.tar.xz`](https://deb.debian.org/debian/pool/main/n/neard/neard_0.20-3.debian.tar.xz), control/copyright inspected in memory | `neard`, `neard-dev`, `neard-tools`; tools include nfctool/nciattach. Daemon relies on kernel NFC socket/netlink, not Sailfish API. Source package still autotools while current upstream is Meson. |
| Debian PC/SC | Official [pcsc-lite pool](https://deb.debian.org/debian/pool/main/p/pcsc-lite/) and [ccid pool](https://deb.debian.org/debian/pool/main/c/ccid/) | Source versions observed include pcsc-lite 2.3.3/2.5.2/2.5.3 and ccid 1.6.2/1.8.4. Typical binary names are `pcscd`, `libpcsclite-dev`, `libccid`; qualify exact suite/architecture before installation. No suite-specific resolution run here. |
| Alpine / postmarketOS PC/SC | [`main/pcsc-lite/APKBUILD`](https://github.com/alpinelinux/aports/blob/master/main/pcsc-lite/APKBUILD): 2.5.2-r0; [`community/ccid/APKBUILD`](https://github.com/alpinelinux/aports/blob/master/community/ccid/APKBUILD): 1.8.4-r0 | `pcsc-lite`, `pcsc-lite-libs`, `pcsc-lite-dev`, `pcsc-lite-openrc` / `-systemd`; recipe disables polkit/libsystemd in core build. Do not assume Arch service and access-control instructions apply. |
| Alpine / postmarketOS diagnostics | [`community/neard/APKBUILD`](https://github.com/alpinelinux/aports/blob/master/community/neard/APKBUILD): 0.20-r0; [edge aarch64 package](https://pkgs.alpinelinux.org/package/edge/community/aarch64/neard), built 2026-09-03 at aports `9c1274da27ccbe74bcc958ee203eb845cd03a7a0`; [`community/libnfc/APKBUILD`](https://github.com/alpinelinux/aports/blob/master/community/libnfc/APKBUILD): 1.8.0-r2 | neard recipe depends on dbus/glib/libnl3 and supplies OpenRC/systemd subpackages. libnfc splits dev/tools/doc. None creates a kernel HCE API. |
| Sailfish nfcd ecosystem on other distros | Upstream RPM specs and source inspected; attempted pmaports `main/nfcd/APKBUILD` URLs returned 404 | No claim that Arch/Debian/Alpine officially ships this entire stack. Package nfcd + support libraries + I/O plugin + narrow D-Bus policy only after choosing an actual backend. AUR or third-party builds, if used later, need separate review. |

Exact licencing remains per artifact: Arch pcsclite metadata lists
0BSD/BSD-2/BSD-3/GPLv3+; Alpine separates BSD/ISC core from GPL spy files. These
differences are packaging scope, not grounds to overwrite upstream notices.
Keep lab GPL/EUPL tools external unless intentionally accepting their derivative
work obligations. No license badge is evidence of redistributable NFCC firmware.

## Recommended next increments, with acceptance evidence

### A. Finish the application milestone without adding hardware dependencies

Use existing private transport first. Send a **fresh valid laboratory invoice**
through SELECT `F04C54415000`, OPEN, PUT chunks ≤220 bytes and COMMIT. The checked-in
fixture is expired, so it is transport input, not an approvable invoice.
The receiver's persisted output must become the wallet's actual input.
Preserve merchant key pinning, chain/asset/template verification and explicit
typed approval; the transport process never receives keys.

One realistic acceptance scenario is enough:

1. Deliver the invoice via actual APDU receiver, then show verified exact terms.
2. Approve once, persist the exact signed transaction before sending a claim.
3. Simulate the merchant recording the claim **then suppress its reply**. This
   is an unknown-ack outcome, distinct from the existing demo's controlled exit
   before send. Restart both processes from the same private journals.
4. Resume/replay the existing bytes; compare exact stored bytes and digest, and
   show the merchant's unique simulated receipt remains the same single row.
5. Keep `chain_submission: false` and simulated receipt labels visible. A lost
   APDU transport ACK is not the same event as an uncertain payment/merchant ACK;
   record which boundary was interrupted.

This is a proposed next check, not a check performed by this research. It should
exercise real process/journal behavior rather than mock internal functions.
Human approval must not be held open inside the ISO-DEP response deadline.

### B. Exercise the existing merchant PC/SC code in software

After A, use isolated pcscd/vpcd and a transport-only virtual ICC. Keep the
merchant's PC/SC path unchanged. Translate VICC frames and reset controls to the
Rust receiver, supply a valid test ATR, bound message sizes and use exact socket
reads. Run the same invoice-to-wallet acceptance scenario. Record:

- package/revision and config, reader name, selected protocol and APDU transcript;
- that this is vpcd virtual PC/SC, not NFC/RF or CCID hardware;
- exact signed-byte equality and single simulated receipt after lost ACK/restart;
- proof the vpcd endpoint is not exposed outside its private lab environment.

Do not install a global PC/SC driver or start a host service merely to learn
whether the application's retry semantics work. A private disposable Linux guest
is an option if packaging otherwise requires host-wide changes.

### C. Qualify one RF path only after the software milestone

Order candidates by demonstrated integration fit, not marketing:

1. **Existing PN7160 eval board + Linux host:** source-pinned NXP HCE path,
   matching electrical interface, vendor baseline first. Blockers: actual board,
   modern kernel transport build, firmware/RF config and trace. Add only a narrow
   vendor-callback→Rust APDU adapter if baseline passes.
2. **Existing Sailfish Xperia with functioning NFC:** native LocalHostApp already
   fits the project. Blockers: exact variant/controller/firmware and installed
   binder/direct plugin; arbitrary host SELECT must work. BSP/HAL/proprietary
   dependencies must be accepted explicitly, not hidden behind “native.”
3. **Existing PN532 module with real target-mode access:** inexpensive reuse if
   already available. Blockers: module firmware/USB path and Type A/UID/MTU/timing
   limits. Not a production phone solution.
4. **OnePlus 6/6T or SHIFT6mq native port:** strong application platform and known
   DT hardware; higher NFC adaptation cost. Stop before driver replacement and
   choose a separate bounded kernel/raw-I/O integration task.
5. **FP5/ST21NFCD:** upstream reader patch/NDLC work is real and useful, but HCE
   explicitly untested. Contribute to that boundary only if it is the desired
   hardware engineering project, not as a prerequisite for the checkout demo.

For each physical candidate record exact model/region/board, NFCC identity,
kernel source/config/DT, bound driver, firmware version and provenance,
power/IRQ/clock configuration, userspace package revisions, device permissions,
reader USB identity/firmware/driver, SELECT and full APDU transcript, ATS/negotiated
limits, response latency, field removal mid-transfer, reselect and screen/lock/
suspend behavior. Receipt and signer approval claims remain separate. No shopping,
flashing, firmware update or live payment is authorized by this recommendation.

### D. Optional NCI depth, not a prerequisite

If emulator-first work must include a full native NCI host stack, inspect/pin
Casimir's AOSP build dependencies and add one small `NciHalIo` transport for
nfcd's existing NCI libraries. Prove reset/init, CE discovery, ISO-DEP activation,
lab AID APDUs and simulated link loss before claiming that bridge exists.
Alternatively reuse virtual_nci selftests for kernel **reader** work. Do not
conflate the two directions or implement both in parallel without a demonstrated
need. Neither is necessary to prove wallet approval/journal idempotency.

## Scope and remaining uncertainty

This report inspected repository instructions/docs/lock, the local nfcd bridge
and APDU specification; live kernel/source files, GitHub commit metadata,
Sailfish XML and adapter implementation, NXP API/config/driver, vsmartcard C and
Python wire/control code, libnfc/nfcpy source, AOSP Casimir Rust/proto/docs,
QEMU docs, current vendor/support pages, distro package pages/recipes, and Debian
source-package control/copyright files read from tarballs **in memory**.

HTTP sometimes returned 429; retrying selected source requests succeeded.
GitHub unauthenticated API subsequently rate-limited with 403, so no API dates
are invented for unqueried auxiliary repositories. postmarketOS wiki was
challenge-blocked; some pmaports and guessed documentation paths returned 404;
ST product page timed out. Those gaps remain gaps, not negative device tests.
Phone firmware binaries, complete downstream Xperia/Volla DTs and current
per-phone installed images were not inspected. Optional Proxmark/Chameleon
coverage is intentionally shallow and not a hardware endorsement.

**Validation performed by this worker: source/metadata inspection only. No
compilation, wallet demo, pcscd, D-Bus daemon, emulator, kernel module or physical
hardware runtime was run.** Existing project compilation and pipe/socket results
remain the claims of their recorded validation documents; this research neither
reruns them nor upgrades them to RF, trusted approval, live payment, EMV or
commercial-terminal acceptance proof.
