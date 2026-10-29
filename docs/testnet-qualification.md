# Testnet qualification and fresh-wallet prerequisites

## First emulated real testnet checkout verified (29 October 2026)

**Complete for one checkout:** human invoice/register → native private PC/SC
delivery → approved saved SDK signature → submit → merchant accept → wallet
confirm. Independent verifier `ses_e7ccea915ffeYUiCnAyPFmRDnJ` passed at
00:39 UTC: **0.01 Circle testnet USDC**, digest
`7LemioWwAmjRYA9gqsZZBCnCnUgpESH3qfg7mJzJCJxz`, checkpoint **400344020**,
epoch 1267. [Durable sanitized public excerpt](evidence/emulated-testnet-checkout-2026-10-29.json).

One fresh keyless reconciliation checked exact saved bytes/signature and typed
sender SPLIT/receiver MERGE effects. Duplicate accept/confirm returned existing
validated credit/association **without new node queries**. Original artifacts
and replayed receipt stayed identical; only one wallet observation appended.
No additional transfer was made by verification. Both roles trust the same
official fullnode, not independent operators or a light-client proof.

**The recorded receiver is now USED. Preserve its receiving custody and private
journals; do not rerun the historical commands below or create another ledger
to reset receiver reuse policy.** Setup, conversion and checkout are complete;
there is no new payment instruction here. The separately approved application-
boundary recovery trial is now verified below; wire-level unknown outcomes and
host/power failures remain unqualified. Hardware remains deferred. This milestone
does not prove RF, production custody, power-cut behavior, a security audit or
authenticated remote business ACKs; earlier simulated faults remain separate.

## Opt-in application-boundary recovery (review completed)

**Human-run PASS independently verified 29 October 2026 at 09:47:53 UTC**
(`ses_e7ccea915ffeYUiCnAyPFmRDnJ`), for one new **0.01 USDC** payment:
digest `52wFhV654LoLhXycxB5nxn3B2ddM4siC7Mm72opYt9sW`, checkpoint
**400484678**, epoch 1267. [Public recovery excerpt](evidence/emulated-testnet-recovery-2026-10-29.json)
records both real exit-86 boundaries, exact recovery and retained prior history.
The authoritative ledger has **two total credits: first payment + one trial**.
The verifier made only one new keyless reconciliation; historical restart/duplicate
accept and duplicate confirm returned cached data, not fresh node queries.

**Both receiving addresses are now USED. Preserve custody, trial journals and the
original authoritative ledger. Do not rerun the reference recipe below, reset
history with an empty ledger or infer permission for a third payment.** Further
funds activity requires a genuinely fresh receiver and separate explicit native
approval. This is application-received-result loss before journaling and credit
commit interrupted before receipt publication—not wire loss, host crash, power-cut
or filesystem durability proof. Receipt absence is supported by the native marker
and reviewed contemporaneous runner assertion, not a standalone historical
filesystem-absence snapshot. Same-operator fullnode trust, not a light client or
authenticated remote business ACK; hardware remains deferred.

Independent review and focused re-review completed 29 October 2026
(`ses_e7d355137ffeaXiT10YXbCLGve`): the initial catchable-signal orphan P2 is
**RESOLVED**, with no remaining material findings. Review read source, negative
guards and harmless real-OS TERM/HUP/INT evidence only; that source review alone
was not live fault proof. The subsequent human trial above supplies the scoped
application-boundary proof, not live host-signal/power-failure qualification.
**No new agent or human funds activity is authorized by this evidence.** The first
verified checkout above remains complete; its receiver is used and its journals
must not be reset, deleted or copied into a new trial. Hardware stays deferred.

Two per-invocation native flags deliberately exit **86 without destructors**:

- `submit --fault-exit-after-execution-before-observation`: only a first,
  human-approved 10,000-unit submission, no existing intent/terminal result and
  no `--replay-saved`. After the SDK returns verified checkpoint success, emit
  `FAULT_EXIT_AFTER_EXECUTION_BEFORE_OBSERVATION` with digest/attempt/checkpoint,
  then exit before recording that dispatch observation. Intent and one attempt
  are already committed; pre-send Query/NotFound may exist. This loses an
  **application-received result before persistence**, not a dropped RPC packet.
- `accept --fault-exit-after-credit-before-receipt`: absent output (including
  links), no existing credit, verified new 10,000-unit payment. Commit the
  merchant observation/credit/exact receipt, emit
  `FAULT_EXIT_AFTER_CREDIT_BEFORE_RECEIPT`, then exit before receipt export.
  Ordinary restarted accept regenerates the committed receipt, never new credit.
  Unknown/failed responses retain normal evidence and emit `FAULT_NOT_REACHED`;
  ordinary errors or cached success are not an expected fault exit.

`scripts/testnet-recovery.py` is explicitly opt-in and keyless. It never prepares,
discovers keys, signs, injects approval text or resubmits. Native submit verifies
the saved signature, template, typed history and exact amount **before its real
terminal prompt**; Python only checks public bindings/counts and evidence. The
runner refuses prior trial intent/attempt/association, offline mode, an empty or
wrong merchant namespace, and a ledger lacking the explicitly chosen prior credit.
Read-only WAL-aware row fingerprints preserve all unrelated authoritative-ledger
rows; total credits must be baseline + one, not globally one. No SQLite backup or
database copy is made. Snapshots/output are bounded, private and non-overwriting.

After each expected exit it uses a fresh native process: reconcile saved digest,
accept with the credit fault, ordinary accept, confirm, then cached duplicate
accept/confirm. It checks unchanged full BCS/signature/digest and original files,
one new intent/attempt/credit/association, missing receipt/association at the
credit boundary, exact stored/regenerated receipt bytes and prior-row retention.
Confirm already atomically commits observation/association; no artificial split
or third flag is added. A 300-second per-process bound includes manual approval.
Any unexpected exit, timeout, unknown or mismatch stops with `STOPPED_NOT_DONE`,
retains state and writes no final PASS. Never rerun the fault or create a repair
payment. Use ordinary keyless reconciliation and obtain scoped guidance.

Signal-cleanup P2 fix is implemented and focused re-review found it **RESOLVED**.
Catchable SIGTERM/SIGHUP/SIGINT unwind to STOPPED evidence with exit `128+signal`.
Launch defers interruption until the owned handle is tracked; repeated signals
cannot interrupt cleanup. Only owned `Popen` children are stopped/waited (five
seconds after TERM, then five after KILL); prior handlers are restored afterward.
Possible broadcast/effects remain unknown: retain journals and reconcile the
saved digest, never re-sign or create a new operation. SIGKILL, root/kernel crash
and unavailable evidence storage are outside this process-cleanup guarantee;
this is not power-cut recovery or proof that no payment occurred.

### Historical recovery workflow reference (milestone complete; do not rerun)

The human retained the **old funded sender and old merchant invoice key**, using
only a new controlled receiver from private init; unused companion roles were not
the funded sender/merchant. The blocks below retain the staged workflow's variable/
path examples, not an exact shell transcript or instructions for another trial.
Do not execute them again; both recorded receivers are used. Recording/key privacy
and original-ledger preservation requirements remain in force.

Private operator locations are redacted for publication. `PRIVATE_TESTNET_STATE`
and `ORIGINAL_MERCHANT_LEDGER` denote the human-retained private custody root and
authoritative ledger, not paths supplied by this repository or new trial approval.

```sh
cd $PROJECT_ROOT && umask 077 && TRIAL="$(mktemp -d "${PRIVATE_TESTNET_STATE:?retained private-state root required}/recovery.XXXXXXXX")" && BIN=$PROJECT_ROOT/target/debug/linux-tap && RPC=https://fullnode.testnet.sui.io:443 && SENDER=0xb6b852118f51421abf2a930216ea62b88a283b5d0be7de326695bc23768d14f1 && MERCHANT_IDENTITY=0xb2177bd1848ded5ead76d6371e2c49542d2bd393145fedbc07c2c7988cc78a58 && MERCHANT_PIN=88fd7ab199410d1905c9f6794ab7e6097f45b143941ed4911658b9d67a4c0f43 && LEDGER="${ORIGINAL_MERCHANT_LEDGER:?retained authoritative ledger required}" && printf 'Retain NEW trial: %s\n' "$TRIAL" && "$BIN" testnet init --directory "$TRIAL/receiver-setup"
```

Init asks for `CREATE DISPOSABLE TESTNET WALLETS`. Only the human may generate
and verify these private keys. Never share their contents. Then privately:

```sh
"$BIN" testnet verify-setup --directory "$TRIAL/receiver-setup" && RECEIVER="$(python3 -c 'import json,sys; m=json.load(open(sys.argv[1])); assert m["profile"]=="circle-sui-testnet/disposable-software-setup/v1"; print(m["receiver"]["address"])' "$TRIAL/receiver-setup/READY.json")" && test "$RECEIVER" != 0x9a70b588da0ddd4f08c829df7fb502d7467f4699aec142fbd271f2c081027288
```

Reuse the original authoritative merchant ledger below; **never create a new
empty ledger to forget the old receiving history**. Invoice lifetime is 15 minutes.

```sh
"$BIN" testnet invoice --keystore "$PRIVATE_TESTNET_STATE/wallets/merchant.keystore" --expected-sender "$MERCHANT_IDENTITY" --receiver "$RECEIVER" --amount 10000 --merchant 'Controlled testnet recovery trial' --out "$TRIAL/invoice.json" && "$BIN" testnet register --invoice "$TRIAL/invoice.json" --merchant-key "$MERCHANT_PIN" --merchant-endpoint "$RPC" --ledger "$LEDGER" --out "$TRIAL/registration.json" &&
python3 scripts/pcsc-lab.py --prefix $PCSC_CACHE/prefix --invoice "$TRIAL/invoice.json" --out "$TRIAL/inbox.json" &&
"$BIN" testnet prepare --invoice-inbox "$TRIAL/inbox.json" --merchant-key "$MERCHANT_PIN" --keystore "$PRIVATE_TESTNET_STATE/wallets/sender.keystore" --expected-sender "$SENDER" --endpoint "$RPC" --journal "$TRIAL/wallet.sqlite" --out "$TRIAL/prepared.json"
```

Review/type `APPROVE TESTNET` only for the new receiver and 0.01 USDC. Inspect the
public preparation output before proceeding. Set `PCSC_RESULT` to **this trial's**
printed public `<PRIVATE_VALIDATION_ROOT>/result.json`; the runner binds its
snapshots to the trial invoice/inbox, not merely a claimed transport label.

```sh
PCSC_RESULT=<PRIVATE_VALIDATION_ROOT>/result.json &&
python3 scripts/testnet-recovery.py --trial-directory "$TRIAL" --merchant-ledger "$LEDGER" --preserve-credit-digest 7LemioWwAmjRYA9gqsZZBCnCnUgpESH3qfg7mJzJCJxz --merchant-key "$MERCHANT_PIN" --expected-sender "$SENDER" --endpoint "$RPC" --merchant-endpoint "$RPC" --evidence-parent "$TRIAL" --pcsc-result "$PCSC_RESULT"
```

During the separately approved run, the real native `TESTNET SUBMIT` prompt
authorized one possible broadcast. No `--approve-simulation`/keystore option exists in the
runner. Omit `--pcsc-result` only if deliberately making **no PC/SC binding claim**;
the runner never performs transport itself. PASS requires both actual fault exits,
fresh checkpoint reconciliation, independent confirm query and cached idempotency;
only then does it report `chain_submission=true`. The same official node in both
roles is not operator independence/light-client proof; these application boundaries
do not qualify RF, power-cut recovery, custody or authenticated remote business ACKs.

## Historical default-asset result (22 October 2026)

The original default-asset result is retained below. On 22 October 2026, the
public testnet node accepted native Rust gRPC reads but
rejected the existing USDC gasless template. **That historical default-asset
profile did not qualify for submission.** It is not the later explicit Circle
testnet checkout recorded above.
No keys, faucet requests, signatures, funding transactions or broadcasts were
used. Public diagnostic addresses `0x0` and `0x1` are not a funded demo wallet.

## Post-conversion public qualification (28 October 2026)

The human completed the actual 20 USDC whole-coin conversion to the same owner's
address balance. Public transaction digest
`4DoX6pEdoo7ScZy5Pda2Yeuk3XkQ89Kyxw8s5BqBP121`, checkpoint `400301849`,
epoch 1267. Previously recorded keyless reconciliation reports
`VerifiedSuccess`, `converted=true` and indexed address balance `20000000`.
The separate public payment qualification reports the pinned Circle type and
genesis, epoch 1267/protocol 139, 20,000,000 address-balance units and a
successful checked 10,000-unit (0.01 USDC) diagnostic payment simulation:
`9hwpZFoWRcH6oXny3btUPnF6Ukw3bU3s7doji4rawc8s`. Exact transaction, inner
digest and effects matched; `qualified=true`, `chain_submission=false`.
The saved public outputs are
`<PRIVATE_VALIDATION_ROOT>/reconcile.stdout.json`
and `qualification.stdout.json`. This increment read only those public files,
not wallets, and made no new chain requests.

Init, private verification and conversion do not need to be repeated. This
closes the controlled conversion/address-balance prerequisite at that snapshot,
**not merchant payment, credit or receipt association**. Fresh preparation must
qualify its actual candidate; diagnostic simulation is not signing authority.
No new custody, power-loss, hardware or RF qualification follows from conversion.

## Private PC/SC testnet checkout (human only; review completed)

Independent delivery-only increment and human-workflow review completed on
28 October 2026 (`ses_e7d4e67afffeE9pFbLkvW2fFdS`), with no blocking/material
P1/P2. Review used read-only source and existing public-fixture evidence only;
it did not run fresh tests/builds, network, keys or services. Its limits remain
virtual public fixtures, not actual merchant payment, RF, custody or power-cut proof.
**Historical completed human recipe, not next-run instructions.** All five
steps below were subsequently completed; the verified milestone is above.
The supplied receiver is now used: do not rerun these identities or create a
new ledger to bypass reuse protection. No agent may open the real wallet
directory or invoke signing/submission. Any future checkout needs separate
approval and a genuinely fresh receiver, not automatic replay of this recipe.

The new mode uses real native PC/SC/vpcd/VICC/Rust reception rather than piping
frames directly. It delivers the public signed envelope exactly; it does not
verify merchant authority or approve payment. The independently configured
merchant pin remains authoritative in `testnet prepare`. No receiver is inferred
from a display name. The supplied receiver was human-controlled and fresh for
the recorded run; it is now used. A new local ledger is not permission to reuse
a receiving identity.

Both wallet and merchant below explicitly choose the same official fullnode.
They make separate native-process queries, **not independent-operator queries**
or light-client proofs. Local receipt files are not authenticated remote
business acknowledgements. Keep recording disabled for human key use. Paths
are keystore locators only; never copy key contents into arguments or variables.

### 1. Create one fresh private operation, sign invoice and register merchant

Run the setup line once; retain its printed directory and these variables in
the same terminal. The `&&` separators remain valid if pasted as one line.

```sh
cd $PROJECT_ROOT && umask 077 && OP="$(mktemp -d "${PRIVATE_TESTNET_STATE:?retained private-state root required}/checkout.XXXXXXXX")" && BIN=$PROJECT_ROOT/target/debug/linux-tap && RPC=https://fullnode.testnet.sui.io:443 && SENDER=0xb6b852118f51421abf2a930216ea62b88a283b5d0be7de326695bc23768d14f1 && MERCHANT_IDENTITY=0xb2177bd1848ded5ead76d6371e2c49542d2bd393145fedbc07c2c7988cc78a58 && MERCHANT_PIN=88fd7ab199410d1905c9f6794ab7e6097f45b143941ed4911658b9d67a4c0f43 && RECEIVER=0x9a70b588da0ddd4f08c829df7fb502d7467f4699aec142fbd271f2c081027288 && printf 'Retain checkout directory: %s\n' "$OP"
```

This human-only command signs the 10,000-unit invoice using the existing
merchant key, then registers its receiver **before transport/dispatch**:

```sh
"$BIN" testnet invoice --keystore "$PRIVATE_TESTNET_STATE/wallets/merchant.keystore" --expected-sender "$MERCHANT_IDENTITY" --receiver "$RECEIVER" --amount 10000 --merchant 'Controlled testnet merchant' --out "$OP/invoice.json" && "$BIN" testnet register --invoice "$OP/invoice.json" --merchant-key "$MERCHANT_PIN" --merchant-endpoint "$RPC" --ledger "$OP/merchant.sqlite" --out "$OP/registration.json"
```

### 2. Deliver the exact public invoice over private PC/SC

The final executable must have `pcsc-reader` enabled; default `scripts/check.sh`
replaces it with a default-feature binary. The implementation validation ended
with the feature build. No caller `LD_LIBRARY_PATH` or key-wrapping environment
is required; only the delivery subprocess configures its private client library.
Do not rerun dependency downloads/builds for the existing qualified cache.

```sh
python3 scripts/pcsc-lab.py --prefix $PCSC_CACHE/prefix --invoice "$OP/invoice.json" --out "$OP/inbox.json"
```

Require `APDU_DELIVERED_NOT_APPROVED_NOT_PAID`. The output is a new mode-0600
single-link regular file under the mode-0700 operation directory. The lab's
private transport evidence remains under its printed `<PRIVATE_VALIDATION_ROOT>`
path. This is transport acknowledgment, never payment/approval or a business ACK.

### 3. Human-private wallet preparation; then stop

```sh
"$BIN" testnet prepare --invoice-inbox "$OP/inbox.json" --merchant-key "$MERCHANT_PIN" --keystore "$PRIVATE_TESTNET_STATE/wallets/sender.keystore" --expected-sender "$SENDER" --endpoint "$RPC" --journal "$OP/wallet.sqlite" --out "$OP/prepared.json"
```

Review the pinned merchant, receiver, exact asset, **10,000 units / 0.01 USDC**,
chain, expiry and candidate digest. Type `APPROVE TESTNET` only if correct.
Require `TESTNET_PREPARED`; share public status/digest only, never keys. No funds
are submitted by preparation. Stop here before the following later steps.

### 4. Later human keyless submission, after separate approval

```sh
"$BIN" testnet submit --invoice-inbox "$OP/inbox.json" --merchant-key "$MERCHANT_PIN" --expected-sender "$SENDER" --endpoint "$RPC" --merchant-endpoint "$RPC" --journal "$OP/wallet.sqlite" --registration "$OP/registration.json"
```

Review and type `TESTNET SUBMIT` only when separately approved. Saved digest
lookup and exact-candidate qualification precede dispatch; no new transfer is
constructed by this command. If uncertain, stop and use only the keyless saved
digest lookup below—do not redo step 1, create another invoice or sign a repair.

```sh
"$BIN" testnet reconcile --invoice-inbox "$OP/inbox.json" --merchant-key "$MERCHANT_PIN" --expected-sender "$SENDER" --endpoint "$RPC" --merchant-endpoint "$RPC" --journal "$OP/wallet.sqlite"
```

### 5. Later merchant credit and wallet confirmation, only after verified payment

```sh
"$BIN" testnet accept --invoice "$OP/invoice.json" --merchant-key "$MERCHANT_PIN" --merchant-endpoint "$RPC" --ledger "$OP/merchant.sqlite" --prepared "$OP/prepared.json" --out "$OP/receipt.json" && "$BIN" testnet confirm --invoice-inbox "$OP/inbox.json" --merchant-key "$MERCHANT_PIN" --expected-sender "$SENDER" --endpoint "$RPC" --merchant-endpoint "$RPC" --journal "$OP/wallet.sqlite" --receipt "$OP/receipt.json"
```

These are separate keyless process queries and durable local credit/association,
not proof of independent node operators or authenticated remote merchant ACK.
Unknown results retain the same journals/artifact. Never silently switch node,
nonce, epoch, receiver or asset. See the existing controlled-dispatch recovery
rules below rather than automatically replaying or rebuilding.

## Native conversion inspection (28 October 2026)

**Historical first inspection:** the false blocker below was subsequently
reviewed and fixed. See the [current lifecycle](#native-conversion-lifecycle-review-completed).

The human reports successful private init/verify-setup and Circle faucet
completion; no agent checked private key contents. The prior post-funding
public snapshot found 20 USDC in coin objects but zero address balance, so the
10,000-unit merchant payment remained ineligible.

The native read-only command now uses the fixed official endpoint and genesis:

```sh
target/debug/linux-tap testnet conversion inspect \
  --sender 0xb6b852118f51421abf2a930216ea62b88a283b5d0be7de326695bc23768d14f1
```

It selects the smallest single eligible address-owned exact Circle `Coin<T>`
covering 10,000 units and the current protocol minimum. It checks official SDK
object BCS against ID/version/digest and re-reads that exact reference; shared,
receiving, wrong-owner/type and changed references fail closed. A bounded
single-page scan refuses partial results. There are no retries or alternate
coins after failure. Its entire snapshot has a 60-second deadline.

The unsigned PTB contains only that whole owned coin, BCS of its owner's address,
and `0x2::coin::send_funds<CircleUSDC>(Input(0), Input(1))`. Gas objects, price
and budget are empty/zero; `ValidDuring` pins genesis and the current epoch.
Diagnostic nonce zero and printed BCS are **not** a signing/submission artifact.
No fake merchant invoice or payment-template substitution is used. The whole
selected coin would be consumed; this is not a 0.01 USDC split or merchant
payment. Gasless execution forgoes the storage rebate; no rebate is promised.

One actual checked unsigned simulation was run, with transaction checks enabled
and gas selection disabled. It returned success at epoch 1266/protocol 139:

- Coin `0x84d893750b52dfce1d012778c964e93cb59d45fb874caa6681bf56948a7b4bf9`,
  version `951287977`, digest `FHXmv9TgfCZA7Bzyqz79iZiER4nzrr9L2uzuCSeSSQm9`.
- Whole value `20,000,000` units (20 USDC), recipient exactly the sender above.
- Candidate digest `5STxwgqXPGtRwebskWmcG9FC14ExP6QKHm2K4ciGS9pD`.
- Effects show deletion of that reference and a `Merge Integer(20000000)` write
  to that owner's `Balance<CircleUSDC>`, no gas object. Reported computation
  cost/storage rebate were both `1322268`, nonrefundable storage fee `132`;
  these are simulated accounting fields, not SUI funding requirements.
- **Qualification failed (exit 1)** because the conservative conversion verifier
  rejects unchanged consensus inputs, while effects include `PER_EPOCH_CONFIG`
  at `0x403`. This is an implementation-policy mismatch, not a failed on-chain
  execution or evidence that a paid-gas fallback is needed. No retry occurred.

At this historical checkpoint the command stayed fail-closed. Review of the exact system-read semantics and
SDK effects coverage is required before any narrowly justified policy change.
`prepare`, durable approval/recovery, signing, `submit` and `reconcile` for
conversion are **not implemented**. No human conversion command is ready yet.
After implementation and independent review, a separate explicit human approval
must cover the then-current whole coin/reference/value, same-owner recipient,
gasless constraints and foregone rebate before human-private key use. Faucet
completion is not conversion approval. No keys were read, signatures made,
execution RPCs called or funds moved by this inspection.

## Native conversion lifecycle (review completed)

Independent lifecycle review completed on 28 October 2026 in
`ses_e7c57274cffe7qT1NSnKx1gggB`, with no demonstrated P1/P2 finding; the
previous read-only `0x403` P2 is resolved. Review covered source, public unsigned
qualification and public-fixture acceptance/logs only. Template/self-recipient/
whole-coin/reference/epoch/zero-gas binding, pre-sign reservation and key/dump
ordering, typed approvals/recovery/artifacts, offline guards, digest-first
intent-before-execution and conservative unknown/checkpoint effects verification
were checked. The 1,537 lines were bounded to four actual responsibilities;
no redundant framework was found. This is not actual live signing, execution,
checkpoint conversion or payment evidence. Human conversion approval/signing
were pending at review; the subsequent human conversion and saved public
reconciliation are recorded above. Review itself grants no execution authority.

Independent inspection-source review `ses_e7c57274cffe7qT1NSnKx1gggB` reported
one P2: the blanket unchanged-consensus rejection above. The fix permits
**exactly one** unchanged object, ID `0x403`, kind `PerEpochConfig`; missing,
duplicate, extra or other-kind entries still fail. Pinned SDK `1313d51`
`effects/v2.rs` describes these entries as unmutated and `PerEpochConfig` as
epoch-fixed reads. The GraphQL schema identifies `0x403` as CoinDenyList.
Runtime-loaded deny-list child information is not an ordinary mutation and is
not present in the SDK's typed BCS effects. No child or deny-list writes are
allowed: changed objects remain **exactly** the selected owned coin deletion
and a same-owner `Balance<CircleUSDC>` `Merge Integer(full value)` credit.
Gas accounting counters need not be zero; no gas objects, price or budget may
be introduced.

One corrected public checked unsigned inspection passed at checkpoint
`400242202`, epoch 1266/protocol 139, with the same 20,000,000-unit coin,
version/digest and candidate digest recorded above. `qualified=true`, no
blockers, `chain_submission=false`. Exact response/BCS is retained locally at
`<PRIVATE_VALIDATION_ROOT>/inspection.json`.
This is unsigned simulation evidence, **not conversion or settlement**.

The implemented lifecycle is separate from merchant payment:

- `prepare` selects the smallest eligible single whole coin, freezes its exact
  BCS/reference/epoch/nonce, and prompts with the full amount, same-owner
  recipient, asset/genesis/endpoint, zero-SUI-gas terms and foregone rebate.
  A dedicated journal stores one immutable operation; its ID hashes all saved
  terms and exact bytes, not merely nonce zero. The owned input reference makes
  the transaction unique without generating wallet entropy.
- After terminal approval, the exact candidate is checked again. Typed
  qualification, approval and pre-sign reservation commit **before** opening
  the key and before signing. The existing official legacy Ed25519 loader is
  used only after checked dump protection. The official SDK signs only the
  reconstructed conversion template; private material is never journaled.
- Saved artifacts are cryptographically verified, matched to the candidate and
  exported without replacement. Unresolved issuance requires explicit
  `--recover-saved-candidate` and a new recovery prompt; every recovery record
  is strictly decoded/validated. Recovery never rediscovers a coin or changes
  epoch/reference/nonce. An expired candidate remains saved and is not refreshed.
- Keyless `submit`/`reconcile` load that approved journal artifact, not arbitrary
  external transaction JSON. They query its digest **before** current epoch,
  coin or simulation checks, so a consumed coin does not imply failure.
- First dispatch requires `CONVERT TESTNET WHOLE COIN`; a prior intent requires
  explicit `--replay-saved`, a fresh NotFound lookup, exact-candidate eligibility
  and `REPLAY SAVED TESTNET CONVERSION`. Intent and attempt approval commit
  transactionally before SDK execution. Whole execution is bounded to 60 seconds
  around the SDK's 40-second checkpoint helper. Lookup is bounded to 30 seconds,
  qualification to 60 seconds. There is no retry, endpoint fallback or paid gas.
- Unknown/timeout/partial outcomes retain the same artifact. Any saved checkpoint
  result, even an unverified checkpoint-bearing response, blocks resend after
  degraded later RPC. Verified success requires exact signature, SDK transaction
  and digest, checkpoint, epoch, successful typed effects, consumed reference
  and full same-owner accumulator credit. Verified failure is separate from
  uncertainty and cannot authorize a fresh conversion.
- After verified success, a separate 15-second address-balance read reports
  indexing availability. Missing/lagging balances never undo checkpoint success
  or trigger conversion again. Merchant payment still requires the unchanged
  `qualify-testnet --circle-testnet-usdc` and its independent merchant workflow.

**Historical conversion recipe: the existing human conversion is complete;
do not repeat it.** For another separately authorized fresh setup, independent
lifecycle review is complete, but do not run the following until
separate explicit human conversion approval. Faucet completion and review
were not that approval.
The actual whole value in the snapshot is **20 USDC to the same owner**, not
0.01 USDC to a merchant; fresh preparation must display its then-current coin.
All real key use is human-private, with recording and agent access disabled.
No need to regenerate keys, copy coin IDs, decode JSON or install a Sui CLI.

```sh
# FUTURE HUMAN-ONLY RECIPE, AFTER SEPARATE APPROVAL. Review complete; not run by the agent.
# Replace these two paths; keep the existing sender keystore, create a NEW private operation directory.
KEYSTORE=/absolute/path/to/existing/sender.keystore
OP=/absolute/private/path/to/new-conversion-operation
(umask 077; mkdir -- "$OP")
SENDER=0xb6b852118f51421abf2a930216ea62b88a283b5d0be7de326695bc23768d14f1
target/debug/linux-tap testnet conversion prepare \
  --keystore "$KEYSTORE" --expected-sender "$SENDER" \
  --journal "$OP/conversion.sqlite" --out "$OP/prepared.json"
# Review the displayed whole coin and type PREPARE TESTNET CONVERSION only if approved.
# Keyless dispatch still requires a second terminal confirmation:
target/debug/linux-tap testnet conversion submit \
  --expected-sender "$SENDER" --journal "$OP/conversion.sqlite"
```

The endpoint defaults to the fixed `https://fullnode.testnet.sui.io:443` and
any other supplied `--endpoint` is rejected. This explicitly trusts that node's
checkpoint observation, not a light-client proof. On uncertainty use only
`testnet conversion reconcile --expected-sender "$SENDER" --journal "$OP/conversion.sqlite"`.
Do **not** delete the journal, choose another coin or create another operation
to resolve uncertainty. If preparation alone reports unresolved issuance, retain
the journal and rerun the same `prepare` command with
`--recover-saved-candidate`; never add an offline flag to a real operation.
Public-fixture mode accepts only known key `[9;32]` with a fixed synthetic coin;
its artifacts are never live-submittable/reconcilable, regardless of claimed
public sender. The fixture blacklist is finite, not a secrecy/freshness proof.

These are unencrypted, cooperative single-owner local journals and host-terminal
approvals, not production custody, hostile-local-storage authentication,
global cross-journal coordination or power-loss certification. The later human
conversion checkpoint is evidenced above; merchant payment remains unexercised.

## Reproduce the read-only check

```sh
cargo build --locked -p tap-cli
target/debug/linux-tap qualify-testnet \
  --endpoint https://fullnode.testnet.sui.io:443 \
  --chain-id 69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD \
  --sender 0x0 --receiver 0x1 --amount 10000 > qualification.json
```

Expected for the recorded network state: JSON with `qualified: false`,
`chain_submission: false`, and exit code 1. The full genesis digest must be
pinned independently of the chosen endpoint; the name “testnet” alone is not a
chain pin. This command also requires the node to report testnet and requires
the epoch/chain to stay unchanged during the check. HTTPS is mandatory except
for explicit loopback development nodes. Each RPC has a 15-second deadline;
there is no automatic retry or endpoint fallback. Reads trust that chosen node,
not a light-client proof or permanent network guarantee.

Without the explicit flag below, the command still uses `tap_core::USDC`:
a sender withdrawal,
`balance::redeem_funds`, `balance::send_funds`, exact recipient/amount, no gas
objects, zero gas price/budget, and current-epoch `ValidDuring` expiration with
the full chain digest. It uses Mysten BCS/digest/types, not a new transaction
encoder. Nonce zero is diagnostic only; the unsigned bytes are not a payment
artifact or approval to spend. It never reads a private key or merchant key.

Qualification requires the lossless protocol flags and token allowlist, exact
asset metadata with six decimals, sufficient **address balance** (not summed
coin objects), and successful simulation of unchanged transaction bytes/digest
and effects. Simulation explicitly enables transaction checks and disables gas
selection. A probe, unchecked simulation, successful RPC transport, or SUI coin
balance alone cannot qualify this payment. Simulation does not verify a user
signature, settlement, merchant identity or permission to submit.

## Actual result

### Original default asset (unchanged)

The [saved evidence excerpt](evidence/testnet-qualification-2026-10-22.json)
retains the identity, exact unsigned bytes, selected lossless protocol fields
and blockers from the live report. Other protocol fields are intentionally
omitted from that excerpt; the command emits the full configuration.

- Chain: `69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD`, epoch 1260,
  initial checkpoint 397908031, protocol version 139.
- `enable_gasless`, `enable_accumulators` and `enable_object_funds_withdraw`
  were all true. Gasless support is not the blocker in general.
- The allowlist contained
  `0xa1ec7fc00a6f40db9693ad1415d0c193ad3906494428cf252621037bd7117e29::usdc::USDC`,
  **not** the project's pinned
  `0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC`.
- `GetCoinInfo` for the pinned type returned NotFound. The diagnostic sender's
  total balance was zero and no usable address balance was confirmed.
- Checked `SimulateTransaction` returned InvalidArgument: “Fund type
  0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC
  is not currently allowed in gasless transactions”. No successful effects were
  returned; this is a live rejection, not successful dry-run evidence.

Do not fix this by requiring SUI or changing the asset silently. The explicit
testnet policy below was separately authorized; the default remains unchanged.
Submission/reconciliation code and any specific testnet broadcast remain gated.
The recording demo stays an offline simulation with no physical NFC claim.

The node identified its build as `1.82.2-ecf82404d44d`. The corresponding
[upstream protocol source](https://github.com/MystenLabs/sui/blob/ecf82404d44d/crates/sui-protocol-config/src/lib.rs)
independently names these `MAINNET_USDC` and `TESTNET_USDC`, and defines the
allowlist entries as token types with minimum transfer sizes. That confirms the
network-specific mismatch; it does not establish that an alternate asset or a
funded transaction would simulate successfully. The SDK remains pinned at
`1313d5112464ecc25d7407f42442adea08589fc5`; no vendor code was changed.

## Explicit Circle testnet configuration

Independently checked against Circle's [official contract addresses](https://developers.circle.com/stablecoins/usdc-contract-addresses)
on 22 October 2026, the **Sui Testnet** entry is exactly:

```text
0xa1ec7fc00a6f40db9693ad1415d0c193ad3906494428cf252621037bd7117e29::usdc::USDC
```

This is a no-financial-value test token, not mainnet USDC or a symbol-based
substitute. `--circle-testnet-usdc` enables only this type on genesis
`69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD`. The node must also report
testnet. A reset or wrong chain fails closed; do not update the pin just to pass.
Metadata must confirm the exact type and six decimals; the current protocol
must allow the exact type/amount. No arbitrary asset override is available.

Public-address diagnostic reproduction (no keys, signatures or funding requests):

```sh
target/debug/linux-tap qualify-testnet --circle-testnet-usdc \
  --endpoint https://fullnode.testnet.sui.io:443 \
  --chain-id 69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD \
  --sender 0x0 --receiver 0x1 --amount 10000 > circle-qualification.json
```

The [saved evidence excerpt](evidence/circle-testnet-qualification-2026-10-22.json)
records `qualified: true`, exit 0, protocol 139 and successful checked simulation
using **pre-existing funded third-party public address `0x0`**. Nobody has
established control of that address here. Never fund it or use it as your wallet.
Unsigned bytes, inner transaction digest and effects transaction digest match;
the node omits the optional outer `ExecutedTransaction.digest`. The verifier
requires the inner digest, rejects a conflicting outer digest when present,
and checks effects directly against the SDK digest. No signature is verified
by simulation. This is neither fresh-wallet funding nor execution/settlement.
Simulation is epoch-scoped and may stop qualifying as state changes.

In the original qualification increment, the alternate policy was confined to
read-only qualification. Invoice terms carry
the exact chain/type and are checked in the chain core before template building;
the template is reconstructed exactly with the same explicit policy. These
diagnostic terms are not signed merchant invoices. `prepare-draft`, the
development signer, receipt verification, merchant trust pins, draft v0 and the
offline demo retain their existing behavior and reject this alternate asset.
There is no runnable end-to-end testnet checkout in this increment.

## Current preparation checkpoint — 28 October 2026

The later explicit `testnet invoice` / `testnet prepare` CLI now carries this exact
fixed-genesis/type policy through authenticated invoices, template construction,
software signatures and optional `verify-on-node --circle-testnet-usdc` receipt
verification. Legacy defaults and the historical results above remain unchanged.
Preparation's live path calls `qualification::qualify_candidate` on the frozen
actual transaction, not the nonce-zero diagnostic builder. It requires current
epoch, protocol/type/metadata/address-balance checks and unchanged checked
simulation with gas selection disabled before invoking a signer. A saved artifact
is loaded/verified without re-signing or claiming current eligibility.

Only public-fixture offline PTY approval/recovery was exercised in this checkpoint;
no existing keys were accessed, no fresh secrets generated, no faucets requested,
no RPCs or broadcasts performed. The offline flag accepts only the known public
fixture sender/merchant pins, records qualification as absent, and cannot imply
ready-to-submit or paid. Never fund these keys. Independent review precedes any
controlled live signing/execution trial. See README for key-interface limits and
validation for exact local evidence. Fresh controlled funding/conversion,
independent merchant settlement and dispatch/reconciliation barriers remain open.

## Fresh disposable wallet setup (human-operated, not performed by the agent)

Historical setup recipe for separately approved new setups only. The supplied
human setup/private verification/conversion/checkout is complete; do not repeat it.

**Init-only source review completed on 28 October 2026**
(`ses_e7d355137ffeaXiT10YXbCLGve`), with no blocking/material P1/P2 finding.
See the [review record](validation.md#human-gated-disposable-testnet-setup--28-october-2026).
This is not a production security audit or proof of positive generation,
publication or failure cleanup. Subsequently the human reported successful init;
metadata-only/public READY checks passed, without agent access to any private key.
The remaining private SDK decode prerequisite now has a native command:
**`verify-setup` review completed on 28 October 2026**
(`ses_e7d355137ffeaXiT10YXbCLGve`), with no blocking/material P1/P2 finding.
Source and public-fixture evidence covered matching, tamper rejection, unchanged
files and no key payload in output. The user's wallet was not accessed or its
keys read; human-private verification was pending at that review and was
subsequently reported complete by the human. See the
[verification review record](validation.md#human-private-read-only-setup-key-matching--28-october-2026).
This is not proof of key freshness, funding, eligibility or settlement.
No native `sui` executable was available on the inspected PATH. Rather than
installing a large CLI just for key generation, this bounded command reuses the
pinned official SDK. The agent has not run its fresh-key path.

The next qualification is **human-private and unfunded**. With an existing owned
canonical mode-0700 absolute parent and a new leaf that does **not** exist, the
human's private setup is the following reviewed sequence:

```sh
target/debug/linux-tap testnet init --directory /absolute/private-parent/fresh-testnet
target/debug/linux-tap testnet verify-setup --directory /absolute/private-parent/fresh-testnet
cat /absolute/private-parent/fresh-testnet/READY.json
```

The first requires a terminal and exact typed `CREATE DISPOSABLE TESTNET WALLETS`.
If init already succeeded, **skip it**: keep the existing directory and keys,
and privately run only the reviewed read-only verifier.
There is no seed input, unattended approval, default wallet search, RPC, faucet
or signing action. Run with recording, shell tracing and agent access disabled.
The three dedicated SDK-generated Ed25519 keys use OS entropy and official legacy
Sui CLI JSON/Base64 encoding: `sender.keystore`, `merchant.keystore` and
`receiver.keystore`. New directory/files are owned mode 0700/0600, without links
or replacement. Linux process dumpability is disabled and checked before entropy
use. Private files and directory entries are synced before atomic no-replace
publication of the **public-only** `READY.json`. It contains public addresses,
public keys/pin, explicit file paths and the fixed Circle type/genesis/profile,
not keys or RPC credentials. Its status is
`PROVISIONED_ONLY_NOT_FUNDED_OR_ELIGIBLE`.

Require successful command completion as well as READY. On error/interruption,
treat the directory as **STOPPED**: retain private files, do not fund/use or retry
over it. Incomplete generation has no READY publication; ordinary post-publication
errors attempt to withdraw only the command's own manifest. If filesystem failure
prevents withdrawal, the command explicitly warns not to use that directory even
if a READY file remains. No other data or private files are deleted. This is not
power-loss qualification, encryption, guaranteed memory erasure, backup or root/
same-user adversary isolation. SDK/JSON temporary buffers may remain in memory or
swap. Keys themselves are not cryptographically network-bound: **never use these
disposable files for mainnet or production custody**. The receiver key remains
under your control, retained privately/offline; merchant observation needs only
its public receiving address.

**Minimum positive qualification before funding:** the human must privately
confirm exit status 0 and no STOPPED condition; check ownership, mode 0700 on the
new directory and mode 0600 on regular files (no symlinks; link count 1); obtain a
successful `testnet verify-setup --directory SAME_EXISTING_DIRECTORY` result.
The verifier checks the fixed profile/status/genesis/type/10000-base-unit policy,
owned canonical directory and regular private files, then disables/independently
checks process dumpability before any key bytes are read. Only the three fixed
role files are opened; READY cannot redirect reads to arbitrary paths. The
existing official legacy SDK decoder must find exactly one Ed25519 key per file
matching both READY's public key and derived address; roles must be distinct and
not the four built-in/project laboratory fixture vectors (7/9/11/12). A blacklist cannot identify every
publicly disclosed key or prove fresh entropy. This is key matching only, not
funding, eligibility, private-key secrecy, power-loss or production-custody proof.
The command is read-only: no signing, RPC, READY update or private output, and no
TTY approval is required for this explicit read-only operation. Its public-only
JSON has `verified: true` and `status: KEY_MATCHES_ONLY_NOT_FUNDED_OR_ELIGIBLE`.
Do not print raw keystore entries or send keys to an agent. Only public match/pass
information may be reported. Actual user-wallet key matching was subsequently
reported complete by the human;
agents must not invoke verification on that real directory. Source review did
not execute generation/publication/cleanup or replace the human-private check.
Do not request faucet funding before these checks pass.

1. Keep all three keystores private; never `cat` them, paste their contents into
   chat, or put keys/mnemonics in arguments, environment, shell history or logs.
   Share only public addresses/pin when coordinating checkout. Do not reuse
   production keys or public laboratory seeds. Only READY's public data is used
   in the later configuration block below.
2. Do not run `prepare-draft` for this task. Its existing raw 32-byte mode-0600
   seed-file interface is development software, not a Sui keystore importer or
   secure custody solution, and it does not enable the Circle testnet policy.
   No seed creation/import, signing or submission is needed for qualification.
3. The user opens [Circle's public faucet](https://faucet.circle.com/), selects
   **USDC / Sui Testnet**, supplies only their new public sender address and
   completes the CAPTCHA themselves. The faucet advertises 20 test USDC per
   address/blockchain every two hours; availability and limits may change.
   The agent must not request funding or bypass CAPTCHA. Wait for user completion
   before any fresh-funded validation. Tokens have no monetary value.
4. A faucet receipt or summed balance is insufficient. Qualification reads the
   exact token's **address balance**, separately from `coinBalance`. If funding
   arrives as `Coin<USDC>` objects, they cannot satisfy our withdrawal template.
   Research reported a successful read-only coin-to-address conversion simulation
   using an already-funded public address; that did not convert any real funds
   or establish a fresh-wallet procedure. **The original qualification increment
   had no conversion command or submission path.** The controlled submission
   below remains the merchant-payment flow, not conversion. A separate native
   [conversion lifecycle](#native-conversion-lifecycle-review-completed) now exists
   and its independent review is complete. The existing human conversion is
   complete; future conversions still need separate explicit approval. Do not
   implicitly convert on faucet completion. No blanket SUI requirement is added.
5. After user-confirmed faucet completion, the next authorized read-only check
   uses only public addresses (replace both placeholders):

   ```sh
   target/debug/linux-tap qualify-testnet --circle-testnet-usdc \
     --endpoint https://fullnode.testnet.sui.io:443 \
     --chain-id 69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD \
     --sender YOUR_FRESH_PUBLIC_SENDER --receiver YOUR_FRESH_PUBLIC_RECEIVER \
     --amount 10000 > fresh-wallet-qualification.json
   ```

   Exit 1 / `qualified: false` means stop and inspect blockers; do not disable
   checks, substitute types, use someone else's funded address, or sign a fresh
   transfer to repair an unknown result. Even exit 0 is read-only eligibility,
   not authority to sign/broadcast. The supplied controlled sender's later
   positive public qualification is recorded above; it is not payment proof.
   A later live trial needs separate submission authorization, reconciliation,
   independently trusted merchant pins and actual checkpoint evidence. Physical
   NFC and trusted hardware approval remain untested.

## Controlled submission and independent merchant flow

**Historical 28 October source-review checkpoint, not itself live qualification.** Independent review completed
on 28 October 2026 (`ses_e7d355137ffeaXiT10YXbCLGve`): preparation's qualification-record
P2 and dispatch's self-transfer P2 are both **RESOLVED**, with no remaining finding
in the reviewed dispatch scope. See the [review record](validation.md#dispatch-review-p2--reject-merchant-self-transfers).
Review used read-only source, pinned SDK and private offline evidence; it does
not establish actual RPC execution, timeout behavior or settlement. No controlled sender,
receiving wallet, faucet action, live signing, broadcast or merchant chain credit
was exercised by the agent. Historical qualification results above are unchanged.
Offline checks alone did not meet the real end-to-end goal; the subsequent
29 October human checkout and independent verification above close one such run.

For a new setup, human-private provisioning and faucet/address-balance
prerequisites must precede the following commands. The supplied existing setup
has completed them and the checkout; do not reprovision or rerun either historical
recipe with its now-used receiver. The generic commands below document the
flow, not authorization for another transfer. Use
fresh, dedicated no-value testnet accounts and the private manual setup above.
Never share keys, mnemonics, keystore contents or faucet credentials with an agent;
complete any CAPTCHA privately yourself. Never fund any public laboratory seed.
The receiving address must be controlled and fresh, not a random example address.
The invoice signer identity and independent wallet merchant pin must be agreed
out of band. Use explicitly chosen trusted testnet gRPC endpoints; preferably
independent operators for wallet and merchant. The full fixed genesis/type are
built into the profile, not inferred from a symbol, endpoint name or receipt.

For the later checkout, load **only public configuration/explicit file paths**
from your own successfully created READY manifest, never from an untrusted remote
file. This formatter opens no keystore and outputs only fixed, shell-quoted public
assignments. Stop if it fails. Private key contents never belong in variables,
arguments or recordings. Choose the two trusted RPC endpoints yourself; init
neither guesses them nor stores credentials:

```sh
BIN=target/debug/linux-tap
READY=/absolute/private-parent/fresh-testnet/READY.json
PUBLIC_CONFIG=$(python3 - "$READY" <<'PY'
import json, shlex, sys
with open(sys.argv[1]) as f:
    m = json.load(f)
assert m["profile"] == "circle-sui-testnet/disposable-software-setup/v1"
assert m["status"] == "PROVISIONED_ONLY_NOT_FUNDED_OR_ELIGIBLE"
values = {
    "LAB": m["directory"],
    "SENDER_KEYSTORE": m["sender"]["keystore"],
    "MERCHANT_KEYSTORE": m["merchant_invoice"]["keystore"],
    "SENDER": m["sender"]["address"],
    "MERCHANT_IDENTITY": m["merchant_invoice"]["address"],
    "MERCHANT_PIN": m["merchant_invoice"]["public_key_hex"],
    "FRESH_RECEIVER": m["receiver"]["address"],
}
for name, value in values.items():
    print(f"{name}={shlex.quote(value)}")
PY
) && eval "$PUBLIC_CONFIG"
WALLET_RPC=https://YOUR_EXPLICIT_TRUSTED_TESTNET_GRPC_ENDPOINT
MERCHANT_RPC=https://YOUR_INDEPENDENTLY_CHOSEN_TESTNET_GRPC_ENDPOINT
```

Paths must already have the documented ownership/permissions. The new human-only
init command is not reviewed production encrypted custody. Independently configure
the generated merchant pin for the wallet as agreed out of band. The merchant invoice
command alone needs its dedicated signing key; the running observer/ledger does not.

```sh
# Create a signed testnet invoice; independently verify the configured public pin.
"$BIN" testnet invoice --keystore "$MERCHANT_KEYSTORE" \
  --expected-sender "$MERCHANT_IDENTITY" --receiver "$FRESH_RECEIVER" \
  --amount 10000 --merchant 'YOUR TESTNET MERCHANT' --out "$LAB/invoice.json"

# Register BEFORE transport/dispatch. One immutable namespace per merchant DB;
# chain/receiver is never reused for a different invoice, even after expiry.
"$BIN" testnet register --invoice "$LAB/invoice.json" --merchant-key "$MERCHANT_PIN" \
  --merchant-endpoint "$MERCHANT_RPC" --ledger "$LAB/merchant.db" \
  --out "$LAB/registration.json"

# Actual native emulated APDU framing/receiver, not a payment acknowledgement.
"$BIN" apdu-frames --input "$LAB/invoice.json" | \
  "$BIN" apdu-stdio --out "$LAB/inbox.json"

# Typed APPROVE TESTNET: exact-candidate eligibility, reservation, software signature.
"$BIN" testnet prepare --invoice-inbox "$LAB/inbox.json" --merchant-key "$MERCHANT_PIN" \
  --keystore "$SENDER_KEYSTORE" --expected-sender "$SENDER" --endpoint "$WALLET_RPC" \
  --journal "$LAB/wallet.db" --out "$LAB/prepared.json"

# KEYLESS. Type TESTNET SUBMIT only after reviewing exact terms/endpoint/digest.
"$BIN" testnet submit --invoice-inbox "$LAB/inbox.json" --merchant-key "$MERCHANT_PIN" \
  --expected-sender "$SENDER" --endpoint "$WALLET_RPC" --merchant-endpoint "$MERCHANT_RPC" \
  --journal "$LAB/wallet.db" --registration "$LAB/registration.json"

# Separate KEYLESS merchant process: independently query its pinned node, then
# atomically persist the verified observation and unique credit/immutable receipt.
"$BIN" testnet accept --invoice "$LAB/invoice.json" --merchant-key "$MERCHANT_PIN" \
  --merchant-endpoint "$MERCHANT_RPC" --ledger "$LAB/merchant.db" \
  --prepared "$LAB/prepared.json" --out "$LAB/receipt.json"

# KEYLESS wallet: independently requery the chosen merchant node, verify receipt
# association, then atomically save observation + association. Not a business ACK.
"$BIN" testnet confirm --invoice-inbox "$LAB/inbox.json" --merchant-key "$MERCHANT_PIN" \
  --expected-sender "$SENDER" --endpoint "$WALLET_RPC" --merchant-endpoint "$MERCHANT_RPC" \
  --journal "$LAB/wallet.db" --receipt "$LAB/receipt.json"
```

Keep the 15-minute invoice window in mind; register/prepare promptly. The sender
must hold the exact token's sufficient **address balance**. Coin objects alone
are insufficient: stop rather than automatically converting or constructing a
different transaction. No blanket SUI requirement is imposed on this qualified
gasless transfer. The profile remains software key/host approval, not trusted
hardware display or production custody.

`submit` loads/revalidates the approved immutable artifact, never a key or builder.
Its endpoint must exactly match preparation; changing it requires separate future
policy, not a hidden fallback. First-send terminal decline performs no RPC or
dispatch. The digest is looked up before current eligibility. If not already
observed, fresh protocol/metadata/address-balance/checked unchanged simulation
must qualify the actual saved candidate. First intent and first attempt approval
commit transactionally **before** the execution helper is polled. An outer
60-second timeout bounds subscription, execution, helper probing and checkpoint
wait together; the helper's own 40-second wait is not the whole deadline.

Any interrupted attempt is possibly sent. Query first, without balance/expiry or
current-epoch gates on the observation:

```sh
"$BIN" testnet reconcile --invoice-inbox "$LAB/inbox.json" --merchant-key "$MERCHANT_PIN" \
  --expected-sender "$SENDER" --endpoint "$WALLET_RPC" --merchant-endpoint "$MERCHANT_RPC" \
  --journal "$LAB/wallet.db"
```

Verified checkpoint success, verified checkpoint failure, NotFound and unknown
are distinct. NotFound/timeouts/RPC/helper errors never authorize a replacement.
To deliberately resend after reconciliation, rerun the exact `submit` command
with `--replay-saved` and type `TESTNET REPLAY SAVED`. It rechecks the saved digest,
allows only a still-eligible identical transaction and appends a durable attempt
approval before execution. Unknown RPC results block it; a known checkpoint
result prevents resubmission even if a later lookup says NotFound. Old epochs or
spent balances can naturally prevent replay; never change nonce/epoch or sign a
fresh payment to repair that uncertainty. No automatic retry/backoff is provided.

Merchant `accept` has no current-balance, invoice-expiry or current-epoch gate.
It requires exact transaction/effects/inner digests, unchanged SDK transaction
bytes, the stored signature, successful effects and a checkpoint from its chosen
fixed-genesis node. Failed/unknown observations are stored without credit. A late
successful observation is credited with `observed_late=true`, meaning local
observation after invoice expiry, **not** a claimed execution timestamp or refund.
Repeating an identical accepted claim returns the identical existing receipt;
different associations fail closed. Local uniqueness is not global custody or
onchain invoice deduplication. Keep the merchant journal: replacing it defeats
its local no-reuse history. Power-loss durability, backups and custody remain
unqualified.

Registration/receipt files are local coordination artifacts, not cryptographically
authenticated remote merchant messages. The independent merchant key pin always
authenticates invoice terms; the explicitly chosen node is trusted for chain
inclusion, not a light-client proof. Wallet confirmation independently observes
the chain instead of believing a `paid` field in a file; it does **not** authenticate
delivery of goods or a remote business acknowledgement. Do not deploy this file
workflow as an authenticated remote checkout protocol.

The emulated APDU pipe commands above connect the real native receiver to this
product path. The existing private PC/SC lab still runs its separate synthetic
fixture scenario; it has not been changed into a live-testnet orchestrator.
No full live run, automatic live PC/SC orchestration, production refunds, UI,
HD keystore, packaging or hardware qualification is claimed.
