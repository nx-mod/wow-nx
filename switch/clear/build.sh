#!/bin/sh
# Builds wow-nx-clear.nro, the wgpu-on-deko3d screen clear (docs/SWITCH-RENDER.md).
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
. "$ROOT/switch/env.sh"
cd "$ROOT/switch/clear"
# deko3d's debug build and a check after every submission (third_party/wgpu-hal/build.rs).
export WOWNX_DEKO3D_DEBUG=1
cargo build --release $BUILD_STD "$@"
T=/opt/devkitpro/tools/bin
# An NRO without an asset section (no --nacp) is refused at load by Sphaira/hbl (2026-10-20).
$T/nacptool --create "wow-nx clear" "nx-mod" "0.1.0" target/clear.nacp
$T/elf2nro target/aarch64-nintendo-switch-newlib/release/clear.elf target/wow-nx-clear.nro --nacp=target/clear.nacp
echo "target/wow-nx-clear.nro"
