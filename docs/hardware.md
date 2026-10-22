# Hardware qualification

The delivered PC/SC client compiles against system PC/SC. The nfcd development
bridge imports successfully and is tested through its Rust subprocess. No reader,
phone, secure element, firmware or RF timing has been physically tested here.

## Initial bench

Use the existing Poco X3 Pro as an Android HCE test harness when useful. It does
not establish a native Linux phone port. The current bootstrap does not contain
an Android APK. Keep MIUI banking and wallet use separate from lab experiments.

For a Linux merchant use a suitable PC/SC reader, with the ACS ACR1552U as the
candidate from the earlier hardware plan. Record the driver actually required,
firmware, supported protocol and reader name. A reader advertised with card
emulation does not necessarily expose arbitrary host-controlled APDUs.

For native Linux listen mode use the PN7160 branch recorded in upstreams.lock.json
and NXP's Linux guide. It is a hardware qualification path. Running this vendor
library and running nfcd are different integrations. Do not assume nfcd has a
working PN7160 adapter merely because both projects are open source. First prove
vendor card emulation with its tools. Separately qualify an nfcd adapter or add
a narrowly scoped Rust transport bridge to the vendor interface.

## nfcd stepping stone

On a Linux device where nfcd and its hardware plugin already work:

```sh
python3 -m venv .venv
.venv/bin/pip install -r adapters/nfcd/requirements.txt
cargo build --locked -p tap-cli
.venv/bin/python adapters/nfcd/bridge.py \
  --binary target/debug/linux-tap --out /PRIVATE/DIRECTORY/unverified-invoice.json
```

Choose an existing private directory for the output and use a fresh filename for
each unrelated test. Registration needs the device's D-Bus policy. The script
does not install broad permissions or run nfcd as root. The upstream daemon
handles RF and application selection; the bridge exports its LocalHostApp and
requests card-emulation mode. It receives no customer keys.

The bridge is a development stepping stone. Its D-Bus registration, hardware
callbacks and physical timing still need to be tested on a real qualified device.
The supplied bridge test checks method signatures and APDU reconstruction using
OS pipes, not a live D-Bus service or NFCC.

## Hardware signer selection

PaymentSigner is the integration boundary; only the development software signer
is implemented. The initial qualification must prove algorithm compatibility,
native Linux transport, non-exportable key generation, signature encoding and
recovery. Use existing hardware apps or standards-based interfaces before
writing firmware. Generic PKCS11/FIDO/TPM capability is not evidence that the
device understands Sui transactions.

LedgerHQ/app-sui is an Apache-2.0 comparison target. Its actual device OS and
secure element remain vendor dependencies. Pin and inspect the corresponding
host transport before an adapter; do not flash a personal wallet to test this
bootstrap. Other open hardware signers can be qualified through the same
interface without a Google or initiative-owned attestation root.

Declare one of these exact claims for each implementation: development software;
hardware key with host approval; or trusted on-device transaction approval. A
touch or fingerprint authorizing an opaque digest does not establish the last.

## Evidence to record

Model and regional variant; board revision; NFCC and signer identity; firmware;
kernel and driver; source revision; bus and permissions; APDU trace; field-loss
recovery; approval profile; signature fixture; and complete checkout results.
Do not convert phone-brand sales or vendor revenue into chipset market share.

Sources: https://www.nxp.com/docs/en/application-note/AN13287.pdf and
https://github.com/sailfishos/nfcd and https://github.com/LedgerHQ/app-sui
