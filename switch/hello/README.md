# Rust std on the Switch

Step 1 of [the port plan](../../docs/SWITCH.md): Rust's standard library on Horizon, before any of benilla.

`aarch64-nintendo-switch-newlib.json` is the Switch target with `std`: `target_os = "horizon"` on devkitA64's newlib (the
path `std` already has for the 3DS), linked by devkitA64's gcc with libnx's `switch.specs`. `crates/benilla-horizon` gives
`std` the two C functions newlib lacks (`getrandom`, `sysconf`) from libnx. libnx and newlib both define `fcntl`; the link keeps
the first (libnx's), and the program links libnx itself so it comes first.

`./build.sh` makes `target/wow-nx-hello.nro` (`../env.sh` sets the toolchain up; `../build-crates.sh` builds
benilla's crates that do not draw, the same way).

**libc:** the `libc` crate's Horizon support was written for the 32-bit 3DS. On the Switch its `struct stat` read the
file size at the wrong offset, and its pthread condition attribute was half the size devkitA64 writes into it.
`third_party/libc` fixes the types for aarch64 Horizon; `src/layouts.rs` checks every size, alignment and offset at compile
time against devkitA64's gcc. std gets the same libc through a patched copy of its source (`../env.sh`), since
`-Zbuild-std` ignores the workspace's `[patch]`. On the console it prints its checks and writes them to
`sdmc:/wow-nx/hello.txt`: allocation, a thread, a HashMap (std's random seed), a file written and read back.

Status: builds and links (2026-10-22); not run on the console yet.

**Why the earlier Rust NROs crashed (2026-10-20):** not Rust. Every NRO that crashed in Sphaira/hbl at load
(`hbl+0x4040`, a User Break) had **no asset section**: `elf2nro` was run without `--nacp`. That includes a pure C
`mini.nro`. Every NRO with an asset section ran, Rust included (`freerust.nro`). `build.sh` always passes `--nacp`.
`nrodoc` (installed) checks the libnx TLS ABI markers; this NRO has the current ones (LNY2 revision 1), like the C
ones that work.
