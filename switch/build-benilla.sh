#!/bin/sh
# Builds wow-nx.nro: benilla's player build for the Switch (the `switch` profile, Cargo.toml).
# Copy it to sdmc:/wow-nx/ with your 1.12.1 client's Data folder at sdmc:/wow-nx/Data.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/switch/env.sh"
cd "$ROOT"
cargo build -p benilla --no-default-features --profile switch $BUILD_STD "$@"
T=/opt/devkitpro/tools/bin
# An NRO without an asset section (no --nacp) is refused at load by Sphaira/hbl (2026-10-20).
$T/nacptool --create "wow-nx" "nx-mod" "0.1.0" target/wow-nx.nacp
# switch/icon.jpg (256x256 baseline JPEG, made from your client's art with switch/blpdump; not in the repo): the
# NRO's icon, which forwarders take.
ICON=""
[ -f "$ROOT/switch/icon.jpg" ] && ICON="--icon=$ROOT/switch/icon.jpg"
$T/elf2nro target/aarch64-nintendo-switch-newlib/switch/benilla.elf target/wow-nx.nro --nacp=target/wow-nx.nacp $ICON
echo "target/wow-nx.nro"
