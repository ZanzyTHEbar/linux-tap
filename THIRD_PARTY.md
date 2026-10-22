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
System PC/SC, reader drivers, Qt and future hardware dependencies have their own
distribution requirements and are not included as binaries.

## Copied unmodified source

- `.agents/skills` contains five selected skill directories and reference files
  from MystenLabs/skills at e31b38016723e2dcee36161d59c50e028e13b52a.
  Its Apache-2.0 license is retained at `.agents/skills/LICENSE`.
  deps/mysten-skills.sha256.json records checksums of all 25 included files.
  Evaluation datasets are omitted; included guidance files are unchanged.
- `upstream/interfaces` contains LocalHostApp and Daemon XML from sailfishos/nfcd
  at 6fc7aa906df5b6f2c508aab1832c51827095586e. Its BSD-3-Clause license and
  copyright notices are retained in nfcd-LICENSE. The bridge implements this
  existing interface; it does not include the daemon binary or hardware plugins.

## Reviewed source stepping stones

deps/upstreams.lock.json records exact reviewed revisions for the SDK, official
guidance, nfcd, the NXP PN7160 branch, pcsc-rust and CXX-Qt. The fetch script only
downloads those revisions. It does not imply all of those packages are linked,
installed, license-qualified for every use or tested with our hardware.

The Gem Wallet, Electrum, Monero GUI, Trust Wallet Core, Ledger Sui application
and BTCPay repositories were evaluated as possible foundations or references.
Their code is not copied into this bootstrap. See docs/reuse.md for the decision.
The current Gem monorepo's GPL-3.0 terms must not be replaced with the historical
standalone core's MIT description when considering a future fork.

No firmware blobs, commercial SDKs or third-party wallet branding are distributed.
Open hardware interfaces do not establish that a secure element's firmware or
vendor provisioning service is open. No project endorsement is implied.
