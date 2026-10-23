#!/bin/sh
# Builds wow-nx-cube.nro: Bevy's 3D renderer on the Switch (docs/SWITCH-RENDER.md).
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
. "$ROOT/switch/env.sh"
cd "$ROOT/switch/cube"
cargo build --release $BUILD_STD "$@"
T=/opt/devkitpro/tools/bin
# An NRO without an asset section (no --nacp) is refused at load by Sphaira/hbl (2026-10-20).
$T/nacptool --create "wow-nx cube" "nx-mod" "0.1.0" target/cube.nacp
$T/elf2nro target/aarch64-nintendo-switch-newlib/release/cube.elf target/wow-nx-cube.nro --nacp=target/cube.nacp
echo "target/wow-nx-cube.nro"
