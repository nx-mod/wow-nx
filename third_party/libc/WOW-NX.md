# libc 0.2.186 for wow-nx

Upstream `libc` 0.2.186 (crates.io), with the types devkitA64's newlib has on the Switch (aarch64 Horizon): `dev_t`,
`ino_t`, `off_t`, `blkcnt_t`, `blksize_t`, `suseconds_t` in `src/unix/newlib/mod.rs`, under
`all(target_os = "horizon", target_arch = "aarch64")`. The crate's Horizon support was written for the 32-bit 3DS: on
the Switch its `struct stat` read the file size from the wrong offset. `switch/hello/src/layouts.rs` checks every layout
at compile time against sizes measured with devkitA64's gcc. Used by `[patch.crates-io]`, including `std`'s own copy
under `-Zbuild-std`.
