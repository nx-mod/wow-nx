#!/bin/sh
# Sourced by the Switch build scripts: the toolchain, devkitPro's C compiler for crates that build C (mlua's Lua),
# and std's source with our libc (third_party/libc/WOW-NX.md). -Zbuild-std resolves std's dependencies from the
# toolchain's own library/Cargo.toml and Cargo.lock, not from our [patch]: a private copy of that source with libc
# pointed at ours, used through cargo's source-root override. The installed toolchain is not touched.
# ROOT (the repo) is set by the script that sources this.
: "${ROOT:?set ROOT to the wow-nx checkout before sourcing env.sh}"
export RUSTUP_TOOLCHAIN=nightly-2026-07-02
TARGET_SPEC="$ROOT/switch/hello/aarch64-nintendo-switch-newlib.json"
A64=/opt/devkitpro/devkitA64/bin
export CC_aarch64_nintendo_switch_newlib=$A64/aarch64-none-elf-gcc
export AR_aarch64_nintendo_switch_newlib=$A64/aarch64-none-elf-ar
export CFLAGS_aarch64_nintendo_switch_newlib="-march=armv8-a+crc+crypto -fPIE -D__SWITCH__ -I/opt/devkitpro/libnx/include"
export CXX_aarch64_nintendo_switch_newlib=$A64/aarch64-none-elf-g++
export CXXFLAGS_aarch64_nintendo_switch_newlib="-march=armv8-a+crc+crypto -fPIE -D__SWITCH__ -I/opt/devkitpro/libnx/include"
SRC="$ROOT/target/switch-rust-src/library"
if [ ! -f "$SRC/.wow-nx-libc" ]; then
  rm -rf "$ROOT/target/switch-rust-src" && mkdir -p "$ROOT/target/switch-rust-src"
  cp -r "$(rustc --print sysroot)/lib/rustlib/src/rust/library" "$ROOT/target/switch-rust-src/"
  printf '\n# wow-nx: devkitA64 newlib types for aarch64 Horizon\nlibc = { path = "%s" }\n' "$ROOT/third_party/libc" >> "$SRC/Cargo.toml"
  python3 -I - "$SRC/Cargo.lock" <<'PY'
import re, sys
p = sys.argv[1]; s = open(p).read()
s, n = re.subn(r'name = "libc"\nversion = "[^"]+"\nsource = "[^"]+"\nchecksum = "[^"]+"\n', 'name = "libc"\nversion = "0.2.186"\n', s)
assert n == 1, "libc entry not found in std's Cargo.lock"
open(p, 'w').write(s)
PY
  touch "$SRC/.wow-nx-libc"
fi
export __CARGO_TESTS_ONLY_SRC_ROOT="$SRC"
# getrandom 0.3/0.4 do not know aarch64 Horizon: their custom backend, defined in crates/benilla-horizon.
# Frame pointers: Atmosphere's crash reports walk them, and without them a report stops after a few frames.
export CARGO_TARGET_AARCH64_NINTENDO_SWITCH_NEWLIB_RUSTFLAGS='--cfg getrandom_backend="custom" -C force-frame-pointers=yes'
BUILD_STD="-Zbuild-std=std,panic_abort -Zjson-target-spec --target $TARGET_SPEC"
