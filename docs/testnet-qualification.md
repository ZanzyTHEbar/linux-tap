# Testnet qualification: blocked on the pinned asset

On 22 October 2026, the public testnet node accepted native Rust gRPC reads but
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

The command always uses `tap_core::USDC` and `build_direct`: a sender withdrawal,
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

Do not fix this by requiring SUI or changing the asset silently. A separately
reviewed network-specific asset policy and explicitly approved disposable
testnet funding/qualification experiment are needed to revisit the gate.
The observed alternate type is evidence, not authorization to enable it.
Submission/reconciliation code and any specific testnet broadcast remain gated.
The recording demo stays an offline simulation with no physical NFC claim.

The node identified its build as `1.82.2-ecf82404d44d`. The corresponding
[upstream protocol source](https://github.com/MystenLabs/sui/blob/ecf82404d44d/crates/sui-protocol-config/src/lib.rs)
independently names these `MAINNET_USDC` and `TESTNET_USDC`, and defines the
allowlist entries as token types with minimum transfer sizes. That confirms the
network-specific mismatch; it does not establish that an alternate asset or a
funded transaction would simulate successfully. The SDK remains pinned at
`1313d5112464ecc25d7407f42442adea08589fc5`; no vendor code was changed.
