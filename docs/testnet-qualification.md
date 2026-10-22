# Testnet qualification and fresh-wallet prerequisites

The original default-asset result is retained below. On 22 October 2026, the
public testnet node accepted native Rust gRPC reads but
rejected the existing USDC gasless template. **Do not proceed to submission.**
No keys, faucet requests, signatures, funding transactions or broadcasts were
used. Public diagnostic addresses `0x0` and `0x1` are not a funded demo wallet.

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

The alternate policy is confined to read-only qualification. Invoice terms carry
the exact chain/type and are checked in the chain core before template building;
the template is reconstructed exactly with the same explicit policy. These
diagnostic terms are not signed merchant invoices. `prepare-draft`, the
development signer, receipt verification, merchant trust pins, draft v0 and the
offline demo retain their existing behavior and reject this alternate asset.
There is no runnable end-to-end testnet checkout in this increment.

## Fresh disposable wallet setup (human-operated, not performed by the agent)

1. Provision fresh disposable **testnet-only** sender and receiver using an
   independently reviewed native wallet's local workflow. Do this yourself in a
   private local session with recording, shell tracing and agent access disabled.
   Keep keys/mnemonics in that wallet's local storage, never in command arguments,
   shell history, logs, screenshots, this repository or chat. Share only public
   addresses. Do not reuse production keys or the public fixture keys. This
   repository has no safe key-provisioning command or production custody claim;
   if you lack a reviewed local workflow, **stop here** rather than improvise one.
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
   or establish a fresh-wallet procedure. **Linux Tap has no conversion command or
   submission path.** If conversion is required, stop: separately reviewed and
   authorized human-operated conversion/submission is a blocker, not an implicit
   part of these instructions. No blanket SUI requirement is added to gasless
   transfers; this increment does not qualify conversion funding/gas prerequisites.
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
   not authority to sign/broadcast. Fresh-wallet validation has **not** been run.
   A later live trial needs separate submission authorization, reconciliation,
   independently trusted merchant pins and actual checkpoint evidence. Physical
   NFC and trusted hardware approval remain untested.
