#!/bin/sh
# Builds wow-nx-hello.nro: Rust std for the Switch (aarch64-nintendo-switch-newlib.json), linked with libnx.
# Needs the nightly in rust-toolchain.toml with rust-src, and devkitPro's devkitA64, libnx and switch-tools.
set -e
cd "$(dirname "$0")"
cargo build --release -Zbuild-std=std,panic_abort -Zjson-target-spec --target aarch64-nintendo-switch-newlib.json
T=/opt/devkitpro/tools/bin
$T/nacptool --create "wow-nx hello" "nx-mod" "0.1.0" target/hello.nacp
$T/elf2nro target/aarch64-nintendo-switch-newlib/release/hello.elf target/wow-nx-hello.nro --nacp=target/hello.nacp
echo "target/wow-nx-hello.nro"
