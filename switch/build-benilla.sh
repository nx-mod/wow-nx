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
# A release (switch/release.sh) passes WOWNX_BUILD_NUMBER, which the self-updater compares with GitHub's latest.
VERSION=${WOWNX_BUILD_NUMBER:+build $WOWNX_BUILD_NUMBER}
$T/nacptool --create "wow-nx" "nx-mod" "${VERSION:-dev}" target/wow-nx.nacp
$T/elf2nro target/aarch64-nintendo-switch-newlib/switch/benilla.elf target/wow-nx.nro --nacp=target/wow-nx.nacp
# Which build target/wow-nx.nro is, for switch/release.sh; none for a build without a number.
printf '%s\n' "${WOWNX_BUILD_NUMBER:-0}" > target/wow-nx.build
echo "target/wow-nx.nro"
