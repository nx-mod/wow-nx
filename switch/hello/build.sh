#!/bin/sh
# Builds wow-nx-hello.nro: Rust std for the Switch (aarch64-nintendo-switch-newlib.json), linked with libnx.
# Needs the nightly in rust-toolchain.toml with rust-src, and devkitPro's devkitA64, libnx and switch-tools.
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
. "$ROOT/switch/env.sh"
cd "$ROOT/switch/hello"
cargo build --release $BUILD_STD
T=/opt/devkitpro/tools/bin
# An NRO without an asset section (no --nacp) is refused at load by Sphaira/hbl, Rust or C (2026-10-20).
$T/nacptool --create "wow-nx hello" "nx-mod" "0.1.0" target/hello.nacp
$T/elf2nro target/aarch64-nintendo-switch-newlib/release/hello.elf target/wow-nx-hello.nro --nacp=target/hello.nacp
echo "target/wow-nx-hello.nro"
