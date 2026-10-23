#!/bin/sh
# Builds the vendored wgpu-hal with its deko3d backend for the Switch (docs/SWITCH-RENDER.md). Arguments go to cargo.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/switch/env.sh"
cd "$ROOT"
cargo build -p wgpu-hal --features gles --lib --release $BUILD_STD "$@"
