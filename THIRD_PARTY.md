# Third party provenance

New project code is Apache-2.0. Dependencies, copied interface definitions and
agent guidance retain their own licenses. This file is a provenance record,
not a legal opinion or claim that third-party projects have audited this work.

## Linked dependencies

Cargo.lock records exact registry versions and checksums and an immutable Mysten
SDK revision. deps/cargo-licenses.json records the license metadata of the full
resolved Cargo graph, including optional and platform-specific packages. Metadata
is not a substitute for reviewing the relevant license files before distribution.

The core integration uses sui-sdk-types, sui-crypto and sui-rpc under Apache-2.0;
minicbor under BlueOak-1.0.0; blake2 and Tokio under their declared permissive
licenses; rusqlite with bundled SQLite; and clap, serde and related dependencies
under the alternatives recorded in the inventory. The optional pcsc crate is MIT.
The B14 SQLite fix selects rusqlite 0.39.0 / libsqlite3-sys 0.37.0 (MIT),
bundling upstream SQLite 3.51.3 (public domain). The inventory records the
changed dependency closure, including rusqlite's non-Linux WASM dependencies;
those are not a new Linux runtime or wallet engine. Native SQL identity and
fixture checkpoint/reopen qualification passed; [scoped evidence](docs/evidence/sqlite-wal-qualification-2026-10-29.json)
is not wire-loss, power-cut or production durability proof.
System PC/SC, reader drivers, Qt and future hardware dependencies have their own
distribution requirements and are not included as binaries.

## Copied unmodified source

- `upstream/interfaces` contains LocalHostApp and Daemon XML from sailfishos/nfcd
  at 6fc7aa906df5b6f2c508aab1832c51827095586e. Its BSD-3-Clause license and
  copyright notices are retained in nfcd-LICENSE. The bridge implements this
  existing interface; it does not include the daemon binary or hardware plugins.

## Reviewed source stepping stones

Official [Mysten guidance](https://github.com/MystenLabs/skills) at revision
e31b38016723e2dcee36161d59c50e028e13b52a is an external Apache-2.0 reference,
not distributed guidance or a build dependency. `deps/mysten-skills.sha256.json`
retains the reviewed 25-file upstream checksum inventory, not a claim that those
files or their license are bundled here. Consult the upstream license for reuse.

deps/upstreams.lock.json records exact reviewed revisions for the SDK, official
guidance, nfcd, the NXP PN7160 branch, pcsc-rust and CXX-Qt. The fetch script only
downloads those revisions. It does not imply all of those packages are linked,
installed, license-qualified for every use or tested with our hardware.

The Gem Wallet, Electrum, Monero GUI, Trust Wallet Core, Ledger Sui application
and BTCPay repositories were evaluated as possible foundations or references.
Their code is not copied into this bootstrap. See docs/reuse.md for the decision.
Clarification, 28 October 2026: the inspected current Gem monorepo's
`core/LICENSE` is MIT, separately from its top-level application GPL terms.
Neither applies indiscriminately to every file. See the pinned license/source
evidence in the [payment implementation map](docs/research/payment-implementation-map.md).
This is a scope correction, not adoption or clearance of dependencies/assets.

No firmware blobs, commercial SDKs or third-party wallet branding are distributed.

The opt-in [virtual PC/SC lab](docs/pcsc-lab.md#sources-checksums-and-licenses)
pins official pcsc-lite 2.3.3, vsmartcard `93286a90…` and Meson 1.7.0-1 downloads
with SHA-256 checks. Only `scripts/pcsc-lab-vpcd.patch` is retained here as a
GPL-3.0-or-later modification to vpcd; new orchestration code remains Apache-2.0.
Complete sources, upstream notices/licenses and locally built binaries stay in
the private external cache, not the checkout. No package/system service install.

Open hardware interfaces do not establish that a secure element's firmware or
vendor provisioning service is open. No project endorsement is implied.
