# Execution-response-loss operator lab candidate

**29 October 2026 — compiled; local offline native guards PASS. Independent source/
offline-evidence review complete; live execution-loss qualification remains open.**
Nothing here authorizes funding, new keys, signing or submission by an agent.
The existing [read-only query lab](evidence/query-response-loss-2026-10-29.json)
and its two-method proxy are unchanged; do not add Execute to that component.

## Settled contract

Lose **only ExecuteTransaction's response** at the local gateway after a complete
real upstream response, before any downstream success headers/body. Do not hide
ledger reads, delay checkpoint proof, patch the SDK or force durable Unknown.
The pinned helper's concurrent/post-error lookup may correctly recover success.

Future qualification requires all of:
- Exactly one consumed sealed-request gate and one real Execute response captured
  from the fixed official TLS testnet node; byte-identical request to the native seal.
- Actual proxy reset plus native `EXECUTION_RESPONSE_TRANSPORT_TRACE` reporting
  a received remote CANCEL transport error, not merely a dropped future.
- Matching successful checkpoint, full BCS/signature/digest and effects through
  ordinary native verification, then unique merchant credit/wallet association.
- Either in-helper verified recovery (normal durable Dispatch observation) or
  actual durable Unknown followed by fresh keyless saved-digest reconciliation.

`NotStarted`, `HeadersReceived`, or `CancelledBeforeTransportResult` cannot prove
this fault was received. If SDK recovery cancels Execute before the reset reaches
its transport future, classify **FAULT_NOT_REACHED / INCONCLUSIVE**, even if a
payment settled. Never repeat a payment to repair a fault-test result.
Source Dispatch denotes the SDK helper result, not necessarily raw Execute data.
The proxy deliberately emits **no PASS/payment-success verdict** by itself.

## Source implementation and boundaries

- `testnet execution-seal` loads existing approved records with native wallet,
  invoice/pin, template, signature, identity/history and registration validation.
  It rejects existing intent/terminal state, wrong independent receiver, non-0.01
  amount, nonliteral loopback endpoint or nonofficial merchant endpoint. It reads
  no keystore, makes no RPC, and builds no new transaction/nonce/signature.
- The same native constructors supply actual dispatch and official prost payloads.
  The pinned execution client accepts Zstd responses but does not compress requests;
  the gateway adds only the standard uncompressed unary gRPC length header.
  Native seal/framing guard compatibility passed offline; actual SDK transport
  interoperability is **not runtime-qualified yet**.
- `testnet check-lab-simulation` decodes official protobuf/types, verifies native
  Circle invoice/template/sender/amount policy and compares the entire canonical
  checked simulation request, including re-encoding. Extra fields, changed gas
  selection and disabled checks are rejected; it never signs or calls a node.
- `execution-response-loss-proxy.mjs` defaults to **UNARMED_NO_EXECUTE**. It accepts
  only fixed preparation/ledger reads, checked simulation and checkpoint subscription.
  It verifies TLS to `https://fullnode.testnet.sui.io:443`, excludes credentials,
  bounds bytes/calls/time and passes normal reads/streams with backpressure.
- Human arming requires a native seal, an unconsumed gate, a real terminal and
  the exact arming phrase. Native `TESTNET SUBMIT` consent remains independently
  mandatory. Compressed/multiple/changed Execute frames are rejected before any
  upstream connection. No other mutation path is supported.
- A create-new gate, file sync and parent sync precede forwarding. The gate stays
  consumed on every failure and across restart; it grants at most one forwarding
  opportunity, not proof of broadcast/settlement or hostile-host/power-cut safety.
- Before consuming that gate, `testnet check-execution-seal` independently reads
  the bound native records and requires a validated first Submit intent, exactly
  one matching attempt, no terminal result or previous Dispatch, and identical
  canonical payloads/registration. The seal alone cannot authorize forwarding.
  This predicate runs asynchronously so ledger/subscription recovery remains free
  to win. Cancellation before forwarding leaves the trial inconclusive.
- The adjacent Store change adds a read-only SQLite snapshot without schema
  initialization or the submit process's cooperative owner lock. It is needed
  because submit retains that lock while the gateway verifies its durable barrier.
  SQLite may use normal WAL shared-memory coordination; this is not a promise of
  zero filesystem activity. Normal writable Store ownership is unchanged.

Private files/directories must be owned 0600/0700, canonical and nonsymlinked.
This is cooperative same-UID operator tooling, not loopback authentication,
encrypted custody, a security audit or a compromised-host defense.

## Future human-only workflow — do not run now

1. After review and separate approval, allocate a genuinely fresh receiver; retain
   the original sender, merchant key and authoritative ledger with both prior credits.
   Existing receivers are USED. No empty merchant ledger or replacement operation.
2. Use a fresh **durable** private gateway operation directory, not temporary funds
   custody. Start unarmed with the public preparation inbox/pin/sender context.
   Port 0 allocates once and persists `endpoint.json`; use that exact endpoint
   for candidate preparation. Native checked-simulation validation precedes forwarding.
3. Human private preparation/signing uses the normal APDU inbox/policy/approval
   flow. Export `execution-seal` afterward with independent expected receiver and
   matching registration; the private seal is binding metadata, not new approval
   or global receiver-freshness certification.
4. Stop only the owned unarmed gateway; restart the same directory/port with the
   seal and explicit arming option. Never retarget candidate/intent endorsements.
   Run normal native submit with `--trace-execution-response` in a separate human
   terminal; type the real native consent only if its exact terms are approved.
5. Preserve the trace, native transport marker, response body and wallet facts.
   If needed, reconcile the same saved digest through the same gateway endpoint;
   after restart it may be unarmed, and consumed gates still prohibit all Execute.
   Continue ordinary merchant verification against its original official endpoint.
   Only a future successful approved third payment would add one ledger credit.
6. If the port is occupied or a binding/gate differs, STOP. Do not choose another
   endpoint, delete/reset markers or start a new directory to replay. After possible
   effect, retain state and reconcile; stopping a process does not prove no payment.

## Granted-window offline validation

One actual local HTTP/2/native guard scenario passed on 29 October, using only
reconstructed records from the retained public verifier snapshot (SHA256
`976aed4bb4bee0339c475d4c31ff360d107f35b12926505dee02c054f49ae6b9`).
The historical signed artifact and registration stayed unchanged; endpoint
endorsements were explicitly synthetic, and intent/attempt rows were staged after
seal export. This was not a fresh approval, new signature or live-custody journal.
The previously used receiver was only a public fixture, not eligible for another trial.

Real native `execution-seal` succeeded; the first-dispatch predicate rejected the
pre-intent state, then accepted the reconstructed historical first intent/attempt
while a cooperative exclusive journal owner lock was held. The checked simulation
predicate also passed. Real gateway requests then exercised unarmed refusal,
altered-frame refusal, exact offline gate consumption, second-forward refusal and
same-endpoint restart/second-forward refusal. Every reply was HTTP 403/zero bytes;
no fake-success chain server or mocked validator was used. Native rows were identical
before/after; the consumed gate bytes survived both second requests and restart.

`--offline-guard-lab` returns before the upstream branch for every method, including
an exercised allowlisted GetServiceInfo read. A separate fail-closed harness
interlock rejected any proxy `http2.connect` call; it recorded **zero attempts**.
The trace has no upstream-start/response events. All three proxy lifetimes and its
native predicate child exited/reaped, with PID/start identities checked retired.
This qualifies local guards, **not Execute forwarding, TLS, response reset delivery,
the transport observer, live SDK recovery, or host/power/storage durability**.

Commands/results and retained 0700/0600 artifacts are recorded in
[validation](validation.md#execution-response-loss-candidate-offline-native-guards-validated).
Existing format/Clippy/15-test/build/smoke checks and PC/SC 15-test/build checks
passed. Initial Clippy exposed a cancellation-guard capture problem, fixed by
consuming the complete Drop guard on completion; no assertion or SDK change.
Final PC/SC-enabled binary: **162037640 bytes**, SHA256
`bb8d2808223e52d315b7cb41552163d43b22b691ce3dc04c8c44157aaa349aa1`.

All paid forwarding remains **UNEXERCISED** until a separately consented human
trial after review. Host/power/filesystem durability and production custody remain
open; a trial outcome must never authorize automatic resubmission or new funds.
