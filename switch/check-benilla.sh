#!/bin/sh
# Type-checks the player build of benilla for the Switch (no codegen): the list of what still needs a Switch path.
set -e
RAIZ=$(cd "$(dirname "$0")/.." && pwd)
. "$RAIZ/switch/entorno.sh"
cd "$RAIZ"
cargo check -p benilla --no-default-features $BUILD_STD "$@"
