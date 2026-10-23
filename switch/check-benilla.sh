#!/bin/sh
# Type-checks the player build of benilla for the Switch (no codegen): the list of what still needs a Switch path.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/switch/env.sh"
cd "$ROOT"
cargo check -p benilla --no-default-features $BUILD_STD "$@"
