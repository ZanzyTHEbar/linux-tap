# Security status

This bootstrap is not an audited wallet. Demo keys are deliberately public; the
demo chain is synthetic. Do not send funds to fixture addresses. No broadcast
command is implemented. Development key files are not a production keystore.

The current protection mechanisms are strict bounded invoice parsing, explicit
merchant key pins, local asset/network policy, canonical bytes, domain-separated
merchant signatures, exact transaction reconstruction and durable signed-byte
journals. Tests exercise several failure cases but are not a security audit.

The merchant's real-node verifier trusts its selected full node for execution
and checkpoint inclusion. It does not independently verify validator consensus.
The hardware signer boundary has only a development implementation. The nfcd
bridge has no spending keys and stores unverified requests only.

Before a customer-money pilot: qualify actual chain behavior and hardware,
isolate custody, test recovery, implement full reconciliation and refunds, and
commission an independent review of signing and settlement verification. Keep
the laboratory APDU identifier private until a public identifier and stable
protocol are agreed.

No disclosure mailbox or hosted repository has been established by this local
bootstrap. The maintainer must establish a private reporting route before a
public pilot. Never include seeds, private keys or personal transaction traces
in a public issue.
