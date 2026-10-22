# wow-nx

[benilla](https://github.com/samwhosung/benilla), a World of Warcraft 1.12.1 client written in Rust and Bevy, ported to
the Nintendo Switch. It draws through wgpu on a deko3d backend, the Switch's native GPU API (no Vulkan).

> [!NOTE]
> You need your own 1.12.1 client for the game data. This repository contains no Blizzard content.

## Status

Not playable yet. On the console so far:

- Rust's standard library runs (threads, files, clock, random): `switch/hello`.
- wgpu clears the screen through deko3d: `switch/clear`.
- Next: shaders and draws, then Bevy, then the game. Plan: [docs/SWITCH.md](docs/SWITCH.md),
  [docs/SWITCH-RENDER.md](docs/SWITCH-RENDER.md).

## Build

Needs devkitPro (devkitA64, libnx, switch-tools) and the Rust nightly in `switch/hello/rust-toolchain.toml` with
`rust-src`. Then `sh switch/hello/build.sh` or `sh switch/clear/build.sh`; each writes an NRO under its `target/`.
Copy it and your client's `Data` folder to `sdmc:/wow-nx/`.

The PC client is upstream's: [README.benilla.md](README.benilla.md).

## What nx-mod changed

- A Rust std target for the Switch (`aarch64-nintendo-switch-newlib`, `switch/`), linked with libnx.
- `third_party/libc`: the libc crate's Horizon types fixed for 64-bit (they were the 3DS's).
- `third_party/wgpu-hal`: a deko3d backend in wgpu's GL slot (device, swapchain, clears, copies; shaders in progress).
  `third_party/wgpu` and `wgpu-core` only turn that slot on for the Switch.
- `crates/benilla-uam`: uam, deko3d's shader compiler, built into the NRO; compiled shaders are cached on the SD card.
- `crates/benilla-horizon`: random numbers and `sysconf` from libnx.
- `main` mirrors upstream; this work is on `switch`.

## Credits

- **[samwhosung](https://github.com/samwhosung/benilla)** and contributors: benilla.
- **[gfx-rs](https://github.com/gfx-rs/wgpu)**: wgpu and naga.
- **[devkitPro](https://github.com/devkitPro/uam)**: uam, deko3d, devkitA64; **[switchbrew](https://github.com/switchbrew/libnx)**: libnx.
- **[centollOS](https://github.com/centollOS/SwitchWakerHD)**: the uam patches.
- **[rust-lang](https://github.com/rust-lang/libc)**: the libc crate.

## Legal

Unofficial fan project, not affiliated with Blizzard Entertainment or Nintendo. World of Warcraft is a trademark of
Blizzard Entertainment. Code is under MIT or Apache 2.0, as upstream; uam is under zlib and MIT. Running homebrew
needs custom firmware, which breaks Nintendo's terms and can get a console banned.
