#!/bin/sh
# Builds wow-nx-cubo.nro: Bevy's 3D renderer on the Switch (docs/SWITCH-RENDER.md).
set -e
RAIZ=$(cd "$(dirname "$0")/../.." && pwd)
. "$RAIZ/switch/entorno.sh"
cd "$RAIZ/switch/cubo"
cargo build --release $BUILD_STD "$@"
T=/opt/devkitpro/tools/bin
# An NRO without an asset section (no --nacp) is refused at load by Sphaira/hbl (2026-10-20).
$T/nacptool --create "wow-nx cubo" "nx-mod" "0.1.0" target/cubo.nacp
$T/elf2nro target/aarch64-nintendo-switch-newlib/release/cubo.elf target/wow-nx-cubo.nro --nacp=target/cubo.nacp
echo "target/wow-nx-cubo.nro"
