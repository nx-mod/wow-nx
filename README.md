# wow-nx

[benilla](https://github.com/samwhosung/benilla), a World of Warcraft 1.12.1 client written in Rust and Bevy, ported to
the Nintendo Switch. It draws through wgpu on a deko3d backend, the Switch's native GPU API, and plays with the Joy-Con
through [benilla-pad](https://github.com/lucasvallejodev/benilla-gamepad).

<!-- screenshot: docs/images/switch.jpg -->

> [!NOTE]
> You need your own 1.12.1 client (build 5875) for the game data, and a 1.12.1 server to play on. This repository
> contains no Blizzard content.

## Status

In progress, and playable: it logs in to a 1.12.1 server (tested on Kronos), enters the world, and you can fight,
loot, cast and quest with the Joy-Con, touch, or a USB keyboard and mouse.

Known issues, being worked on ([docs/SWITCH.md](docs/SWITCH.md) has the details):

- **Speed:** below a steady 60 fps; the main thread is the limit and the work on it continues. Numbers come with the
  first tested release.
- **The latest builds can crash while the world loads** (a GPU timeout); a debug build is narrowing it down.
- **Some status bars** (health, cast bars) draw with chunks missing.
- The controller menu's descriptions still use Xbox button names.

## Install

1. Copy your 1.12.1 client's `Data` folder to `sdmc:/wow-nx/Data`.
2. Put `wow-nx.nro` in `sdmc:/wow-nx/` (or `sdmc:/switch/`) and start it from the homebrew menu. A release build looks
   for newer releases on GitHub while it loads and offers them; `WOWNX_UPDATE=0` turns that off.
3. Optional: `sdmc:/wow-nx/benilla.env` sets the server and login (below). Without it, type them on the login screen
   with the system keyboard (tap the text box).

Settings and saved state live in `sdmc:/wow-nx/benilla-config/`, as on the PC.

### benilla.env

`KEY=VALUE` lines, read at start:

| Key | Does |
|---|---|
| `WOW_HOST` | the logon server (`localhost` by default) |
| `WOW_USER`, `WOW_PASS` | log in without typing (kept in plain text on the card) |
| `WOWNX_FPS=1` | the fps overlay, top left |
| `WOWNX_UPDATE=0` | no update check |
| `WOWNX_PROFILE=1` | a sampling profiler: `samples.txt` every 30 s, named by `switch/profile.py` |
| `WOWNX_GPU_PIPELINES=1` | GPU time per pipeline in the log |

Vertical sync is the game's own Video option.

## Controls

With the Joy-Con (benilla-pad; its controller menu, −, rebinds every button on every layer):

| Button | In the world | In menus and windows |
|---|---|---|
| Left stick | move | |
| Right stick | camera (L or R held: zoom) | cursor |
| B | jump | back |
| A, Y, X | action slots 1, 2, 3 | A: confirm / click |
| D-pad | action slots 4–7 | move between buttons |
| ZL, ZR (held) | the other action layers | |
| L, R | target nearest friend, enemy | |
| Left stick press | autorun | |
| Right stick press | interact: talk, loot, skin, attack | |
| + | window wheel | |
| − | controller menu | |

Touch moves the cursor and clicks; a tap on a text box opens the system keyboard. A USB keyboard and mouse work as on
the PC.

## Build

Needs devkitPro (devkitA64, libnx, switch-tools) and the Rust nightly in `switch/env.sh` with `rust-src`. Then:

```sh
sh switch/build-benilla.sh -j2        # target/wow-nx.nro
```

`WOWNX_BUILD_NUMBER=<n>` numbers a release build, and `switch/release.sh <n> <notes>` publishes it as GitHub release
`build-<n>`, the one the updater takes. `WOWNX_DEKO3D_DEBUG=1` links deko3d's debug library and names the work in a
GPU fault.

The PC client is upstream's: [README.benilla.md](README.benilla.md).

## What nx-mod changed

- A Rust std target for the Switch (`aarch64-nintendo-switch-newlib`, `switch/`), linked with libnx;
  `third_party/libc` with Horizon's socket and type layouts fixed for 64-bit (they were the 3DS's).
- `third_party/wgpu-hal`: a deko3d backend in wgpu's GL slot, with naga → uam shader translation cached on the SD
  card, per-pass GPU timing, and upstream's Vulkan stall patch alongside. `third_party/wgpu` and `wgpu-core` turn that
  slot on for the Switch.
- `crates/benilla-uam`: uam, deko3d's shader compiler, built into the NRO.
- `crates/benilla-horizon`: what libnx gives Rust (random numbers, `sysconf`, sockets), a thread-caching allocator,
  threads spread over the cores, the launch picture, the system keyboard, the self-updater and a sampling profiler.
- `crates/benilla-app/src/switch.rs`: the window, the applet loop, HID input (the Joy-Con as a gamepad), the SD card
  paths and `benilla.env`.
- `crates/benilla-pad`: lucasvallejodev's controller crate, made to work with today's upstream, with Nintendo button
  names and a Switch layout.
- `main` mirrors upstream; this work is on `switch`.

## Credits

- **[samwhosung](https://github.com/samwhosung/benilla)** and contributors: benilla.
- **[lucasvallejodev](https://github.com/lucasvallejodev/benilla-gamepad)**: benilla-pad, the controller crate.
- **[thelinkin3000](https://github.com/thelinkin3000/halo-ce-universal)** and nx-mod: the self-updater design, from
  haloce-nx.
- **[gfx-rs](https://github.com/gfx-rs/wgpu)**: wgpu and naga.
- **[devkitPro](https://github.com/devkitPro/uam)**: uam, deko3d, devkitA64; **[switchbrew](https://github.com/switchbrew/libnx)**: libnx.
- **[centollOS](https://github.com/centollOS/SwitchWakerHD)**: the uam patches.
- **[rust-lang](https://github.com/rust-lang/libc)**: the libc crate.

## Legal

Unofficial fan project, not affiliated with Blizzard Entertainment or Nintendo. World of Warcraft is a trademark of
Blizzard Entertainment. Code is under MIT or Apache 2.0, as upstream; uam is under zlib and MIT; the updater's design
is CC0. Running homebrew needs custom firmware, which breaks Nintendo's terms and can get a console banned.
