# ADR 0002 Offline gasless construction

Status: experimental adapter, upstream issue candidate. No issue has been posted.

The SDK is pinned to 1313d5112464ecc25d7407f42442adea08589fc5.
In crates/sui-transaction-builder/src/builder.rs, try_build returns
MissingGasObjects when the gas-object set is empty, before checking price or
budget. The bootstrap's capture_offline_builder_gap test reproduces this.

Do not add a fake gas object to a signed gasless transaction. For the experimental
USDC address-balance template, construct a Transaction using official SDK types
and BCS. Its only calls are balance::redeem_funds and balance::send_funds. Use
empty gas objects, zero price and budget, and ValidDuring with chain identity,
the explicit epoch and a journaled nonce. Reject any deviation by rebuilding
and comparing the exact template.

This proves offline construction and signing only. It does not prove balance
availability, protocol-version eligibility, live simulation success or terminal
acceptance. The current adapter assumes source funds are already address
balances. It does not select, consume or convert coin objects.

Next action: qualify this shape against a development network that supports the
feature, add an explicit capabilities check, and propose a minimal upstream
offline-builder improvement. No local SDK fork is required for the bootstrap.
Replace our narrow construction adapter when the official builder supports the
qualified shape. Do not migrate through deprecated JSON-RPC or a cloud signer.

Reference: https://docs.sui.io/develop/transaction-payment/gasless-stablecoin-transfers
