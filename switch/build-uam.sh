#!/bin/sh
# Builds uam (crates/benilla-uam over third_party/uam) for the Switch on its own: the Mesa compile is the slow part.
set -e
RAIZ=$(cd "$(dirname "$0")/.." && pwd)
. "$RAIZ/switch/entorno.sh"
cd "$RAIZ"
cargo build -p benilla-uam --lib --release $BUILD_STD "$@"
