#!/bin/sh
# Builds uam (crates/benilla-uam over third_party/uam) for the Switch on its own: the Mesa compile is the slow part.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/switch/env.sh"
cd "$ROOT"
cargo build -p benilla-uam --lib --release $BUILD_STD "$@"
