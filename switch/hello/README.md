# Rust std on the Switch

Step 1 of [the port plan](../../docs/SWITCH.md): Rust's standard library on Horizon, before any of benilla.

`aarch64-nintendo-switch-newlib.json` is the Switch target with `std`: `target_os = "horizon"` on devkitA64's newlib (the
path `std` already has for the 3DS), linked by devkitA64's gcc with libnx's `switch.specs`. `src/horizon.rs` gives `std`
the two C functions newlib lacks (`getrandom`, `sysconf`) from libnx. libnx and newlib both define `fcntl`; the link keeps
the first (libnx's), and the program links libnx itself so it comes first.

`./build.sh` makes `target/wow-nx-hello.nro`. On the console it prints its checks and writes them to
`sdmc:/wow-nx/hello.txt`: allocation, a thread, a HashMap (std's random seed), a file written and read back.

Status: builds and links (2026-10-22); not run on the console yet.
