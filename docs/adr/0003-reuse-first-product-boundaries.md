# ADR 0003 — Reuse-first product boundaries

**Status: Proposed target design; not implemented.** 28 October 2026.
Baseline `f6ea6a8`; see the dated [independent review record](#independent-review-record).

## Context

Linux Tap already has three Rust crates, strict signed invoices, a narrow Sui
adapter, simulator journals and keyless NFC glue. Research found useful native
application and payment implementations, but no ready Linux Sui payment wallet
whose guarantees can be inherited by a fork. The immediate user-selected slice
is APDU-origin verification/approval and post-commit lost-receipt replay, not live
settlement. The named initiative and charter/engineering v0.2 remain unavailable.

Evidence: [research index](../research/README.md),
[phone implementation map](../research/linux-phone-implementation-map.md),
[payment implementation map](../research/payment-implementation-map.md) and
[security notes](../research/transport-security-notes.md).

## Decision

Keep the three crates and logical boundaries in [product architecture](../product-architecture.md).
Use [product design](../product-design.md) for conceptual records, approval binding,
single-owner issuance and crash barriers. No new services or stable SDK ABI yet.
Transport is keyless/untrusted; policy and exact candidate approval stay in Rust;
signing, immutable artifacts, dispatch and observations remain distinct effects.

Keep pinned Mysten SDK types/BCS/signing/gRPC and the narrow offline adapter until
a qualified official replacement preserves approved bytes. Later reuse the
execute/checkpoint helper with full receipt fields and a whole-operation deadline.
Keep strict v0 and private AID pending actual interoperability evidence.

Adapt LocalHostApp registration/owned-handle/cleanup patterns from native C/Qt
references on a qualified backend. Use external isolated vpcd for the later PC/SC
boundary; Casimir is optional. No shared daemon takeover or new controller stack.
Consider Gem MIT core keystore only behind `PaymentSigner` after custody review;
hardware key custody is not trusted display. Qt/CXX-Qt remains a thin future UI,
with QML JS capability and component licensing explicitly unresolved.

## Rejected alternatives

- Whole Gem/Electrum/Monero/Trust wallet fork: unrelated engines/UI and qualification
  burden; selective modules/interaction lessons are sufficient.
- Ten services, plugin registry or distributed leases: no demonstrated consumer
  need; one state-directory owner and DB constraints are smaller and safer.
- Node/JS chain core, cloud signing or mandatory foundation backend: conflicts
  with native local authority and adds trust without closing current gaps.
- NDEF/OCP/EMV substitution: different trust/transport/acceptance models, not the
  unidentified initiative or Linux Tap interoperability.
- Async builder mutation, custom crypto/BCS or blind signing: breaks exact approval
  or duplicates reviewed SDK work. A parser mismatch is a qualification blocker.
- Fresh signing after timeout: risks another valid payment. Recover/reconcile
  the same artifact; interrupted external issuance may require fail-closed handling.

## Consequences and reopening criteria

Stage A can proceed without hardware, production keys or RPC. Research is not
adoption; no capability states advance. Current journals lack durable approval,
pre-sign reservation and dispatch state; uniqueness after signing is insufficient
for a claim of one signer invocation. Power-loss, SQLite runtime/fix, custody and
merchant correlation still require qualification. See the design evidence gates.

Reopen for a concrete SDK consumer, identified interoperability specification,
qualified builder replacement, production custody model or literal no-JS UI
decision. Require exact source/license and evidence before adopting a module;
never inherit upstream outbox, refund, trusted-display or RF guarantees by name.

## Independent review record

28 October 2026: an independent read-only source/contract-consistency review
(`ses_e7d89d305ffe7gbGpov2ZmmLE5`) reported no blocking or material findings.
It reviewed these three proposed documents, navigation/research and relevant
local receiver, signer, store and checkout source; the architecture was judged
coherent and implementable within three crates, with current capabilities
narrower than the target. Single-credit uniqueness was understood across invoices
within each merchant operator's authoritative ledger, not a shared registry.
This was not a security audit, formal verification, runtime, compilation,
physical or power-loss/durability verification; none was performed or inferred.
The proposal remains unimplemented, not user-approved production design.
