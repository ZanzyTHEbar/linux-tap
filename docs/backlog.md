# Ordered implementation backlog

Each row is one reviewable increment. Complete the evidence before claiming the
corresponding capability. The first slice is deliberately smaller than the
funded multi-person programme.

| ID | Task | Completion evidence | State |
| --- | --- | --- | --- |
| B01 | Pin Rust and official SDK | Build and checked lockfile | Done |
| B02 | Import selected official agent skills | Unmodified files and SHA256 manifest | Done |
| B03 | Implement signed invoice draft | CBOR, signature, pinning and rejection tests | Done |
| B04 | Implement narrow Sui transaction template | Official SDK BCS/signatures and exact reconstruction tests | Offline only |
| B05 | Persist transaction before sending | Reopen and reject replacement tests | Done for simulator |
| B06 | Demonstrate two-process checkout | Interrupted wallet and restarted merchant yield one simulated receipt | Done over pipes |
| B07 | Add merchant reader transport | Optional PC/SC build and transmit command | Compiled only |
| B08 | Reuse Linux card-emulation interface | nfcd bridge and upstream XML | Software bridge only |
| B09 | Test Unix socket transport on your Omarchy host | Same smoke scenario over real private sockets | Linux socket demo verified; Omarchy host pending |
| B10 | Qualify actual Sui gRPC access | Saved genesis identity, epoch and endpoint policy | Read-only public testnet verified; see testnet-qualification.md |
| B11 | Qualify gasless native USDC shape | Live dry-run, live protocol configuration and address-balance funding evidence | Blocked: pinned asset absent/not gasless-allowed on testnet; checked simulation rejected |
| B12 | Propose offline-builder fix upstream | Minimal reproducer and reviewed patch | Reproducer done; not posted |
| B13 | Add capability-driven preparation | Wrong network/protocol/asset fails before signing | Read-only qualification gate added; signing integration still open |
| B14 | Add controlled submission and reconciliation | Persisted exact bytes, no replacement on timeout, successful checkpoint verification | Open |
| B15 | Add fresh merchant receiving custody | Unique per invoice, watch-only acceptance and tested backups | Open |
| B16 | Test real PC/SC reader | Reader inventory and successful APDU traces | Open |
| B17 | Prove native Linux NFCC listen mode | Exact board, driver, firmware and vendor test trace | Open |
| B18 | Connect native NFCC to our receiver | Complete invoice transfer, field-loss recovery and timing | Open |
| B19 | Qualify first hardware signer | Native signature and key-policy fixtures | Open |
| B20 | Implement recovery and custody isolation | Lost-device exercise and explicit approval profile | Open |
| B21 | Add Qt Quick confirmation | Real invoice, exact terms and explicit signer invocation through CXX-Qt | Selected; not implemented |
| B22 | Add second asset through a reviewed manifest | Full type, decimals, issuer controls and actual eligibility | Open |
| B23 | Implement live refund and reconciliation | Linked refund and late/overpayment ledger | Open |
| B24 | Integrate one willing POS | Deployment permission and complete real checkout | Open |
| B25 | Obtain independent security review | Closed or explicitly blocking findings | Open |
| B26 | Run limited merchant pilot | Every attempt and funds movement accounted for | Open |
| B27 | Build independent second wallet | Common golden vectors and cross-implementation checkout | Open |
| B28 | Package Omarchy integration | Install, permission, removal and update instructions | Open |

Begin locally with the README build and smoke command, then B09 and B10. B11 is
the next chain blocker: do not spend a week building UI while gasless source
funds and template eligibility remain unproven. B17 can proceed independently
once the board is available; do not rewrite a phone driver first.

The [recording-friendly demo](demo.md) exercises private sockets, explicit
terminal approval and exact-byte replay without submitting a transaction. It
does not close B10–B28 or substitute for finalized production requirements.

Use tests or a development network for destructive fault injection. A live
mainnet qualification is a separate deliberately funded experiment with fresh
accounts after review, not a default script step. No production money, external
messages, upstream issues or repositories were created by this bootstrap.
