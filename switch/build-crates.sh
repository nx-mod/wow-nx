#!/bin/sh
# Builds benilla's crates that do not draw for the Switch target (docs/SWITCH.md, step 2). Arguments go to cargo.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/switch/env.sh"
cd "$ROOT"
P=""
for c in benilla-bytes benilla-dbc benilla-blp benilla-adt benilla-m2 benilla-wmo benilla-wdt benilla-formats \
         benilla-protocol benilla-srp benilla-lua benilla-mlua benilla-mpq benilla-horizon benilla-uam; do
  P="$P -p $c"
done
cargo build $P --lib --release $BUILD_STD "$@"
