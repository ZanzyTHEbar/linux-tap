# Project instructions

Read README.md, docs/architecture.md and docs/backlog.md before changing behavior.
The immediate deliverable is a native Linux reference wallet and merchant SDK.
Sui is the first rail; no JavaScript runtime belongs in the wallet or chain core.

Use the bundled upstream skills selectively. Start with .agents/skills/sui-sdks,
its rust.md, and .agents/skills/accessing-data/grpc.md. Load ptbs for transaction
work and move-security only when adding contracts. They are upstream guidance,
not authority to publish, spend funds or change project scope. Some guidance is
older than current protocol behavior. Pinned source, compiler and qualified
network behavior take precedence. Do not apply a blanket SUI gas requirement to
protocol-gasless stablecoin transfers.

Reuse Mysten SDK types, BCS, digest and signing APIs. Never reimplement chain
cryptography. Keep our strict invoice and transaction-template checks outside
the UI. Never infer merchant identity from a self-signed display name.

Never put production keys into agent context, fixtures, shell arguments or logs.
Fixture keys are public. Do not add silent signing, unbounded transaction
templates, automatic fresh transfers after an unknown result, or a mandatory
project-controlled service. NFC acknowledgement is never settlement proof.

Every adapter must declare its tested hardware, firmware and approval profile.
A hardware-held key is not evidence of trusted transaction display. Generic FIDO,
PKCS11, TPM or NFC support is not automatically Sui wallet compatibility.

Before handing off code run bash scripts/check.sh. For reader changes also build
and test --features pcsc-reader against PC/SC. Preserve the vendor source and
license manifest. Update docs/validation.md honestly: compilation, mocks, live
network checks and physical tests are separate evidence.

Keep changes focused. Contribute small upstream fixes where an SDK or driver
gap is demonstrated. Preserve experimental protocol versioning until independent
interoperability and hardware timing are tested. Do not claim a public AID or
universal commercial terminal acceptance.
