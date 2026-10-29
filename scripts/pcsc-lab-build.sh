#!/usr/bin/env bash
# Explicit opt-in dependency build. No host install, services, USB or network listeners.
set -euo pipefail
umask 077
root=$(cd "$(dirname "$0")/.." && pwd)
cache=${1:?Usage: bash scripts/pcsc-lab-build.sh ABSOLUTE_PRIVATE_CACHE}
[[ $cache = /* && $cache != *[[:space:]:]* && ${#cache} -lt 65 ]]
mkdir -p "$cache"
[[ -d $cache && ! -L $cache && $(stat -c %u "$cache") = "$(id -u)" && $(stat -c %a "$cache") = 700 ]]
cd "$cache"
exec 9>build.lock
flock -n 9
# Do not rebuild underneath a running lab.
mkdir -p prefix/run prefix/etc prefix/lib prefix/bin
exec 8>prefix/run/lab.lock
flock -n 8
[[ ! -e prefix/run/pcscd.comm && ! -e prefix/run/pcscd.pid ]]
fetch() {
    local file=$1 hash=$2 url=$3
    if [[ ! -f $file ]]; then curl --fail --location --max-time 90 "$url" -o "$file"; fi
    printf '%s  %s\n' "$hash" "$file" | sha256sum -c -
}
fetch pcsc-lite.tar.xz cdff7d7153a0b37aa74e26dfec89ec7dc5c5286aa21b91b903e38739d227e8e7 \
    https://deb.debian.org/debian/pool/main/p/pcsc-lite/pcsc-lite_2.3.3.orig.tar.xz
fetch vsmartcard.tar.gz 9f55b62246e745d87e5ce21b6c83e135773906cfa2c9fd02d6b00ff178c55763 \
    https://codeload.github.com/frankmorgner/vsmartcard/tar.gz/93286a90d2411120ae17299f9902e1fdd8954185
fetch meson.deb e92d04e60e784336883d537755d09b7dcb856809fa318a0931df3898e84ded95 \
    https://deb.debian.org/debian/pool/main/m/meson/meson_1.7.0-1_all.deb
# Extract only to this private cache. Package maintainer scripts are never executed.
dpkg-deb -x meson.deb meson-local
tar -xf pcsc-lite.tar.xz
tar -xf vsmartcard.tar.gz
export PYTHONPATH="$cache/meson-local/usr/lib/python3/dist-packages"
meson=(python3 "$cache/meson-local/usr/bin/meson")
setup=()
[[ ! -e pcsc-build/build.ninja ]] || setup=(--reconfigure)
"${meson[@]}" setup "${setup[@]}" pcsc-build pcsc-lite-2.3.3 \
    --prefix="$cache/prefix" --libdir=lib \
    -Dlibsystemd=false -Dlibudev=false -Dlibusb=false -Dpolkit=false -Dusb=false \
    -Dserial=true -Dserialconfdir="$cache/prefix/etc" -Dipcdir="$cache/prefix/run" \
    -Dusbdropdir="$cache/prefix/unused-usb" >pcsc-setup.log 2>&1
ninja -C pcsc-build >pcsc-build.log 2>&1
"${meson[@]}" install -C pcsc-build >pcsc-install.log 2>&1
source="$cache/vsmartcard-93286a90d2411120ae17299f9902e1fdd8954185"
patch --batch --fuzz=0 -d "$source" -p1 -i "$root/scripts/pcsc-lab-vpcd.patch" >vpcd-patch.log
cc -shared -fPIC -D_GNU_SOURCE -DVPCDSLOTS=1 -DHAVE_ARPA_INET_H=1 \
    -DHAVE_DECL_MSG_NOSIGNAL=1 -I"$cache/prefix/include/PCSC" \
    -I"$source/virtualsmartcard/src/vpcd" \
    "$source/virtualsmartcard/src/ifd-vpcd/ifd-vpcd.c" \
    "$source/virtualsmartcard/src/vpcd/vpcd.c" "$source/virtualsmartcard/src/vpcd/lock.c" \
    -o prefix/lib/libifdvpcd.so >vpcd-build.log 2>&1
# Preserve licenses and complete corresponding sources in the cache; ship only our patch.
cp "$root/scripts/pcsc-lab-vpcd.patch" vpcd-private.patch
sha256sum pcsc-lite.tar.xz vsmartcard.tar.gz meson.deb vpcd-private.patch \
    pcsc-build/config.h pcsc-build/pcscd.h \
    prefix/sbin/pcscd prefix/lib/libpcsclite.so.1 prefix/lib/libpcsclite_real.so.1 \
    prefix/lib/libifdvpcd.so >build.sha256
# Keep all extracted files private, with executable bits only for executable inputs.
find "$cache" -type d -exec chmod 700 {} +
find "$cache" -type f -perm /111 -exec chmod 700 {} +
find "$cache" -type f ! -perm /111 -exec chmod 600 {} +
printf 'Built private PC/SC 2.3.3 + Unix-only single-slot vpcd. No runtime started.\n'
printf 'Next (offline): PKG_CONFIG_PATH=%q cargo build --locked -p tap-cli --features pcsc-reader\n' "$cache/prefix/lib/pkgconfig"
