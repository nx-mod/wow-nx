# libc 0.2.186 for wow-nx

Upstream `libc` 0.2.186 (crates.io), with the types devkitA64's newlib has on the Switch (aarch64 Horizon): `dev_t`,
`ino_t`, `off_t`, `blkcnt_t`, `blksize_t`, `suseconds_t` in `src/unix/newlib/mod.rs`, under
`all(target_os = "horizon", target_arch = "aarch64")`. The crate's Horizon support was written for the 32-bit 3DS: on
the Switch its `struct stat` read the file size from the wrong offset. `switch/hello/src/layouts.rs` checks every layout
at compile time against sizes measured with devkitA64's gcc. Used by `[patch.crates-io]`, including `std`'s own copy
under `-Zbuild-std`.

The sockets too (`src/unix/newlib/horizon/mod.rs` and `mod.rs`, same cfg): the Switch's are libnx's, from FreeBSD,
with a length byte and a one-byte family in `sockaddr`, `sockaddr_in` and a 128-byte `sockaddr_storage`, and BSD's
`AF_INET6`, `FIONBIO`, `SO_ERROR`, `TCP_NODELAY`, `IP_*`, `MSG_*` and `EAI_*` values. With the 3DS layouts every
address `getaddrinfo` returned was read with its family in the wrong byte and skipped.

Editing this crate does not rebuild `std`: under `-Zbuild-std` cargo treats the standard library's sources as
immutable and keeps its compiled `libc` and `std`. After a change here, delete std's units' fingerprints
(`target/aarch64-nintendo-switch-newlib/switch/.fingerprint/libc-*` of the `std` build and `std-*`, or all
`libc-*`/`std-*`), then build: the log must show `Compiling std`.
