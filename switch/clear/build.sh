#!/bin/sh
# Builds the wgpu-on-deko3d test for the Switch (docs/SWITCH-RENDER.md).
set -e
RAIZ=$(cd "$(dirname "$0")/../.." && pwd)
. "$RAIZ/switch/entorno.sh"
cd "$RAIZ/switch/clear"
cargo build --release $BUILD_STD "$@"
