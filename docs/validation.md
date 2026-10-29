# Validation record

## Execution-response-loss candidate: offline native guards validated

29 October 2026: contract settled by coordinator engineering. Preserve the pinned
SDK's valid in-helper recovery; do not require Unknown when checkpoint proof is
available. The [separate operator candidate](execution-response-loss-lab.md) is
compiled and **local offline native guards PASS; independent source/evidence review complete**.
Live execution-response loss remains unqualified. The paid forwarding branch was
never exercised and is never part of agent validation or default checks.

Pinned source at `1313d5112464ecc25d7407f42442adea08589fc5`,
[`client/transaction_execution.rs`](https://github.com/MystenLabs/sui-rust-sdk/blob/1313d5112464ecc25d7407f42442adea08589fc5/crates/sui-rpc/src/client/transaction_execution.rs#L233-L268):
- Lines 140–148 subscribe to checkpoints before Execute; a future proxy must
  forward that real stream with bounds/backpressure, not fabricate checkpoints.
- Lines 233–243 can return VerifiedSuccess from the concurrent GetTransaction
  probe while the execution RPC is still in flight.
- Lines 249–268 query the saved digest again after an execution RPC error and
  return success if it is checkpointed. Dropping only Execute's response cannot
  guarantee Unknown; a recovered success is correct, not a regression.

**Selected contract:** lose only Execute's response; accept independently verified
in-helper recovery or durable Unknown followed by explicit reconciliation. No
composite blackout, query delay, SDK bypass or discarded proof is permitted.
Client-side observation of the actual Execute transport error is required in
addition to the gateway reset marker; helper cancellation before that result is
FAULT_NOT_REACHED/INCONCLUSIVE, not PASS. A future funded trial still needs a new
receiver, separate approval and real native human consent.

The existing native [execution-request constructor](../crates/tap-sui/src/settlement.rs)
already uses official transaction/signature conversions and the required read mask.
The SDK passes that Execute request through unchanged; response Zstd acceptance
is not request compression. The new keyless seal reuses official prost serialization
and native wallet/history/registration validation, not custom protobuf/BCS/crypto.
The opt-in request-layer observer passes requests/results through unchanged and
distinguishes a received Execute transport error from a future cancelled by SDK
recovery. Direct `prost 0.14.4`, `http 1.5.0`, `tonic 0.14.6`, `tower 0.5.3` edges
reuse existing locked packages; no package version, SDK or toolchain pin changed.
An adjacent read-only Store snapshot permits a separate native first-dispatch
predicate while submit owns its journal lock; it does not initialize schema or
replace normal writable ownership. The owner-locked fixture read path passed below;
its integration with a live dispatch remains unqualified.
Existing read-only proxy files/whitelist and all four evidence JSONs are unchanged.

Preparation also needs real GetServiceInfo, GetEpoch, GetCoinInfo, GetBalance and
checked SimulateTransaction; execution needs SubscribeCheckpoints, ExecuteTransaction
and ledger reads. A separately reviewed future operator proxy must stay at the
same declared loopback endpoint across preparation/submission/reconciliation,
persist a single-use exact-request gate before forwarding, and never replay
Execute automatically. These are compiled candidate capabilities, **not
live-qualified guarantees** beyond the local guards qualified below. Ordinary
native behavior is unchanged when lab options are unused. The initial source-only
phase ran only formatter/metadata/syntax/source checks; the subsequently granted
exclusive heavy validation window ran the offline checks below. No live RPC,
real key/journal access, new funded signature, submission or funding occurred.

Initial source-only checks completed for this candidate:
- `node --check scripts/execution-response-loss-proxy.mjs` — PASS.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo fmt --all --check` — PASS after formatting corrections.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo metadata --no-deps --offline --locked --format-version 1 > /dev/null` — PASS; not compilation or full dependency resolution.
- `git diff --check` — PASS.
- Inline `python3` read-only documentation/evidence check — PASS: 70 local links/anchors,
  five docs' whitespace/newlines, all four exact evidence SHA256/JSON parses;
  canonical checkpoint remains 150 lines. No new evidence excerpt was created.

### Granted exclusive validation window, 29 October

Exact commands/results (all Cargo commands offline; no dependency download):
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh` — initial
  Clippy failure: `transport_trace.rs` assigned `pending.completed` without a read.
  A consuming `Pending::finish` method now captures the whole Drop guard in the
  async future and marks completion there. No lint suppression, assertion change,
  SDK change or recovery-policy change. Rerun **exit 0**: format, Clippy, **15 tests**,
  build and real-process simulated checkout/checkpoint/reopen/replay smoke PASS.
  The initial shell wrapper also used zsh's reserved `status` name; the retained
  compiler log is authoritative, and the corrected wrapper records explicit exits.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo test --workspace --features pcsc-reader --locked` — **exit 0, 15 tests**.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo build --locked -p tap-cli --features pcsc-reader` — **exit 0**, final binary restored with PC/SC support.
- `python3 <PRIVATE_VALIDATION_ROOT>/qualify.py` — **exit 0,
  PASS_LOCAL_OFFLINE_NATIVE_GUARDS_ONLY**, first scenario run. The retained harness
  uses actual keyless native seal/predicates and real HTTP/2 local gateway/client
  processes, never a mocked validator or fake chain response.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo metadata --offline --locked --filter-platform x86_64-unknown-linux-gnu --format-version 1` and the same command with
  `--all-features` — both **exit 0**. The all-features graph's **178 packages** match
  inventory license/source/repository metadata; inventory covers all **209** lock
  package identities. The initial comparison found two pre-existing optional
  omissions (`pcsc 2.9.0`, `pcsc-sys 1.3.0`); only their verified MIT inventory
  entries were added, with no package/dependency version change.
- `python3 <PRIVATE_VALIDATION_ROOT>/integrity.py` — **exit 0**:
  four original evidence SHA256/JSON parses preserved, 25 copied guidance checksums
  preserved, private artifacts owned 0700/0600, all owned proxy/native predicate
  PID/start identities retired. Read-only query proxy remained unmodified, SHA256
  `e2a1ed6714b001dce43200b54756fd6f6d5969897a478a676412acec75420c00`.

The reconstructed public fixture retained exact historical artifact/registration,
with synthetic loopback endorsements and explicitly staged pre-intent/first-attempt
metadata, not new human approval. Native seal export exited **0**; missing first
dispatch exited **1** as expected; first-dispatch validation while the fixture DB
owner was exclusively locked and checked-simulation validation both exited **0**.
Unarmed Execute, altered-frame Execute, an offline allowlisted read, exact offline
Execute, second Execute and restart/second Execute each returned **403, zero bytes**.
Exactly one real native proxy predicate succeeded and exactly one gate was consumed
as `OFFLINE_CONSUMED_NO_UPSTREAM`; gate bytes remained unchanged across replay/restart.
The gateway retained the same `http://127.0.0.1:39987` endpoint throughout.

Native row snapshots before/after are identical. No upstream-start/response event
occurred, and an independent fail-closed `http2.connect` safety interlock recorded
**zero attempts**. Three owned gateway lifetimes exited **0**, and the predicate
child was reaped; synchronous native/local clients were also waited. This was no
funded Execute, no TLS/response-loss qualification, and no runtime proof of actual
SDK transport-error/cancellation observation. No paid-path or observer result is
inferred from a local guard PASS. The coordinator reports independent execution-loss
source/offline-evidence review complete with no material findings; the reviewed
transport observer SHA256 is `d6f11d147f49b597e476e7ee8477f322f6f0da712e875c6c314de4d1b47806da`.

Private retained artifacts: `<PRIVATE_VALIDATION_ROOT>/`:
initial/fixed compiler logs, PC/SC logs, fixture provenance/rows, seal, predicates,
gateway trace/gate, exact private harness, result and cleanup. No SQLite backup or
original user database was opened/copied. Final PC/SC binary: **162037640 bytes**,
SHA256 `bb8d2808223e52d315b7cb41552163d43b22b691ce3dc04c8c44157aaa349aa1`.
Final `node --check`, `cargo fmt --all --check` and `git diff --check` passed;
inline Python validated 71 local links/anchors, clean documentation whitespace and
the unchanged 150-line checkpoint. All four immutable evidence hashes still match.

## B14 read-query response loss qualified, 29 October 2026

**PASS on the first actual run, no software correction/retry.** This is read-query
response loss for an already executed payment in reconstructed lab state, not
fresh execution-response uncertainty. [Sanitized evidence](evidence/query-response-loss-2026-10-29.json).
Independent source and post-run evidence review in `ses_e78c83a42ffeCR2AuKWE6yCZYD`
completed with no blocking/material P1/P2. Review completed after excerpt capture;
its bytes remain preserved. The queued heavy existing checks now pass as recorded above.

Actual command, exit **0**, using the already-built PC/SC-enabled binary:
`python3 scripts/qualify-query-response-loss.py --run-read-only-network-lab --public-snapshot <PRIVATE_VALIDATION_ROOT>/before-fresh-query.json`.
Native control/loss/recovery processes exited **[0,1,0]**, all waited. The expected
middle failure persisted Query/Unknown with no execution payload, reporting
tonic/hyper `Reset(StreamId(1), CANCEL, Remote)` and `UNRESOLVED Unknown; no execution performed`.
The fresh process then verified checkpoint **400484678**, epoch **1267**, digest
`52wFhV654LoLhXycxB5nxn3B2ddM4siC7Mm72opYt9sW`, against the same saved signed artifact.

The proxy received complete upstream HTTP **200** / gRPC **0**, **1285 bytes**,
then sent **RST_STREAM(CANCEL=8)** before any downstream success headers/body.
Lost transaction response SHA256:
`e6ba372cb8cd5f1103a61bdfcf0f190ad6955497f030328ff9f8a76315abe2b6`.
All three transaction request hashes and response body hashes matched; control
and recovered native executions are identical. Exactly **six** official-node
reads occurred: three GetServiceInfo and three GetTransaction; no other forwarded
method, automatic retry, fallback, execution or simulation. No response was fabricated.

Separate control and pending journals were constructed from public row facts;
the control's success was never inserted into the pending journal. Pending
observations went **NotFound → Unknown → VerifiedSuccess**, counts **1 → 2 → 3**.
Candidate/reservation/artifact/intent/attempt each stayed **one**; credits and
receipt associations stayed **zero**. Full artifact JSON, BCS/signature/digest
and every non-observation row stayed unchanged. Endpoint substitutions are
explicit synthetic lab endorsements, not new human approvals or live custody.

Private evidence `<PRIVATE_VALIDATION_ROOT>/` retains provenance,
all four row snapshots, upstream wire bodies/headers, proxy trace, command captures,
result and cleanup; controller capture is under
`<PRIVATE_VALIDATION_ROOT>/`. Directories/files are
**0700/0600**. Cleanup verified **five** recorded PID/start identities stopped
(three native queries, proxy, controller); proxy exited 0, no owned runtime remains.
The three primary evidence hashes and final binary SHA256 are unchanged. No Rust,
SDK, dependency, transport-policy or proxy/harness correction was needed.

No Cargo build/tests/Clippy/check-script ran in this light window; the earlier
SQLite-phase 15-test/default and PC/SC results remain historical. Heavy checks
await the post-render coordinator window. Execution-response loss, host crash,
power-cut/filesystem durability, custody, independent node operators/light-client
proof and authenticated remote merchant ACKs remain unqualified. No real user
journal/key access, new signature, submission, funding or third payment occurred.

Final lightweight checks passed: `node --check` and Python AST parsing; inline
Python public JSON recursive allowlist/roundtrip/source bindings, all six wire
body hashes, exact artifact/outcome preservation, five stopped identities and
unchanged binary/three primary evidence hashes. Also passed **64** relative
links/anchors across four Markdown files, five-file whitespace/final newlines
and `git diff --check`; new excerpt is 65 lines, checkpoint remains 150 lines.

### Implementation record before the read-only validation window

29 October 2026, resource-serialized implementation phase only. Source inspection
of pinned `sui-rpc` identifies exact LedgerService `GetServiceInfo` and
`GetTransaction` paths. Its client retries provably unsent/refused streams;
the lab uses **CANCEL**, not REFUSED_STREAM. Existing `settlement::query` maps
lookup transport errors to Unknown, and native `testnet reconcile` persists that
Query observation before returning unresolved. No product-policy change was needed.

`scripts/query-response-loss-proxy.mjs` uses Node's built-in HTTP/2/TLS solely as
opt-in diagnostic transport, not JavaScript wallet/chain logic. It binds
127.0.0.1:port-0, validates certificates against the fixed official testnet target,
rejects all methods except the two exact reads **before upstream creation**, and
never accepts ExecuteTransaction/simulation or credentials. Request/response bytes,
deadlines and call counts are bounded. Successful upstream bodies/status/hashes
are retained before the second GetTransaction is reset with zero downstream
response bytes; no protobuf success is fabricated or decoded by the proxy.

`scripts/qualify-query-response-loss.py` reuses native reconciliation and existing
signal-safe child/evidence helpers. Only an explicitly selected retained public
verifier snapshot under `/tmp/opencode` is accepted. Two new synthetic journals
contain one original candidate/reservation/artifact/intent/attempt and the first
Query/NotFound row, not later success/association. Wallet endpoint bindings are
marked **RECONSTRUCTED_LAB_STATE**, not new human approvals; exact signed BCS,
signature/digest, invoice and merchant registration remain unchanged. The planned
single scenario is control read → locally lost query response/Unknown → fresh
keyless read of the same saved digest/checkpoint, with no attempt/credit increase.

At this initial phase, syntax/static qualification was not runtime wire evidence;
the read-only network window was still pending. Actual results are now above.
No native command, proxy server or live node read ran in this phase. No dependency,
Rust/SDK/endpoint-policy, historical evidence, wallet/key/ledger, funds or visual-
asset change occurred. Execution-response loss, host crash and power cuts remain
unqualified; no human-funded trial is authorized by this lab.

Pre-window checks: `node --check scripts/query-response-loss-proxy.mjs` passed
with existing Node 22.23.2; inline Python AST parsing passed without execution.
Four-file whitespace/newlines, 20 local Markdown links/anchors, all three primary
evidence SHA256 values and `git diff --check` passed. No new dependency was added.

## B14 SQLite foundation qualified, 29 October 2026

**Scoped PASS, final cleanup verified 10:40:33 UTC.** Native SQL queries report
SQLite **3.51.3**, exact source ID below, and 51 compile options including
`THREADSAFE=1`, `MUTEX_PTHREADS`, `DEFAULT_SYNCHRONOUS=2`,
`DEFAULT_WAL_SYNCHRONOUS=2` and `DEFAULT_WAL_AUTOCHECKPOINT=1000`.
[Sanitized evidence](evidence/sqlite-wal-qualification-2026-10-29.json) separates
in-memory library identity from actual native file-store verification.

Actual commands/results in the coordinator-granted serialized window:
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: PASS,
  formatting, Clippy, all **15 existing tests**, default build and enhanced smoke.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo test --workspace --features pcsc-reader --locked`: **15 passed**.
- Same environment, `cargo build --locked -p tap-cli --features pcsc-reader`: PASS,
  run after default checks; final executable is PC/SC-enabled.
- `target/debug/linux-tap storage-info`: exit 0, native SQL source ID and full
  options captured. This opens memory only, never a real wallet/journal.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode python3 <PRIVATE_VALIDATION_ROOT>/retained-fixture.py`: three native stage pairs completed; retained row facts establish one exact artifact,
  one simulated merchant credit and one simulated receipt across checkpoint/reopen
  and duplicate replay, with all testnet rows zero. Native Store passed effective
  `journal_mode=wal` / `synchronous=2` checks before schema setup; wallet WAL was
  absent after each native clean close. No Python checkpoint was substituted.
  Python SQLite **3.46.1** was used only for readonly fixture inspection.
  The private summary initially assumed a string signature rather than its typed
  object; `retained-fixture.py --summarize-retained` corrected the decoding and
  passed using the same saved facts, with no additional native stage rerun.
- `target/debug/linux-tap --version`: `tap-cli 0.1.0`; `readelf -d target/debug/linux-tap`
  plus retained Cargo build output corroborate bundled static SQLite, not a system
  SQLite dependency. Cargo artifact inode/fingerprint confirms `pcsc-reader`.

Private evidence: `<PRIVATE_VALIDATION_ROOT>/` (0700, files 0600)
retains official release/WAL HTTP bodies with retrieval UTC/status/SHA256,
command captures, full runtime options, linked-build output, synthetic fixture
stage facts/hashes and cleanup. Official HTML SHA256 is distinct from the release
amalgamation's SHA3-256. All **12 recorded PID/start identities** are absent;
every launched command was waited for and no owned runtime remains.
Final SQLite-phase binary (superseded by the candidate build above): **159796368 bytes**, SHA256
`eb95ae3297a0fd332f6dac73f47c437260f867bf5ff01426dd3d1af5cd02fb28`.
The prior two payment evidence hashes are unchanged. No real journal/key access,
RPC, funds, hardware, service, host-fault or VCS mutation occurred. Independent
source and post-validation review completed 29 October 2026 in
`ses_e78c83a42ffeCR2AuKWE6yCZYD`: **no corrective change required, no blocking/material
P1/P2**. Review confirmed retained official HTTP hashes/source identity, native
runtime/static linkage/options, passing checks, exact fixture persistence,
unchanged payment evidence and stopped recorded processes. This is scoped runtime
qualification, not a security audit or baseline race reproduction. B14 is still partial for
wire-level unknown outcomes, host/power/storage failure and broader durability.

Final local checks: inline Python passed public JSON roundtrip/sanitization,
fixed-field/source bindings, both unchanged payment evidence hashes, final binary
hash and PID/start identity absence. Local Markdown links/anchors and touched-file
whitespace/newlines passed, including the new untracked JSON; `git diff --check`
passed. Exact counts and final-check UTC are retained in `final-local-checks.json`
under the private evidence directory. No creative assets were read or modified.

### Source decision and historical pre-build phase

29 October 2026: source inspection confirms the prior lock selected rusqlite
0.37.0 / libsqlite3-sys 0.35.0, whose amalgamation declares SQLite 3.50.2,
source ID `2025-06-28 14:00:48 2af157d77fb1304a74176eaee7fbc7c7e932d946bf25325e9c26c91db19e3079`.
Retained Cargo build-script output selects that bundled source and static
`sqlite3`, not system SQLite. This is build/source evidence, **not a runtime
SQL probe of the old executable**; no baseline runtime result is claimed.

Fresh authoritative sources inspected:
- [Official WAL-reset advisory](https://sqlite.org/wal.html#walresetbug): affects
  WAL with simultaneous writer/checkpointer connections through 3.51.2; fixed
  in 3.51.3, with 3.50.7/3.44.6 backports. No organic corruption was reproduced here.
- [3.51.3 release](https://sqlite.org/releaselog/3_51_3.html), source ID
  `2026-03-13 10:38:09 737ae4a34738ffa0c3ff7f9bb18df914dd1cad163f28fd6b6e114a344fe6d618`.
- [Original fix](https://sqlite.org/src/info/7168988acbec2d8d51106a263e553f8942b8b23d983dbbe5028e0f9be68cbb83) and
  [3.50 backport](https://sqlite.org/src/info/268c9da28706e163e2dfe65a9848e007ef4adc3cc4764042f0081830b88c4a1d) identify the checkpoint
  salt recheck before backfill; the selected 3.51.3 amalgamation contains it,
  whereas the previous 3.50.2 amalgamation does not.
  Selected `sqlite3.c` SHA3-256 matches the official release exactly:
  `32d5424f97e0a7fc5ed2f6335afbb58be4e0298bd7117a34e39d345ff13d859e`.
  The inspected upstream `test/walrestart.test` uses SQLite's Tcl fault-install
  hook 660; it was not imported, built or run. Native smoke is integration
  evidence, not a deterministic reproduction of that race or power loss.

Store uses one connection per cooperative DB-file owner lock, with automatic
WAL checkpoints and FULL synchronization. That limits application concurrency,
but is not SQLite EXCLUSIVE mode or a lock honored by external SQLite tools;
it does not establish immunity to the advisory's interleavings. Wallet and
merchant journals remain separate authorities, not an atomic cross-file commit.

The bounded fix selects rusqlite 0.39.0 / libsqlite3-sys 0.37.0, the earliest
reviewed maintained crate pair bundling 3.51.3. Hashlink's required closure and
rusqlite's WASM-only dependencies changed; SDK/toolchain and unrelated pins did
not. The resolver's incidental WASI downgrade was explicitly restored.
Downloaded hashlink 0.11.1, libsqlite3-sys 0.37.0, rusqlite 0.39.0, rsqlite-vfs
0.1.1 and sqlite-wasm-rs 0.5.5 only, checking each archive against Cargo.lock
SHA256. Other new closure archives were already cached. License inventory updates
use checksum-verified crate manifests; no system package or private fork was added.

Native `storage-info` queries `sqlite_version()`, `sqlite_source_id()` and compile
options on an in-memory connection, with no path/key/RPC input. Store now checks
effective WAL/FULL before schema setup: WAL selection can otherwise silently
leave another mode. The existing real-process smoke adds exact artifact/credit
byte checks across native close/checkpoint/reopen and duplicate replay, with
owned-child identity logging/cleanup. It uses only new public-fixture state.

**At this initial phase, build, runtime SQL identity/options, file-fixture execution
and independent review were pending the coordinator's validation window.**
No compilation, native runtime, live journal/key access, funds activity, host
fault or power-cut trial ran in this implementation phase. The original two
testnet evidence excerpts remain immutable. README is untouched until that window.

Pre-build checks actually run (subsequent runtime results are above):
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo update -p rusqlite --precise 0.39.0 --offline`, then targeted `cargo update -p wasi --precise '0.11.1+wasi-snapshot-preview1' --offline`: resolution only, no compilation.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo metadata --no-deps --offline --locked --format-version 1`: passed.
- Same environment, `cargo metadata --offline --locked --filter-platform x86_64-unknown-linux-gnu --format-version 1`: passed. Unfiltered metadata stopped on an uncached pre-existing Windows crate; no unrelated download was made.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo fmt --all --check`: passed after correcting formatting in the new Rust lines.
- Inline Python parsed smoke syntax without execution, checked changed-closure archive checksums/license metadata and unchanged SDK/WASI pins, matched the official amalgamation hash and confirmed both original testnet evidence hashes unchanged.
- `git diff --check`: passed. Compilation, tests and runtime results were still pending at this source-only phase, not inferred from these checks.

## Real application-boundary recovery verified, 29 October 2026

**PASS for one new human-approved 0.01 Circle testnet USDC payment**, digest
`52wFhV654LoLhXycxB5nxn3B2ddM4siC7Mm72opYt9sW`, checkpoint **400484678**,
epoch **1267**. Independent verifier `ses_e7ccea915ffeYUiCnAyPFmRDnJ` checked
retained stages and one fresh keyless query at **09:47:53.808602 UTC**.
[Sanitized durable excerpt](evidence/emulated-testnet-recovery-2026-10-29.json)
contains fixed public identifiers, bindings, counts and qualified scope.

- Human runner's seven distinct native stages exited `[86,0,86,0,0,0,0]`:
  submit fault, reconcile, accept fault, accept restart, confirm, duplicate
  accept, duplicate confirm. Initial `NOT_DONE` was a start notice; final result
  was `PASS_REAL_TESTNET_APPLICATION_BOUNDARY_RECOVERY`.
- Submit exited after received verified success but before Dispatch observation:
  one artifact/intent/attempt, only Query/NotFound, no association. Restart
  reconciliation verified that exact saved transaction and checkpoint.
- Accept fault committed one new linked merchant observation/credit before
  receipt export; the boundary snapshot independently shows no wallet association.
  Historical file absence rests on `receipt_exported=false` plus the reviewed
  runner's contemporaneous absence assertion, **not a retained filesystem-absence
  snapshot**. Ordinary restart regenerated the exact committed receipt bytes;
  confirm queried before association. Duplicate accept/confirm used caches.
- Full BCS/signature/digests, public key, typed sender SPLIT 10000/receiver MERGE
  10000 and private-PCSC invoice binding agree with the saved artifact/fresh node.
  The verifier ran only one new reconcile (exit 0), appending wallet observation
  3 → 4, not another dispatch. No new signature, payment or acceptance was made.
- Trial candidate/reservation/artifact/intent/attempt/association stay one each.
  The authoritative ledger has **two registrations, two observations, two credits**:
  preserved first payment plus exactly one new trial credit. All baseline row
  fingerprints survived every stage; simulated payment/receipt counts remain zero.
  Original artifacts and stored/regenerated/duplicate receipt bytes stayed identical.
- Source public evidence is retained at
  `<PRIVATE_VALIDATION_ROOT>/`: `REPORT.md`,
  `result.json`, `retained-stage-facts.json`, `source-file-sha256.json`,
  `fresh-node-observation.json`, `before-fresh-query.json`, `after-fresh-query.json`.
  Seven historical processes were stopped; the one fresh CLI was waited for;
  source reports no owned runtimes. This increment launched no product runtime.

Both receivers are now used. Preserve receiving custody/journals and the original
merchant ledger; no empty-ledger reset or third trial is authorized. B14 remains
partial for wire-level unknown outcomes, host/power failure and broader durability.
Same chosen official node in both roles, not independent operators/light-client
proof; no RF, remote authenticated business ACK, production custody or security-
audit qualification. No code/build/test/network/key/wallet/ledger/funds action,
subagent, service or VCS mutation occurred in this documentation-only persistence.

Local docs checks: inline `python3 - <<'PY'` checks passed JSON roundtrip,
fixed recursive public-field allowlists, source hashes/bindings/counts and the
unchanged first-evidence SHA256; the excerpt is 88 lines. The local check's
initial RPC-field assumption was corrected to parse string checkpoint and
`effects.epoch`; no product/evidence change or RPC was needed. Inline Python
also passed 134 relative inline links/anchors across 20 Markdown files and
whitespace/final newlines for all seven changed files, including untracked JSON.
`git diff --check` passed; `git rev-parse HEAD` returned unchanged
`f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`. No build or product tests were run.

## Recovery-runner signal-cleanup P2 fix, 29 October 2026

Historical pre-live-trial review record; subsequent application-boundary proof is above.

Independent review `ses_e7d355137ffeaXiT10YXbCLGve` found one P2: parent
SIGTERM/SIGHUP could bypass finally and leave its consented native child alive.
**Focused re-review completed 29 October 2026 in the same session: P2 RESOLVED,
no remaining material findings.** Source, negative guards and real harmless
TERM/HUP/INT probes (launch deferral, repeated signals, owned reaping, STOPPED
evidence and handler restoration) were read-verified. No live fault branches
were exercised. Human fresh-receiver preparation and separately approved
0.01 USDC trial remain pending; no agent funds activity is authorized.
Docs-only review closure updated README, qualification guide/current anchors,
this record, backlog and product design. Local checks passed 132 relative links/
anchors across 24 Markdown files, changed-file whitespace/final newlines and
`git diff --check`; all 60 recorded guard/probe PID/start identities were absent.
The existing binary inode matches Cargo's `pcsc-reader` bin artifact; no rebuild
or executable/RPC check was performed. No code, tests, network, wallet/ledger
access, services or VCS mutations occurred; HEAD remains
`f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`.
Only `scripts/testnet-recovery.py` product code changed (337 → 423 lines).
SIGTERM/SIGHUP/SIGINT request planned `BaseException` unwinding, without handler
I/O. Launch defers delivery until the owned child handle/ledger is tracked;
repeated signals cannot interrupt cleanup. Only owned `Popen` handles are
terminated/waited, with five-second TERM then KILL waits. STOPPED evidence records
signal/exit code, no later stage runs, cancellation removes only its own partial
PASS output, and prior handlers are restored. No shared demo/native/SDK changes.

- `python3 <PRIVATE_VALIDATION_ROOT>/probe.py`: passed
  twice, actual OS SIGTERM/SIGHUP/SIGINT on parent and harmless blocking child,
  mixed repeated parent signals during cleanup and SIGHUP inside launch's handle
  handoff window. Actual runner main/run/cleanup were used; only payment execute
  was replaced in the private probe with a harmless OS stage. No product test
  hook, fake RPC/chain success, key, signing or submit. Each case reaped its child,
  retained STOPPED/capture/cleanup evidence, restored handlers and had no later
  stage or PASS. Final evidence:
  `<PRIVATE_VALIDATION_ROOT>/run-1793237813894248415/`;
  first pass `run-1793237710279926512/` also retained.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py`: negative
  guards passed again, final `guards-1793237813527424671/` under that root;
  earlier pass this increment `guards-1793237727651607200/` retained.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  formatting, Clippy, all 15 existing tests, default build and native pipe smoke.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo test --workspace --features pcsc-reader --locked`:
  all 15 tests passed; same environment with
  `cargo build --locked -p tap-cli --features pcsc-reader` passed **after** default
  checks. Final binary remains PC/SC-enabled; no dependency cache rebuild/download.
- Final local checks passed: Python syntax, 132 relative links/anchors across
  24 Markdown files, 20 guide shell snippets (syntax only), changed-file whitespace/
  newlines, milestone JSON syntax and `git diff --check`. All 60 recorded guard/
  probe PID/start identities (including controllers) were absent; all six signal
  cases retained STOPPED evidence, no later stage/PASS and private 0700/0600 files.
  HEAD remains `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`.

Live success-fault branches remain **UNEXERCISED_AGAINST_LIVE_CHAIN**. This is
process signal/reaping evidence, not real-chain interruption proof. After possible
broadcast, STOPPED does not mean no payment: reconcile the saved digest; never
re-sign or create a replacement operation. SIGKILL, root/kernel crash, uninterruptible
OS failure and unavailable evidence storage are outside this guarantee. No real
wallet/checkout/merchant-ledger access, RPC, live signing/broadcast, services,
subagents or VCS mutations occurred; prior dirty work/milestone evidence remain.

## Application-boundary recovery checkpoint, 29 October 2026

Historical pre-signal-review checkpoint; the scoped P2 fix is recorded above.

**Implemented; independent review pending. Both success fault branches remain
UNEXERCISED_AGAINST_LIVE_CHAIN.** The first verified checkout below is unchanged;
no new live trial, real key access, live signing, wallet-key entropy or broadcast occurred.

`testnet/settlement.rs` adds two opt-in, per-invocation 0.01 USDC fault flags:
submit exits 86 only after received verified first-execution success and before
observation persistence; accept exits 86 only after new atomic credit/observation
commit and before receipt export. Both flush a digest-bound marker and bypass
destructors. Replay/existing-intent/historical-terminal and existing-output/credit
guards prevent rearming; unknown/failed results keep ordinary persistence and
`FAULT_NOT_REACHED`. This is lost application-received evidence, **not a dropped
RPC packet or power cut**. No SDK, schema, dependency, service or confirm split.

New `scripts/testnet-recovery.py` (337 lines) uses the real terminal/native
cryptographic gate, not automatic approval/signing/preparation. It fingerprints
bounded read-only WAL-aware row facts, checks all old authoritative-ledger rows
and explicitly chosen prior credit, then uses new native processes for both
fault/recovery stages, fresh reconcile/confirm and cached duplicate acceptance.
Full original artifact bytes, one new attempt/credit/association, receipt bytes,
per-invoice state and total baseline+one credit are checked before any PASS.
It has no resubmit path; unknown/timeout/mismatch retains state without PASS.
Optional retained PC/SC evidence is byte-bound, not transport performed by runner.
The staged guide reuses the old funded sender/merchant and authoritative ledger,
allocating only a new controlled receiver through later human-private init.

Actual checks:
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo check --workspace --locked`:
  passed; `cargo fmt --all` and Python `compile()` syntax check passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  format, Clippy, all 15 existing tests, build and native offline smoke.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py`: passed four times
  (`PASS_NEGATIVE_GUARDS_ONLY`), no signing/network/broadcast. Actual CLI parser
  conflict, offline refusal, existing-output preservation and runner offline/
  prior-intent-metadata/autoapproval/key-option guards passed. No mocked live
  success was supplied. Native live existing-intent/cached-credit branches were
  source-inspected, **not invoked with the settled user artifact**.
- First temporary harness attempt stopped on an archived fixture's older schema;
  isolated fixture setup was corrected to current additive tables, not a product
  assertion weakened. No live database was copied or historical fixture modified.
- Final guard evidence:
  `<PRIVATE_VALIDATION_ROOT>/guards-1793236935930180875/`;
  earlier passes `guards-1793236221192161637/` and `guards-1793236515729648956/`,
  `guards-1793236793974467087/` and the initial stopped capture are also retained.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo test --workspace --features pcsc-reader --locked`:
  all 15 tests passed. Same environment with
  `cargo build --locked -p tap-cli --features pcsc-reader`: passed **after** default
  validation; final binary remains PC/SC-enabled. No cache downloads/rebuilds.
- `python3 scripts/testnet-recovery.py --help`, native `testnet submit --help` and
  `testnet accept --help`: passed, flags/recipe checked against actual clap.
- Final local checks: 132 relative links/anchors across 24 owned Markdown files,
  20 guide shell snippets (`sh -n`, syntax only), Python syntax and changed-file
  whitespace/newlines passed; original evidence JSON parses. All 32 recorded
  owned PID/start identities were absent; nine stopped runner invocations had
  no native child or PASS. Evidence permissions were 0700/0600; `git diff --check`
  passed. HEAD remains `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`.

Live fault/restart, actual receipt regeneration and end-to-end runner PASS remain
pending independent review plus separately approved human trial. Existing public
receipt presentation matched the runner's strict pretty-byte golden, not live
fault proof. No new helper unit-test matrix, hardware, custody/power-loss or
authenticated remote-ACK claim; original user journals/keys were not accessed.

## First emulated real testnet checkout verified, 29 October 2026

**PASS: one complete human-run native private-PC/SC real Sui testnet checkout.**
Independent verifier `ses_e7ccea915ffeYUiCnAyPFmRDnJ` checked it at 00:39 UTC.
[Durable sanitized public excerpt](evidence/emulated-testnet-checkout-2026-10-29.json)
records identifiers, bindings, effects, essential counts and trust limits;
it is evidence, not a new test fixture or production-readiness claim.

- Human invoice/register/delivery/prepare/submit/accept/confirm completed
  **10000 units / 0.01 Circle testnet USDC**, digest
  `7LemioWwAmjRYA9gqsZZBCnCnUgpESH3qfg7mJzJCJxz`, checkpoint **400344020**,
  epoch **1267**. Human funding/conversion and direct payment are distinct stages.
- Native T=1 AF_UNIX pcscd/vpcd/VICC/Rust delivery binds exact invoice, inbox,
  candidate and transport snapshots (shared SHA256 in the excerpt). Saved human-
  approved SDK transaction BCS, digests, public signature/key match the node.
  Typed effects show sender SPLIT 10000 and receiver MERGE 10000; this response
  omitted `balanceChanges`, and no current balance snapshot is claimed.
- Original dispatch intent/attempt preceded execution; original merchant accept
  queried its chosen node before credit, and original confirm queried before
  wallet association. Verifier `testnet reconcile` exit 0 made **one fresh
  execution observation**, same checkpoint. Duplicate `accept`/`confirm` each
  exited 0 using cached linked validated credit/association, **not fresh queries**.
- Counts stayed one candidate/reservation/artifact/intent/attempt, one merchant
  registration/observation/credit and one wallet association. Wallet observations
  alone changed 3 → 4; both DBs retained zero simulated payments/receipts.
  Original invoice/registration/inbox/prepared/receipt and exported replayed
  receipt stayed byte-identical. No new payment, signature or dispatch occurred.
- Source captures: `<PRIVATE_VALIDATION_ROOT>/`:
  `REPORT.md`, `result.json`, `before.json`, `fresh-node-observation.json` and
  `pcsc-binding.json`. `after.json` is absent; after counts are in
  `result.json.after_counts`. No source capture was rewritten.
- Verifier's three keyless CLI processes exited normally and were waited for;
  the capture reports no owned runtimes. This documentation increment launches
  no product runtime and performs no builds/tests, network, wallet/key access,
  signing, funding, broadcast, services or VCS mutations.

Limits: same official fullnode in both roles, not independent operators or a
light client; virtual PCSC, not RF/mainnet. No real lost-RPC/lost-receipt/restart
or power-cut injection occurred; prior simulated fault evidence is unchanged.
Not production custody, a security audit or authenticated remote business ACK.
The receiver is now used: retain receiving custody/journals and do not reset
reuse with a new ledger. Broader B14 fault qualification remains partial and
requires separate approval without an implicit new payment. Hardware stays deferred.

Docs/evidence-only persistence checks: saved-source field/count bindings and
sanitization passed; all three evidence JSON files parse. All 130 relative links/
anchors across 24 owned Markdown files, changed-file whitespace/final newlines
and `git diff --check` passed. The three recorded verifier PIDs are absent;
no product runtime was launched. HEAD remains `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`.

## Public-invoice private PC/SC delivery seam, 28 October 2026

Historical pre-checkout implementation/review record; current milestone is above.

**New seam implemented, locally validated and independently reviewed on
28 October 2026 (`ses_e7d4e67afffeE9pFbLkvW2fFdS`), with no blocking/material
P1/P2.** No actual merchant payment, merchant credit or wallet
confirmation was performed in this increment.

Read-only delivery/workflow review verified bounded regular no-follow snapshot,
native ACK/transcript/exact bytes, receiver exit and cleanup before atomic
no-replace publication; mode 0600/link count 1, source preservation and unchanged
default-simulation isolation. Existing evidence included a 690-byte fixture,
seven APDUs, no application DB and 24 absent runtime identities. Review ran no
fresh tests/builds, network, keys or services. Limits remain virtual public
fixtures, not actual merchant payment, RF, custody or power-cut proof. Human-only
0.01 USDC preparation (steps 1–3) is next; submission still needs separate approval.

Docs-only closure updated six documents/current anchors. Local Python checks
passed 50 relative links/anchors, whitespace/final newlines and absence of all
60 recorded owned PID/start identities across five runtime cleanup records;
the private daemon prefix is idle. `git diff --check` passed; HEAD remains
`f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`. No code, builds, product tests,
network, wallet access, services or VCS mutations were performed in this closure.

Prior human conversion is now evidenced separately from transport: the human
completed the 20 USDC same-owner conversion, digest
`4DoX6pEdoo7ScZy5Pda2Yeuk3XkQ89Kyxw8s5BqBP121`, checkpoint `400301849`,
epoch 1267. Previously saved keyless reconciliation reports verified success
and indexed address balance `20000000`. The separate public payment
qualification at epoch 1267/protocol 139 reports the pinned Circle type/genesis,
20,000,000 address-balance units and a successful checked unsigned 10,000-unit
payment, digest `9hwpZFoWRcH6oXny3btUPnF6Ukw3bU3s7doji4rawc8s`, with exact
transaction/inner/effects identity, `qualified=true`, no blockers and
`chain_submission=false`. This increment read only those existing public files
under `<PRIVATE_VALIDATION_ROOT>/`; it made no
network query and did not access a wallet. Human conversion is not merchant
settlement and does not certify hardware, power-loss behavior or custody.

Product change is confined to `scripts/pcsc-lab.py`: 340 → 443 lines (+103 net).
New mutually exclusive public `--invoice`/`--out` delivery mode reuses the
existing cached-prefix checks, owner lock, native reset probe, Rust `nfc-push`,
Unix-only PC/SC/vpcd/VICC path, transcript verification and PID/start cleanup.
No Rust, dependency, key loader, payment policy, service or crypto code changed.
The shared private-directory check now also requires a directory, necessary for
safe output parents. Input is a bounded regular-file snapshot; native parsing
and frames remain authoritative. Inspected native `MAX_ENVELOPE/MAX_MESSAGE`
is **8192**, with 220-byte chunks, not 2048; no native bound was changed.

Delivery-only completion requires native ACK, exact inbox/source bytes, native
receiver EOF/exit 0, exact APDU requests/success responses and owned-runtime
cleanup. Only then are complete output bytes atomically published without
replacement in an owned canonical mode-0700 parent, final mode 0600/link count 1.
No wallet/merchant database, simulated checkout or artifact-survival claim is
created by delivery mode. Result is
`APDU_DELIVERED_NOT_APPROVED_NOT_PAID`, `chain_submission=false`,
`wallet_approval=false`, `payment=false`. The original Stage B assertions remain
intact and passed again. Python does not interpret invoice policy or APDUs.

Checks actually run, serialized with the cached prefix (no downloads/rebuilds
of dependencies, host daemon/service or namespace changes):

- Python `compile()` syntax check of `scripts/pcsc-lab.py`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  formatting, Clippy with warnings denied, all 15 existing tests, default build
  and native pipe smoke. Rerun after final product edits: passed.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py guard`:
  passed. The default-feature executable refuses PC/SC with the expected rebuild
  error; no output/success result was published and cleanup passed. Evidence
  `guard-1793224624816136947/`, runtime `<PRIVATE_VALIDATION_ROOT>/`.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo test --workspace --features pcsc-reader --locked`:
  passed all 15 existing tests.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo build --locked -p tap-cli --features pcsc-reader`:
  passed **after** default validation. This is the final binary feature state.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py delivery`:
  passed twice, using an existing public synthetic SignedInvoice, no private
  key reads, wallet-key entropy, live signing, RPC or broadcast. Real CLI/PC/SC delivery
  produced exact output bytes, successful APDU transcript, receiver exit 0 and
  no application DB. Reusing an existing output failed without changing its
  bytes/mtime. The original `--approve-simulation` Stage B checkout still passed
  lost-receipt/restart/exact-artifact replay assertions. No mocked paid result
  or helper unit-test matrix was introduced.
- Final acceptance root:
  `<PRIVATE_VALIDATION_ROOT>/delivery-1793224902643193450/`.
  Delivery `<PRIVATE_VALIDATION_ROOT>/result.json`; original Stage B
  `<PRIVATE_VALIDATION_ROOT>/result.json`. Earlier pass retained at
  `delivery-1793224645730713256/` (runtimes `ltap-pcsc-x9sx05t5`, `ltap-pcsc-y8mp80zh`).
- Final local documentation/evidence check: 50 relative links/anchors,
  whitespace/final newlines in six docs and `sh -n` syntax-only checks on all
  31 shell snippets passed. All 60 recorded owned PID/start identities across
  five runtimes were absent; cleanup records, 0700/0600 evidence permissions
  and idle private daemon prefix were verified. `git diff --check` passed.

README, qualification guide, backlog, product design and PC/SC guide now
distinguish completed human conversion/public eligibility from still-pending
merchant checkout. The exact five-step human recipe is staged in the guide,
independently reviewed; steps 1–3 stop after public preparation
output before later separately approved submit/accept/confirm. Both chosen RPC
endpoints are the same official operator, not independent-operator evidence.
Local receipt files are not authenticated remote business ACKs.
No actual user invoice/signature, new wallet entropy, keys, faucet, broadcast,
production custody or physical RF was exercised by the agent. Prior dirty work
remains uncommitted; HEAD is unchanged at
`f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`.

## Native conversion lifecycle continuation, 28 October 2026

Historical implementation/review checkpoint. The subsequent actual human
conversion and public reconciliation/qualification are recorded above.

**Implemented; independent lifecycle review completed 28 October 2026.
Explicit human conversion approval and private actual signing remain pending;
no live conversion or settlement performed.** This supersedes the incomplete
inspection checkpoint below, which is retained as historical evidence.

Lifecycle review `ses_e7c57274cffe7qT1NSnKx1gggB` reported no demonstrated P1/P2;
the previous read-only `0x403` P2 is resolved. Review used source, public unsigned
qualification and public-fixture acceptance/logs only. It checked exact template,
self-recipient, full coin/reference, epoch and zero-gas binding; pre-sign
reservation and key/dump ordering; typed approval/recovery/artifacts and offline
guards; digest-first intent-before-execution; conservative unknown handling and
checkpoint effects predicates. The 1,537 lines were bounded to four actual
responsibilities with no redundant framework found. Review itself made no actual
live signature, execution, checkpoint conversion or payment, and grants no
execution authority or power-loss/production-custody qualification.

Documentation-only review closure: changed README, qualification guide, backlog,
product design and this record only. Local Python checks passed 46 relative
links/anchors plus whitespace/final newlines; all 170 recorded owned PID/start
identities across 27 local transcripts were absent. `git diff --check` passed;
HEAD remains `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`. No code changes,
builds, product tests, network, private-wallet access or VCS mutations occurred
in this closure.

Independent source review `ses_e7c57274cffe7qT1NSnKx1gggB` found the single P2
false blocker: rejecting the read-only epoch deny-list. The narrow fix accepts
only a singleton `UnchangedConsensusKind::PerEpochConfig` at `0x403`; no version
or owner is invented for that variant. Pinned SDK `1313d51`
`crates/sui-sdk-types/src/effects/v2.rs` lines 68–73 and 173–174 describe
unchanged consensus reads and per-epoch configuration. The installed official
GraphQL schema also identifies the epoch CoinDenyList as exactly `0x403`.
The runtime-loaded child is not a mutation exception. Exact two-change
coin-deletion/full-same-owner-credit checks remain intact; no zero-accounting
counter requirement or SUI fallback was introduced.

Exactly one corrected public checked unsigned inspection was run:

```sh
target/debug/linux-tap testnet conversion inspect --sender 0xb6b852118f51421abf2a930216ea62b88a283b5d0be7de326695bc23768d14f1
```

Exit **0**, `qualified=true`, no blockers, `chain_submission=false`.
Official endpoint, pinned genesis, epoch 1266/protocol 139, checkpoint
`400242202`; exact selected coin/reference and candidate digest match the prior
20 USDC inspection. Full response and exact BCS are retained under
`<PRIVATE_VALIDATION_ROOT>/` in `inspection.json`,
with separate stderr. This demonstrates the unsigned candidate only: neither
conversion, live signature, execution RPC nor address-balance funding occurred.

Product paths added/extended:

- `tap-sui/src/conversion.rs`: exact saved-candidate qualification as well as
  selection, reconstruction guard and shared simulation/checkpoint typed effects
  predicate (446 lines).
- `tap-sui/src/conversion/settlement.rs`: conversion-specific digest-first
  observation, exact checkpoint/signature/effects classification, bounded SDK
  execution and separate indexed address-balance reporting (260 lines).
- `tap-cli/src/testnet/conversion.rs`: separate typed candidate/qualification/
  approval/artifact/history validation, terminal preparation, deterministic
  recovery and keyless controlled dispatch/reconcile (716 lines).
- `tap-cli/src/store/conversion.rs`: additive versioned one-operation records
  and history, with transactional intent/attempt commit (115 lines), initialized
  from the existing `store.rs`. Existing owner locking and FULL synchronous
  SQLite durability are reused; old tables and migrations are not replaced.
- `tap-cli/src/testnet.rs`: updated routing description and directly necessary
  adjacent ownership fix: private-file/parent checks use Linux `geteuid`, not
  `/proc/self` ownership, which may become root after checked dump protection.
  The same existing key loader/export routines remain in use.
- README, qualification guide, backlog and product design describe implemented
  versus still-unqualified behavior and the gated human command recipe.

No dependency, crate, pin, invoice template/wrapper, merchant settlement
contract or production custody promise changed. No generic operation-policy
framework or new test suite was added. Separate conversion observation code is
necessary because payment predicates are invoice-bound; sharing them through a
fake invoice would permit the wrong semantics. Official SDK types, BCS,
signing, verification and execution helpers are reused.

Validation commands and actual results:

- `cargo fmt --all`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo check --workspace --locked`:
  passed, followed by `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo build --locked -p tap-cli`:
  passed.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py`:
  **PASS_PUBLIC_FIXTURE_CLI**, real CLI with a real PTY and official SDK-encoded
  known public seed `[9;32]`. No entropy, network, live keys or live signatures.
  Decline leaves only the frozen candidate; a deliberately missing fixture key
  proves reservation precedes key loading. Normal retry rejects unresolved
  issuance, explicit recovery signs the same saved candidate, re-export is
  immutable, and a fixture-only artifact-loss scenario reconstructs exactly
  the same signed artifact. Tampered recovery metadata rejects before reuse.
  Keyless submit/reconcile reject the offline fixture before RPC. No mocked
  paid/checkpoint outcome or synthetic live qualification was created.
- Acceptance result/transcript:
  `<PRIVATE_VALIDATION_ROOT>/run-1793211003364697492/`.
  A second pass at `run-1793211343718647581/` also exercised the explicit
  `--replay-saved` flag's offline-fixture rejection. All 20 recorded owned
  PID/start identities across both runs were absent; fixture key payloads were
  absent from transcripts. Evidence directories/files were checked as 0700/0600.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  format, Clippy with warnings denied, all 15 existing tests, build and native
  separate-process smoke (`chain_submission=false`).
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo check --workspace --features pcsc-reader --locked`:
  passed, compilation only.
- `target/debug/linux-tap testnet conversion prepare --help`, `submit --help`
  and `reconcile --help`: passed, confirming the documented CLI surface.
- `git diff --check`: passed. HEAD remains
  `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`; no staging/commit/push occurred.
- Local Python documentation check: 46 relative links/anchors and final
  newlines in all five edited documents passed.

Remaining validation gaps: explicit human conversion approval, human-private
live preparation/signing, actual execution and timeout/replay reconciliation,
checkpoint conversion verification and indexed address balance on controlled
accounts. Merchant payment qualification, independent merchant credit, live
PC/SC orchestration, physical hardware and production custody remain separate
unproven gates. Faucet completion was not conversion approval. No real wallet
directory or keystore was accessed, no background service was started and no
funds were moved by this work.

## Native conversion inspection, 28 October 2026

Historical inspection-only checkpoint; superseded by the continuation above.

Added only the public read-only `testnet conversion inspect --sender ...`
product slice. The SDK module is separate from invoice payment qualification;
its only shared change is crate-visible reuse of the existing exact simulated
transaction/digest/status check, followed by conversion-specific typed V2 effects
validation. Existing invoice wrappers and checks are unchanged. No dependency,
manifest, signing, database schema or settlement contract changed in this slice.
The two new conversion modules contain 410 lines after formatting.

Human-private init/verify-setup success and Circle faucet completion are
human-reported. The private wallet directory was not accessed. Exactly one
public unsigned inspect run used the supplied public sender and fixed official
testnet endpoint:

```sh
target/debug/linux-tap testnet conversion inspect --sender 0xb6b852118f51421abf2a930216ea62b88a283b5d0be7de326695bc23768d14f1
```

Exit **1**, `qualified=false`, `chain_submission=false`. The checked simulation
itself returned success, deleting the selected 20 USDC coin and merging exactly
20,000,000 units to its owner's accumulator. The verifier rejected the system
`PER_EPOCH_CONFIG` read at `0x403`; this policy mismatch is unresolved, not a
network execution failure. No verifier relaxation or second network run was
made. Exact reference, candidate digest, scope and remaining gates are recorded
in the [qualification guide](testnet-qualification.md#native-conversion-inspection-28-october-2026).
Public response/BCS and stderr are retained locally in the mode-0700 directory
`<PRIVATE_VALIDATION_ROOT>/` (mode-0600 files).
An initial invocation used the nonexistent binary path `target/debug/tap-cli`
and exited 127 before any RPC; the correct binary is `linux-tap`.

Local validation after implementation:

- `cargo fmt --all`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  formatting, Clippy with warnings denied, all 15 existing tests, CLI build and
  separate-process smoke (`chain_submission=false`).
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo check --workspace --features pcsc-reader --locked`:
  passed; compilation only, not hardware evidence.
- `target/debug/linux-tap testnet conversion --help` and
  `target/debug/linux-tap testnet conversion inspect --help`: passed.
- A local Python subprocess check asserted exit 2 for
  `testnet conversion submit` and
  `testnet conversion inspect --sender 0x1 --keystore /does-not-exist`:
  passed; no RPC or key file opened by these parser rejections.
- `git diff --check`: passed. No new tests or subagents were added.
- Local Python link/permission check: 44 relative links/anchors and final
  newlines across the five edited documents passed; evidence directory/file
  permissions confirmed as 0700/0600. HEAD remains
  `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`.

The first development `cargo check --workspace --locked` failed because the
pinned SDK has no `ExecutionStatus::is_success`; corrected to enum comparison
before the successful build/checks above. This is not an outstanding failure.

**Incomplete product phase:** conversion prepare, durable approval/reservation,
immutable artifact/recovery, human-private signing, keyless dispatch/reconcile
and checkpoint-completion verification are not implemented. The conservative
inspection verifier also needs source/effects-coverage review before becoming
an authorization predicate. There is no ready human conversion command, funded
address-balance qualification, real conversion, merchant payment or settlement
claim. No real key, execution RPC, faucet, background service, physical device
or VCS mutation was used; all pre-existing dirty work remains uncommitted.

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

## Explicit Circle testnet configuration (22 October 2026)

- Workmux's `oc2` profile runs interactive OpenCode V2 via
  `<OPENCODE_BIN>/opencode2`; `oc2` is a profile, not a shell binary.
- Independently read Circle's official [type registry](https://developers.circle.com/stablecoins/usdc-contract-addresses)
  and [faucet information](https://faucet.circle.com/). The exact Sui Testnet
  type agrees with live six-decimal metadata and protocol 139's lossless allowlist.
  No symbol-based identity or automatic cross-network asset allowance was added.
- `bash scripts/check.sh`: passed formatting, Clippy with warnings denied, all
  15 existing tests, native CLI build and pipe smoke (`chain_submission: false`).
  Existing tests/smoke sign public offline fixtures only; no new tests were added.
- `cargo build --locked -p tap-cli`: passed with unchanged SDK/dependency pins.
- Ran the explicit public-address command in [the qualification guide](testnet-qualification.md):
  exit 0, `qualified: true`, `chain_submission: false`, epoch 1260, checkpoint
  397916564. Address `0x0` already had 3,400,000 test USDC address-balance units.
  The [saved excerpt](evidence/circle-testnet-qualification-2026-10-22.json) was
  recursively compared with retained fields in the full live report: matched.
- The initial live run exposed an existing verifier/API mismatch: simulation
  omits outer `ExecutedTransaction.digest`, but supplies the inner transaction
  digest and effects transaction digest. The pinned protobuf declares these
  separate optional fields. The verifier now requires the inner digest and
  effects digest to equal the SDK digest, rejects any conflicting outer digest,
  and still requires identical SDK transaction bytes and successful effects.
  The subsequent live run above passed those checks; receipt verification is unchanged.
- Also ran the guide's original command **without** the explicit flag: exit 1,
  unchanged mainnet-type rejection on testnet. No substitution occurred.
- Ran the explicit command with `--chain-id 11111111111111111111111111111111`:
  exit 1, `chain pin mismatch`. With `--amount 9999`: exit 1, minimum rejection.
  With `--sender 0x42 --receiver 0x1 --amount 10000`: exit 1, insufficient
  address balance and checked simulation rejected the unfunded withdrawal.
  All other arguments matched the documented explicit command. A diagnostic
  `--sender 0x1 --receiver 0x2` run also passed; that address was already funded too.
- `target/debug/linux-tap qualify-testnet --help` and `git diff --check`: passed.

This is **existing-public-address simulation**, not fresh-wallet funding,
control of those public addresses, live execution or settlement. No keys were
created/imported, no faucet request was made, and no real transaction was signed
or broadcast. The research worker separately reported a coin-to-address conversion
simulation; it was not rerun here or turned into a submission implementation.
Fresh-wallet validation awaits user-operated local key provisioning and faucet
CAPTCHA. Coin-object conversion, signing-profile integration, submission and
reconciliation remain blockers to end-to-end checkout. See the guide for exact
commands and stop conditions. Reader/vendor code is unchanged; PC/SC, physical
NFC, hardware approval and Omarchy-host qualification were not retested.

## Integrated APDU ingress and lost receipt (28 October 2026)

Local software implementation/checks on the pinned Rust 1.99.1 toolchain; no
dependency, SDK, protocol or asset-policy changes. This increment adds native
`apdu-frames`, optional `pay-demo --invoice-inbox`, `--replay-only` and the
fixture-only `serve-demo --drop-receipt-after-commit` fault. The inbox must match
the private socket's entire signed envelope before verification/approval.
Fresh candidate bytes/digest are fixed before approval, expiry is rechecked
before signing, and saved-artifact replay never builds a new nonce. Store holds
a cooperative OS DB-file lock before review/signing; it is not a durable issuance
reservation or exactly-once signature guarantee across a crash.

Commands and actual results:

- `CARGO_NET_OFFLINE=true cargo build --locked -p tap-cli`: passed from installed
  toolchain/cached dependencies; no fetch or install. Rebuilt by final check below.
- `CARGO_NET_OFFLINE=true bash scripts/check.sh`: passed formatting, Clippy with
  warnings denied, all 15 existing tests, CLI build and unchanged legacy pipe
  smoke. Final repeat with `TMPDIR=/tmp/opencode CARGO_NET_OFFLINE=true bash
  scripts/check.sh` also passed, using approved temporary storage.
- `TMPDIR=/tmp/opencode python3 scripts/demo.py --approve-simulation`: passed
  the real Rust frame process → separate `apdu-stdio` receiver → complete persisted
  inbox → actual wallet verifier/signer/journal flow. The runner waits for receiver
  exit and every transport ACK before wallet launch; no partial-file watcher.
  Pre-send interruption still leaves zero merchant payments. The next cycle commits
  one merchant payment then deliberately loses its receipt: wallet exits nonzero
  with `SIMULATED_OUTCOME_UNKNOWN`, retains the artifact and has zero receipt rows.
  Both processes restart and replay only the saved artifact, receiving `replay: true`.
  Another duplicate replay still leaves one payment and one wallet receipt.
  Independent read-only SQLite comparisons include the full stored `Prepared`
  value (BCS, signature and digest), not merely printed digests. Final evidence:
  `<PRIVATE_VALIDATION_ROOT>/result.json`, APDU outputs and wallet/merchant logs.
- `TMPDIR=/tmp/opencode PYTHONDONTWRITEBYTECODE=1 python3
  <PRIVATE_VALIDATION_ROOT>/interactive_check.py`: passed a temporary
  real PTY approval run of that same demo, a decline with no prepared artifact,
  receipt or merchant payment, a different socket invoice rejected before journal
  creation/signing, and a concurrent Store owner rejected before readiness.
  The real Rust prompt displayed candidate digest, sender, full genesis, terms
  and fixture approval profile. Approved evidence: `<PRIVATE_VALIDATION_ROOT>`;
  declined evidence: `<PRIVATE_VALIDATION_ROOT>/failure.json`. Probe transcripts,
  boundary rejection and result: `<PRIVATE_VALIDATION_ROOT>/run-2zmede6s`.
  An earlier repeat of this private probe stopped on its own existing output
  directory (`FileExistsError`); per-run directories fixed the probe, not product
  assertions. The final invocation above passed.
- `git diff --check`: passed. A local inline Python validator passed 77 relative
  links/anchors across 11 Markdown files, whitespace/final newlines across those
  and the three changed code/script files, and demo Python syntax. It checked
  209 retained private artifact files (0600; directories 0700) and 102 owned
  PID/start identities across this worker's runs: none remained running.

The runner creates mode-0700 evidence directories and mode-0600 files under a
restrictive umask. The PTY probe checked file/directory privacy and reaped child
PID/start identities; no owned runtime remained. Failed/declined runs retain
evidence and never write a PASS result. These synthetic fixtures are not production
database backups. The intentional missing-receipt error is not a suite failure.

Stage A is now verified only for this local application simulation. Approval and
issuance reservations, dispatch/observation states, external signer recovery,
power-cut/filesystem durability, SQLite WAL-reset-fix qualification, production
custody, RF/PC/SC hardware, Omarchy-host qualification and live settlement remain
unimplemented or unqualified. No service, emulator, reader, phone, account,
production key, RPC, faucet or funds action ran. The prior independent read-only
design review does not constitute review of this code increment.

Independent read-only code review, 28 October 2026
(`ses_e7d732762ffeW11LeNKBdfwUXI`), reported no blocking or material findings
in the Stage A application diff against `f6ea6a8`. It inspected actual source/diffs,
both saved result files and retained logs; it performed no new tests, builds,
services, network actions or edits. This is separate from the earlier design
review and qualifies only the intentional local simulation—not hardware, live
settlement, atomic inbox publication, power-loss durability or exactly-once
signer recovery.

## Private virtual PC/SC Stage B (28 October 2026)

The earlier Stage B preflight lacked pcscd/vpcd/headers and namespace creation
failed `uid_map: EPERM`. This increment solved local dependencies without root,
system installation, host PC/SC activation or namespace-denial workarounds.
The [private lab](pcsc-lab.md) records the exact pins, hashes, license boundaries,
source review and opt-in reproduction. Cache: `$PCSC_CACHE`.
pcsc-lite 2.3.3 was built with Meson, private compiled IPC/config paths and USB,
libudev/libusb/systemd/polkit disabled. The vpcd patch uses only AF_UNIX reverse
mode with same-UID peers and private paths; TCP server mode fails closed.

Exact commands and observed results (run from the checkout):

```sh
bash scripts/pcsc-lab-build.sh $PCSC_CACHE
TMPDIR=/tmp/opencode CARGO_NET_OFFLINE=true \
  PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig \
  cargo build --locked -p tap-cli --features pcsc-reader
python3 scripts/pcsc-lab.py --prefix $PCSC_CACHE/prefix --approve-simulation
TMPDIR=/tmp/opencode CARGO_NET_OFFLINE=true bash scripts/check.sh
TMPDIR=/tmp/opencode CARGO_NET_OFFLINE=true python3 scripts/demo.py --approve-simulation
TMPDIR=/tmp/opencode CARGO_NET_OFFLINE=true \
  PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig \
  cargo test --locked -p tap-cli --features pcsc-reader
```

- Build helper passed repeatedly, verifies archive hashes and retains its
  generated headers/build logs/source/licenses. Initial hand-build Ninja failed
  because its Meson subprocess lacked the local `PYTHONPATH`; exporting it fixed
  dependency setup. pcsc-lite's stale Autotools declaration was not used.
- The first offline optional-feature build failed only because locked `pcsc`
  2.9.0 and `pcsc-sys` 1.3.0 were absent. The same build without
  `CARGO_NET_OFFLINE=true` downloaded just those two locked crates and passed
  (5.44 seconds). Subsequent feature builds passed offline. No lock/toolchain edits.
- Full default check passed twice: formatting, Clippy `-D warnings`, all 15
  existing tests, CLI build and original pipe smoke. Optional feature test passed
  (one existing tap-cli test); no assertion was changed or weakened.
- Stage A acceptance remains passing at `<PRIVATE_VALIDATION_ROOT>/result.json`.
  Only optional delivery/result-path arguments were added to reuse its exact
  checkout assertions, rather than copy wallet/cryptographic/recovery logic.
- Real virtual PC/SC passed at `<PRIVATE_VALIDATION_ROOT>/result.json`,
  then after the negative probe passed twice consecutively at
  `<PRIVATE_VALIDATION_ROOT>/result.json` and
  `<PRIVATE_VALIDATION_ROOT>/result.json`. Final repeat:
  `<PRIVATE_VALIDATION_ROOT>/result.json`, **0.651 seconds**. Result includes measured
  elapsed seconds (under one second per cached automated application run), T=1,
  ATR `3b800181`, exact VICC APDU sequence, exact received invoice, real Rust
  wallet consumption and identical BCS/signature/digest recovery. One merchant
  payment and wallet receipt association remain; `chain_submission: false`.
- Native `SCardReconnect(SCARD_RESET_CARD)` discarded a partial assembly:
  post-reset COMMIT returned `6a80`. After real checkout, resetting again left
  the complete ready inbox and approved artifact unchanged. `vicc.jsonl`,
  `reset.json`, real pcscd APDU logs and subprocess outputs retain the evidence.
- Intentional failure: after the default check rebuilt the non-PCSC binary,
  the same lab command exited 1 with the explicit feature-build error. At
  `<PRIVATE_VALIDATION_ROOT>`, `failure.json` exists, `result.json` does not,
  and cleanup reports no owned processes / no private daemon socket. Rebuilding
  the feature and rerunning passed without deleting runtime files. An earlier
  such failure at `ltap-pcsc-ubn_u4zo` exposed upstream pcscd's forced 0755 run
  directory; the still-0700 enclosing prefix prevented exposure. Restoring 0700
  before clients and on cleanup fixed repeatability without patching pcsc-lite.
- `bash -n scripts/pcsc-lab-build.sh`, inline Python AST parsing of both scripts,
  `git diff --check` and a local Markdown validator passed (61 relative links/
  anchors across seven relevant files). Final read-only artifact audit checked
  248 files/directories and 98 owned PID/start identities across this worker's
  eight evidence directories: private modes, no remaining owned process. The
  entire dependency cache's files/directories also passed private-mode checks.
  Python bytecode emission for the imported demo is disabled; the one generated
  cache file/directory from the first trial was removed, not any user source.

No wildcard or loopback TCP listener was started. All captured owned daemon
socket inodes resolved to AF_UNIX; its configured reader inventory was exactly
`Linux Tap private VPCD 00 00`. Each run retained its private configs, source/build
hashes, PID/start identities, logs and cleanup. No owned runtime remained after
the successful or intentional-failure runs. Final `result.json` is written only
after cleanup verification; `application-result.json` alone is not a Stage B PASS.

The five-step outcome is: retain the user-required real PC/SC/private boundary;
delete VM/TCP/full VICC OS/AOSP/new-service requirements; reuse pcscd/vpcd and the
actual receiver/checkout; obtain sub-second cached feedback; only then retain the
explicit opt-in, rerunnable helper. Default checks have no new lab launch/download.
This qualifies virtual PC/SC only. RF/HCE, callback hardware timing, power-cut
durability, production custody, Omarchy physical hardware and live settlement
remain unqualified.

Independent read-only review completed on 28 October 2026
(`ses_e7d4e67afffeE9pFbLkvW2fFdS`) with no blocking or material P1/P2 findings.
It inspected the actual scripts/patch, cached upstream sources/configuration and
retained evidence; verified all 10 manifest hashes, the exact native APDU
transcript and invoice/inbox equality, pcscd ATR/T=1/reset `6a80`, and one
artifact/payment/receipt using read-only in-memory SQLite DB/WAL inspection.
It also checked private permissions, absence of all 16 final recorded PID/start
identities, and the intentional missing-feature STOPPED result rather than PASS.
The reviewer reran no builds, tests, network operations or lab runtimes and made
no edits. This qualifies only private virtual PC/SC, not RF/HCE, hardware approval,
power-loss durability, custody or live chain behavior. Source inspection plus
socket snapshots is not full kernel-level proof; hashes are not signature
verification.

HEAD remained `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`; no staging, commits,
pushes, toolchain/lock changes or edits to the prior Rust product changes.

## Explicit testnet preparation checkpoint — 28 October 2026

Implemented on the existing dirty `f6ea6a8` tree without staging or committing.
The explicit Circle testnet profile now reaches native merchant invoice creation,
APDU-inbox verification with an independent merchant pin, exact construction,
software signing and trusted-node receipt predicates. Legacy default policy,
draft-v0 wire bytes and `Prepared` serialization are unchanged. A reusable
`qualify_candidate` checks actual frozen bytes/epoch, rather than constructing
nonce-zero diagnostic terms. Its live path was compiled, **not run** here.

New additive bounded `testnet_*_v1` tables retain the exact candidate and trust
association, unique merchant/chain/invoice issuance identity, immutable initial
approval/reservation, append-only recovery approvals and first artifact. The
existing cooperative owner lock remains. Reservation commits before key loading
or signing; an incomplete reservation fails closed unless the user explicitly
approves deterministic Ed25519 recovery of the saved candidate with the same key.
Existing artifacts are verified/exported without key access or new signatures.
No dispatch-intent barrier, execution RPC or paid-state migration was added.

Pinned Mysten source `1313d5112464ecc25d7407f42442adea08589fc5` was inspected:
`sui-crypto/src/ed25519.rs::from_base64` officially decodes the legacy
`flag || private_key` Sui CLI entry, rejecting wrong schemes/lengths;
`SuiSigner::sign_transaction` signs the SDK transaction signing digest. The
dedicated importer accepts only explicit absolute owned regular mode-0600 files
with canonical owned mode-0700 parents, rejects links and selects the independently
expected Ed25519 sender. No dependency or hand-written seed/crypto format was
added. JSON strings and SDK decoder temporaries have no guaranteed memory erasure;
this is local dedicated testnet software, not production custody or encryption.
Same-user malicious filesystem replacement is outside this cooperative profile.

Checks run from the repository root:

- `cargo fmt --all`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo check --workspace --locked`:
  passed during implementation.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  formatting, Clippy with warnings denied, all 15 existing tests, doc tests,
  native build and existing pipe smoke (`PASS`, one artifact/payment,
  `chain_submission: false`). No existing assertions were changed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo check --workspace --features pcsc-reader --locked`:
  passed using the existing private cache. No transport hooks were changed, so
  no new PC/SC daemon/lab run was needed or claimed.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py`: final run
  passed with the actual native CLI and a real PTY, without network access.
  Retained result/transcript/journal/fixtures:
  `<PRIVATE_VALIDATION_ROOT>/run-1793195577544120686/`.
- `git diff --check`: passed. HEAD remains
  `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`; manifests/lock/toolchain unchanged.

The temporary acceptance helper was compiled against the already-built official
SDK and project libraries (no new dependency or permanent test framework):

```sh
umask 077
rustc --edition=2024 <PRIVATE_VALIDATION_ROOT>/fixture.rs \
  -L dependency=target/debug/deps \
  --extern sui_crypto=target/debug/deps/libsui_crypto-79c23eb651d092ae.rlib \
  --extern tap_core=target/debug/deps/libtap_core-5596095dabfe8be0.rlib \
  --extern tap_sui=target/debug/deps/libtap_sui-e0f64fc736e4db1d.rlib \
  --extern serde_json=target/debug/deps/libserde_json-3c3759ef10b935ac.rlib \
  -o <PRIVATE_VALIDATION_ROOT>/fixture
```

It generated **known public** merchant `[7;32]` and sender `[9;32]` keystore
fixtures through SDK `to_base64`, never fresh private secrets. Native merchant
invoice → actual APDU frame/receiver processes preserved the exact envelope.
The first typed `APPROVE TESTNET` committed a reservation before an intentional
missing-key-file failure. Normal restart correctly reported unresolved issuance.
Typed `RECOVER TESTNET` signed the saved candidate; the helper independently
verified merchant pin, explicit template and SDK signature, reproduced the exact
deterministic artifact from that candidate, and confirmed legacy policy rejection.
Reopening with a deliberately nonexistent key path exported identical BCS,
signature and digest without key access. A conflicting export was rejected.
The offline reservation has absent qualification, not invented successful RPCs.
An initial harness assertion incorrectly compared the entire APDU response to
`9000`; it was corrected to check the documented status suffix (SELECT/COMMIT
include data). This was a harness issue, not a product or protocol change.

Final fixture directory is mode 0700 and all its files mode 0600. Both PTY
PID/start identities (`1880216/84645918`, `1880256/84645925`) were recorded and
verified absent after cleanup; synchronous helper/receiver processes exited too.
No owned runtime remains. Earlier public fixture attempts are retained in the
same private parent; no existing user key locations were inspected.

**Limits:** this is `PREPARED_NOT_PAID`, `chain_submission=false`,
`ready_to_submit=false`. No RPC, faucet, live signature, funding, conversion,
execution, live receipt or settlement was performed. Fresh controlled accounts,
usable address balances, dispatch/reconciliation, fresh-receiver no-reuse custody
and independent merchant credit remain pending after independent review of this
checkpoint. For later reconciliation, query the saved digest before fresh
eligibility; execution may already have consumed the balance or crossed epochs.
Hardware is deferred. The user's real end-to-end testnet settlement goal is not
yet met; offline signatures and APDU acknowledgements do not close it.

### Independent review P2 and record-integrity fix

Independent review `ses_e7d355137ffeaXiT10YXbCLGve` found a real P2 in this
checkpoint: saved reservation loading checked profile/digest/action but treated
qualification as arbitrary JSON, and never loaded recovery approvals. Thus
contradictory qualification metadata or malformed recovery records did not stop
saved-artifact export. This was not an execution bypass: no execution path exists.
The earlier successful acceptance did not cover this integrity requirement.

The fix replaces arbitrary qualification JSON with a strict typed summary and
validates it against the **saved** candidate's mode, genesis, epoch, digest and
asset, with successful qualification, a known positive protocol version,
transaction checks enabled, gas selection disabled and no submission. Offline
fixtures require absent qualification. Initial and all associated recovery
approvals must also match their expected profile, digest, action and historical
invoice lifetime. Recovery rows are streamed and validated before any saved
artifact export or further recovery. New approvals use the same validator before
persistence. This is historical consistency, not a current epoch/expiry or
eligibility gate on an existing artifact. Schema and valid serialized summaries
remain unchanged; malformed/contradictory records now fail closed.

Verification after this fix:

- `cargo fmt --all`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  formatting, Clippy, all 15 existing tests, doc tests, build and pipe smoke.
- `python3 <PRIVATE_VALIDATION_ROOT>/review-integrity.py`:
  passed. It first reran the existing real-process CLI/APDU/PTY approval and
  saved-candidate recovery acceptance, then changed only its own public-fixture
  journal records. Four corrupted states were rejected before creating a new
  export: a qualification summary in offline mode, malformed recovery JSON,
  a reservation summary with the wrong saved epoch, and a recovery summary
  with a conflicting digest. Restored valid records exported identical bytes
  without reading a key or signing again.
- For the historical live-summary consistency check only, the fixture journal
  used explicitly synthetic metadata and an `.invalid` endpoint on an already
  saved artifact. Consistent historical summaries reopened; contradictory ones
  failed. This made no RPC and is **not** evidence of real qualification or
  settlement. Synthetic records were restored to the original offline records.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo check --workspace --features pcsc-reader --locked`:
  passed; no PC/SC runtime was launched.
- `git diff --check`: passed.

Result, exact command/exit/error transcripts, PTY PID/start identities and restored
public fixtures are retained in
`<PRIVATE_VALIDATION_ROOT>/run-1793196026104348701/`, including
`review-integrity-result.json` and `review-integrity-transcript.json`.
The harness waited/reaped all children. A final check verified all nine recorded
PTY/replay PID/start identities absent and directory/file modes 0700/0600;
no owned runtime remains. No network, existing keys, faucets,
fresh secrets, live signatures, funds or VCS mutations were used. This fix does
not close independent re-review, live eligibility, dispatch, merchant settlement
or the user's end-to-end testnet goal.

## Controlled dispatch and independent merchant commands — 28 October 2026

Preparation P2 re-review `ses_e7d355137ffeaXiT10YXbCLGve` reported **RESOLVED**,
with no remaining finding. The subsequent dispatch slice was reviewed and its
self-transfer P2 independently resolved on 28 October 2026; see the focused
review record below. This does not qualify live execution or settlement.

Implemented runnable `testnet register`, `submit`, `reconcile`, `accept` and
`confirm`, keeping the existing three crates and legacy simulator behavior.
Submission loads the approved immutable artifact and validates all preparation,
dispatch approval, attempt and observation records before using saved state.
Offline/default fixtures cannot submit. First-send consent is a real terminal
`TESTNET SUBMIT`; replay additionally requires `--replay-saved` and typed
`TESTNET REPLAY SAVED`. Neither path loads a key or constructs a new transaction.
The first intent plus first attempt commit in one transaction before polling the
SDK execution helper. Further attempts append, never replace that barrier.
Cancellation after the barrier is conservatively possibly sent.

Pinned SDK source was inspected for `ExecuteTransactionRequest::new`, official
transaction/signature conversion and
`Client::execute_transaction_and_wait_for_checkpoint`. Its helper wait timeout
does not cover the preceding subscription/execute phase; an outer 60-second
timeout now bounds the entire operation. All helper/RPC errors remain unknown,
retaining raw execution information when the SDK supplies it. Queries occur
before current eligibility on resume. Exact checkpoint failure and success are
distinct predicates; no NotFound, unknown or failure creates a fresh transfer.
The common identity predicate was extracted without weakening the existing
success-only verifier. The stricter new observation path additionally checks the
inner transaction digest and actual returned stored signature.

A separate merchant namespace/receiver registration, typed node observations and
unique chain/digest credit/immutable receipt are persisted transactionally.
SQL chain/digest/receiver columns and their typed records are checked together
on load. The merchant queries its separately chosen fixed-genesis node without
expiry, current balance or epoch gates; late successful observation is retained
with a late fact, not refunded. Wallet receipt association requires another
independent query of the explicitly chosen merchant endpoint and its own durable
observation. Files are not authenticated remote business acknowledgements.

Directly necessary endpoint hardening rejects URL userinfo/control characters:
the previous string-prefix loopback exception could otherwise accept an external
HTTP authority such as `http://localhost:80@external-fixture.invalid`. This needs
no dependency and prevents storing credential-bearing userinfo in new trust
records. There are no new services, plugins, key providers or dependencies.

Checks run from the repository root:

- `cargo fmt --all`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo check --workspace --locked`:
  passed after fixing an initial inferred async-result error annotation.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: final run
  passed formatting, Clippy with warnings denied, all 15 existing tests, doc
  tests, native build and pipe smoke. An initial Clippy pass found six needless
  borrows introduced while sharing approval validation; those were fixed before
  the successful run. No assertions were weakened and no permanent tests added.
  Existing caches were reused; no clean/release/dependency rebuild was requested.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo check --workspace --features pcsc-reader --locked`:
  passed. No reader hooks changed and no PC/SC runtime was launched.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py`: passed. It
  reused the previous real APDU/CLI/PTY recovery and P2 corruption acceptance,
  then exercised real new CLI commands: offline-fixture submit rejected before
  RPC/intent; damaged artifact rejected; explicit PTY decline wrote no intent,
  attempt or observation; malformed attempt failed before RPC; receiver reuse
  across invoices failed; unregistered invoice was not credited; loopback-looking
  userinfo was rejected. Tests never supplied affirmative submission consent.
- `git diff --check`: passed; HEAD remains
  `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`, index empty and dependency manifests,
  lockfile and toolchain unchanged. Prior dirty work was preserved.

The temporary helper inspected the **actual production request constructor**:
SDK transaction BCS and the one stored signature round-tripped exactly. A small
synthetic execution fixture exercised success/failure predicate distinction and
inner-digest corruption rejection. This is an offline boundary check, **not** a
mocked successful node qualification, execution or merchant settlement. Known
public seeds `[11;32]`/`[12;32]` were used only for negative-path fixture records
and offline signatures, never funded or submitted. Synthetic historical live-mode
metadata was used only to reach terminal decline/invalid-record guards, never as
evidence of eligibility. The original offline/default fixtures remained rejected.

Exact helper compilation (already-built pinned libraries, no new dependencies):

```sh
umask 077
rustc --edition=2024 <PRIVATE_VALIDATION_ROOT>/fixture.rs \
  -L dependency=target/debug/deps \
  --extern sui_crypto=target/debug/deps/libsui_crypto-79c23eb651d092ae.rlib \
  --extern sui_rpc=target/debug/deps/libsui_rpc-2e7c8609eecf326f.rlib \
  --extern sui_sdk_types=target/debug/deps/libsui_sdk_types-2b3cb85b9f43adc3.rlib \
  --extern tap_core=target/debug/deps/libtap_core-5596095dabfe8be0.rlib \
  --extern tap_sui=target/debug/deps/libtap_sui-e0f64fc736e4db1d.rlib \
  --extern serde_json=target/debug/deps/libserde_json-3c3759ef10b935ac.rlib \
  --extern bcs=target/debug/deps/libbcs-7115e84218b7d9c0.rlib \
  -o <PRIVATE_VALIDATION_ROOT>/fixture
```

Final result, complete commands/errors/PTY transcript and fixture journals:
`<PRIVATE_VALIDATION_ROOT>/run-1793198280851620004/`.
Reused preparation/recovery/P2 evidence:
`<PRIVATE_VALIDATION_ROOT>/run-1793198280248290376/`.
The scenario waited/reaped its children. A final check verified all 23 recorded
owned PID/start identities absent, both artifact directories mode 0700 and all
their files mode 0600. No owned runtime remains.

**Unqualified:** no RPC, real/user-controlled key access, fresh secret generation,
faucet, live signing or broadcast occurred. No successful live dispatch, actual
execution timeout/reconciliation, merchant checkpoint credit or wallet receipt
association was tested. No live end-to-end PC/SC orchestrator, production custody,
power-loss durability, authenticated remote business ACK, HD keys, UI, refunds,
packaging or hardware work is claimed. Review is now complete for the bounded
dispatch scope; human-private provisioning, faucet funding and usable fresh
testnet address balances remain prerequisites for live qualification. The
[runnable manual flow](testnet-qualification.md#controlled-submission-and-independent-merchant-flow)
uses actual native APDU ingress and commands, not a claimed funded result.

### Dispatch review P2 — reject merchant self-transfers

Independent dispatch review `ses_e7d355137ffeaXiT10YXbCLGve` found one P2:
`Identity::new` verified signature/template but allowed the transaction sender
to equal the invoice receiver. A checkpoint-successful self-transfer could thus
be mistaken for merchant payment despite no net payment. The earlier offline
acceptance did not cover this merchant-payment requirement. No other material
finding was reported. Focused independent re-review completed on 28 October 2026
in `ses_e7d355137ffeaXiT10YXbCLGve`: the self-transfer P2 is **RESOLVED**, with no
remaining finding in the reviewed dispatch scope. The qualification-record P2
was previously independently resolved too. Read-only source, pinned SDK and
private offline evidence confirmed the shared guard covers submit, confirm and
merchant accept before RPC/dispatch/credit; historical non-self reconciliation
is unchanged. The reviewer performed no actual RPC execution, timeout exercise
or settlement. Local artifact files are still not authenticated remote merchant
ACKs, and no live PC/SC orchestration proof follows from this review.

The smallest fix adds `tx.sender != Address::new(invoice.receiver)` to the shared
testnet settlement identity constructor, after signature/template verification
and before any RPC, dispatch or credit. Wallet dispatch/confirmation and merchant
acceptance share that guard. Generic protocol/template defaults, wire fixtures,
preparation and dependencies are unchanged; this does not ban self-transfers in
the lower-level transaction API.

Validation after the fix:

- `cargo fmt --all`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  formatting, Clippy, all 15 existing tests, doc tests, build and pipe smoke.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo check --workspace --features pcsc-reader --locked`:
  passed; no PC/SC runtime was started.
- `python3 <PRIVATE_VALIDATION_ROOT>/review-self-transfer.py`:
  passed. It first reran the original real CLI/APDU/PTY acceptance and prior
  record-integrity/dispatch guards. Then native invoice creation and the existing
  SDK fixture helper produced a valid signed self-transfer using known public
  seeds `[11;32]`/`[12;32]` only. Receiver registration succeeded, but both actual
  `testnet accept` and `testnet submit` rejected with
  `self-transfer cannot settle a merchant invoice`. Both journals retained zero
  dispatches, attempts, wallet/merchant observations, credits and confirmations.
  Synthetic historical wallet metadata was only a local negative-path fixture,
  not RPC eligibility evidence. No affirmative submission consent was supplied.
- `git diff --check`: passed. HEAD remains
  `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`; no index, manifest, lockfile or
  toolchain change.

Self-transfer result/commands/errors and fixture journals:
`<PRIVATE_VALIDATION_ROOT>/self-transfer-1793199192736747711/`.
Original dispatch acceptance rerun:
`<PRIVATE_VALIDATION_ROOT>/run-1793199192078020118/`.
Preparation/P2 acceptance rerun:
`<PRIVATE_VALIDATION_ROOT>/run-1793199191463040667/`.
All children were waited/reaped; a final check verified all 30 recorded PID/start
identities absent, directories mode 0700 and files mode 0600. No owned runtime
remains. No RPC, real keys, faucets, live signatures or broadcasts were used.
This is local rejection evidence only: live settlement, actual node behavior and
the full controlled testnet flow remain unqualified pending human-private
provisioning, faucet funding and an actual controlled live trial. The user's
real end-to-end settlement goal remains unmet.

## Human-gated disposable testnet setup — 28 October 2026

The user selected fresh disposable wallets, with private provisioning and faucet
CAPTCHA performed by the human. `command -v sui` returned no executable path.
No wallet/CLI was installed and no default wallet locations were searched. A
bounded native `testnet init --directory ABSOLUTE_NEW_DIRECTORY` now reuses the
existing pinned SDK instead of asking the operator to invent a key workflow.
Focused init-only independent review completed on 28 October 2026 in
`ses_e7d355137ffeaXiT10YXbCLGve`, with no blocking/material P1/P2 finding.
Read-only source/pinned-SDK review verified terminal consent and checked prctl
dump protection before entropy, SDK Ed25519 `generate(OsRng)`/legacy encoding,
synced private files before atomic no-replace public-only READY publication,
and private-file retention/own-manifest cleanup on failure. This is **not** a
production security audit. The reviewer executed no positive generation,
publication or cleanup, and no fresh wallets had been generated at that review point.

Pinned source `1313d5112464ecc25d7407f42442adea08589fc5` was inspected:
`Ed25519PrivateKey::generate` requires `rand_core::RngCore + CryptoRng` and draws
32 bytes from the supplied RNG. Existing `rand` 0.8.8 and the SDK both use
`rand_core` 0.6; `OsRng` implements `CryptoRng` and delegates to OS `getrandom`,
not fixture seeds or a configurable pseudo-RNG. Official `to_base64` emits the
legacy Sui CLI `flag || private_key` encoding already consumed by SDK
`from_base64` in the existing importer. No hand-written crypto/seed encoding,
new dependency, generic keystore framework, large CLI install, encryption,
backup, HD derivation, funding or faucet automation was added.

The command accepts only a new absolute child of an existing owned canonical
mode-0700 parent, rejects roots/traversal/links/overwrite and nonterminal input,
and requires exact typed `CREATE DISPOSABLE TESTNET WALLETS` before entropy use.
Before any key material, Linux `prctl(PR_SET_DUMPABLE, 0)` must succeed and
`PR_GET_DUMPABLE` must report zero. The constants and C ABI were checked against
installed Linux headers. This is process-local dump protection, not a service or
host configuration change, privileged-adversary isolation or guaranteed erasure.
Three SDK/OS-entropy Ed25519 keys have dedicated owner-only mode-0600 files in
the new mode-0700 directory: sender, merchant invoice signer and receiver. Each
private file is synced, then directory entries synced, before a public-only
pending manifest is synced and published atomically using Linux
`renameat2(RENAME_NOREPLACE)`. The corresponding declaration/flag were inspected
in the installed `stdio.h`. READY publishes public addresses, merchant pin,
paths and fixed testnet constants, never private keys or RPC credentials.

Partial errors retain private files and report STOPPED. After publication,
ordinary errors attempt to withdraw only the manifest inode created by this
command. Filesystem failure can prevent withdrawal; an explicit STOPPED warning
then forbids using the directory even if a READY file remains. Require successful
command completion, never mere file presence. No private/other data is deleted,
and no existing directory is reused. JSON/SDK secret temporaries are not
guaranteed erased or protected from swap. This is unencrypted disposable testnet
software, not production custody, secure backup or power-loss qualification.

Checks actually run:

- `cargo fmt --all`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo check --workspace --locked`:
  passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  formatting, Clippy, all 15 existing tests, doc tests, native build and pipe smoke.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo check --workspace --features pcsc-reader --locked`:
  passed. No PC/SC runtime launched.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py`: passed. The real
  CLI refused an existing public-fixture directory/link, root destination and
  nonterminal new-directory request. A real PTY sent only `DECLINE`: no directory,
  keystore or READY was created. The existing fixture marker stayed byte-identical.
  The same harness reran original APDU/approval/recovery, P2 record-integrity,
  dispatch/receiver and self-transfer acceptance, all passing. These use known
  **public** fixture seeds only. Official legacy encoding and actual native
  importer decoding/public-sender matching were exercised, not mocked.
- The exact documented READY formatter was run separately against
  `PUBLIC-MANIFEST-EXAMPLE.json`, containing only known public example metadata
  and nonexistent example key paths. All seven shell-quoted public assignments
  matched, including spaces/apostrophes in paths; it opened no keystore. Exact
  command/PID/output is retained in `doc-formatter-check.json`.
- `git diff --check`, relative Markdown links/anchors, whitespace and final
  newlines: checked separately for the changed documentation.

Init guard result/transcript/public formatter example:
`<PRIVATE_VALIDATION_ROOT>/guards-1793201620467739471/`.
Reused public-fixture acceptance paths are listed in its `result.json`.
Its deliberate symlink points only to its own public KEEP fixture. No real
wallet keys were created or accessed, and no positive init confirmation was
sent. Final checks passed for 27 relative links/anchors plus whitespace/final
newlines in the three changed documents, private artifact directory/regular-file
modes 0700/0600, absence of all 37 recorded owned PID/start identities, and no
keystore/READY in the init guard directory. No owned runtime remains. HEAD is
unchanged at `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`; index, dependency
manifests, lockfile and toolchain are unchanged. **Unexecuted:** fresh entropy/key generation, dump-protection syscalls,
successful init publication and post-confirmation failure cleanup. Those have
source/SDK/header inspection and compilation evidence only. No RPC, faucet,
live signing or broadcast occurred. The human still needs privately provisioned
controlled wallets and eligible address-balance funding. The minimum next gate
is a human-private **unfunded** successful init: verify exit status 0/no STOPPED,
ownership and directory/file modes 0700/0600 with regular files (no symlinks;
link count 1), and
locally decode each keystore using the official legacy decoder to match READY's
public addresses and merchant pin before any funding. Only public match/pass
information may be shared; no agent key access or raw entries. This positive
qualification remains pending despite completed source review. Coin-object
conversion remains a separate blocker; no blanket SUI requirement is added.
Real end-to-end settlement, authenticated remote merchant ACKs and live PC/SC
orchestration remain unproven; hardware is deferred.

## Human-private read-only setup key matching — 28 October 2026

The human subsequently reported successful init and supplied public READY data
only. Metadata-only/public qualification worker
`ses_e7ccea915ffeYUiCnAyPFmRDnJ` reported owner/mode/regular-file/nonlink checks
and public fields matching. Its separate read-only testnet qualification reported
the pinned genesis, epoch 1266/protocol 139, exact Circle gasless type and six
decimals, but zero available address balance for 10000 base units (exit 1).
These are provided context, not operations rerun in this implementation task.
They do not prove private-key matches, funding, eligibility or settlement.
The agent did **not** open the real user's directory, READY or any keystore, and
did not execute verification against it. No real user path/address is recorded here.

New `testnet verify-setup --directory ABSOLUTE_EXISTING_SETUP_DIRECTORY` closes
the need for a human to hand-write cryptographic verification code. Independent
review completed on 28 October 2026 in `ses_e7d355137ffeaXiT10YXbCLGve`, with no
blocking/material P1/P2 finding. Source plus public-fixture evidence covered key
matching, tamper rejection, immutability and no key payload in output. The
reviewer did not access the user's wallet or read its keys. The actual user-wallet
key-match check remains human-private and pending; completed review is not proof
of freshness, funding, eligibility, settlement or production custody.

The bounded command reuses the init manifest types and constants, now strict
deserializable public types with unchanged JSON serialization. It checks exact
profile/status/genesis/type/amount and false submission/production-custody flags,
canonical owned mode-0700 directory and owned regular mode-0600 singly linked
manifest/key files. Only fixed READY/sender/merchant/receiver names are opened;
all manifest key paths must match those role paths before any key decoding.
File handles/ownership are checked before dump protection because Linux can
change `/proc/self` ownership when dumpability is disabled. These initial opens
read metadata only. The shared checked `prctl` helper runs before reading any
manifest/key bytes. Init retains the same protection-before-entropy ordering.

The existing sanitized official SDK loader is shared through an already-checked
file-handle helper: legacy callers keep their 32-entry selection limit; verify
requires exactly one Ed25519 entry per role. SDK-derived addresses and public
keys must match READY, roles must be distinct and the four existing project/lab
fixture vectors (7/9/11/12) are rejected. That blacklist cannot prove that an arbitrary key was
never public or was freshly generated. The verifier performs no entropy draw,
signing, RPC or content/mode/state writes. Only public JSON with
`KEY_MATCHES_ONLY_NOT_FUNDED_OR_ELIGIBLE` is printed; READY is never promoted to
funded/eligible. No guaranteed memory erasure, encrypted custody, power-loss
safety or authenticated remote business ACK follows from key matching.

Validation actually run, without real-user key access or network:

- `cargo fmt --all`: passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode cargo check --workspace --locked`:
  passed.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode bash scripts/check.sh`: passed
  formatting, Clippy, all 15 existing tests, doc tests, native build and pipe smoke.
- `CARGO_NET_OFFLINE=true TMPDIR=/tmp/opencode PKG_CONFIG_PATH=$PCSC_CACHE/prefix/lib/pkgconfig cargo check --workspace --features pcsc-reader --locked`:
  passed; no PC/SC runtime was launched.
- `python3 <PRIVATE_VALIDATION_ROOT>/acceptance.py`: passed.
  The real new CLI decoded all three **known public** fixture keys (seeds
  `[21;32]`, `[22;32]`, `[23;32]`; NEVER FUND) using official legacy encoding and
  produced matching public output. One contradictory READY receiver public key
  was rejected. Neither successful nor failing verification changed content,
  mode or mtime, and no encoded private payload appeared in captured output.
  This exercises the actual shared dump check and SDK decoding on public fixtures,
  not the real user's keys or fresh entropy. It also reran all prior native
  APDU/PTY recovery, record-integrity, dispatch, self-transfer and init-negative
  acceptance with public fixtures, all passing. No positive init consent was sent.
- `git diff --check`, relative documentation links/anchors, whitespace and final
  newlines are checked separately. No production dependencies or wire formats
  were changed; no permanent unit matrix or test-only product flags were added.

Exact public-fixture producer compilation, against existing pinned build outputs:

```sh
umask 077
rustc --edition=2024 <PRIVATE_VALIDATION_ROOT>/fixture.rs \
  -L dependency=target/debug/deps \
  --extern sui_crypto=target/debug/deps/libsui_crypto-79c23eb651d092ae.rlib \
  --extern sui_sdk_types=target/debug/deps/libsui_sdk_types-2b3cb85b9f43adc3.rlib \
  --extern serde_json=target/debug/deps/libserde_json-3c3759ef10b935ac.rlib \
  --extern hex=target/debug/deps/libhex-b0757a7bc7b6b2d3.rlib \
  -o <PRIVATE_VALIDATION_ROOT>/fixture
```

Result, exact CLI commands/exit/output and public fixtures:
`<PRIVATE_VALIDATION_ROOT>/public-fixtures-1793203736556136914/`.
Prior acceptance paths are listed in its `result.json`. The helper uses only
documented public constants, never RNG, private user files, network or signatures.
Final checks passed for 28 relative documentation links/anchors, whitespace and
final newlines, private artifact directory/regular-file modes 0700/0600, and
absence of all 39 recorded owned PID/start identities. No owned runtime remains.
HEAD stays `f6ea6a8af5af715dfbb52b87b5f7f3d4e59ac037`; index, dependency
manifests, lockfile and toolchain are unchanged; all prior dirty work is preserved.
With that review complete, the human must privately run the command on the
already-created directory before requesting Circle funding; do not regenerate
or disclose keystores. Share only public match/pass information. Actual private
user verification, funding/address-balance qualification, execution and settlement
remain pending. Coin-object conversion and live PC/SC orchestration remain open;
no blanket SUI requirement or production-custody claim is added.

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
