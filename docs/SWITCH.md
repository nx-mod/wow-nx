# wow-nx: the Switch port

Status (2026-10-24): **the client runs on the console.** It reaches the login screen (EULA and Terms accepted and
saved), draws through our deko3d wgpu-hal backend at 30–60 fps, plays the glue music, and types with the system
keyboard. Next: logging in to a server and playing. `main` mirrors upstream untouched.

## What stands in the way

| Piece | On the PC | On the Switch | Work |
|---|---|---|---|
| Rust `std` | yes | none upstream: `aarch64-nintendo-switch-freestanding` is no_std | a custom target with `target_os = "horizon"`, `target_env = "newlib"` (the code path the 3DS target already uses in `std`), built with `-Zbuild-std` and linked against devkitA64's newlib and libnx |
| Rendering | Bevy on wgpu (Vulkan/Metal/DX12) | deko3d only (no Vulkan on our Switch projects) | a wgpu-hal backend over deko3d, or Bevy's renderer replaced; the largest piece |
| Audio | cpal | audout / audren | a small cpal host, or our own output |
| Windowing, input | winit | libnx applet, HID | a winit-like event source for Bevy |
| Files | the WoW folder | `sdmc:/wow-nx/` beside the NRO | paths only |
| Threads, memory | OS | libnx threads, ~3.2 GB in an application | Bevy's task pools sized to 3 cores |

## Order

1. **std on Horizon:** a Rust hello world as an NRO that prints, allocates, spawns a thread and reads a file
   on the console. Nothing else moves until this runs (`switch/hello/`).
2. **The non-graphics crates** (`benilla-mpq`, `-dbc`, `-blp`, `-adt`, `-m2`, `-wmo`, `-lua`...) build for the target and
   pass a console smoke test that opens the user's MPQs.
3. **Rendering:** decide between a deko3d wgpu-hal backend and a narrower renderer, after measuring what Bevy needs.
4. Audio, input, the app.

## Open work

Speed, found by measuring on the console (the backend logs every 300 frames; `WOWNX_GPU_PIPELINES=1` in
`sdmc:/wow-nx/benilla.env` times each pipeline):

- **Start-up tables load one at a time.** Each Startup loader holds the MPQ chain's mutex through its read and its
  parse. Planned: a lock-free shared chain (`lock_recover()` hands out a copy: a `Chain` is a few `Arc` handles and
  reads are `&self`). The first frame spends ~10 s on them. (The compute pool now has a worker per core, and threads
  spread over the cores: benilla-horizon's thread.c.)
- **Which material the world's GPU time goes to.** In the world `pbr_opaque_mesh_pipeline` is ~10.8 of ~18 ms, but
  Bevy gives terrain, models and the sky that one label (their materials extend `StandardMaterial`): label the
  pipelines per material in the timing to see which to work on.
- **The GPU is the frame limit at the login screen** (~19.5 ms a frame): Bevy's PBR fragment shader is 93 registers
  after folding its dead shadow paths (from 104), so few warps are in flight. Next: what keeps the rest live
  (`switch/shadercheck` prints registers and scratch), and whether the clustered lights read faster from uniform
  buffers than from storage buffers (`max_storage_buffers_per_shader_stage < 3` switches Bevy to them).
- `deko3d: N blocks made` stays at ~50 per 300 frames: something above the buffer pool's 1 MiB still makes and frees
  memory blocks.
- Pipelines built during the first frames (~200 ms for 27) are hidden behind a black start; a warm cache of
  translated shaders would shorten it.
- Upstream #2305 (zone events every frame) and #1865 (frame pacer) are CPU per frame; left to upstream.

Play without an outside server (an idea, not started):

- **Single player:** a 1.12.1 server (vmangos) embedded in the NRO, the client dialling it over loopback. A MaNGOS
  core already simulates only the grids around a player and unloads the rest, so one player costs about a zone.
  The work is the port: vmangos's C++ on Horizon, its MySQL world database as SQLite (or a converted subset, only
  the current map's spawns in memory), and vmaps/mmaps loaded tile by tile.
- **Or simpler, our own:** a purpose-built server in Rust in this workspace instead of a vmangos port: benilla's
  protocol types (in-process messages for one player, no sockets or SRP), its DBC readers and map data, and only
  the game logic on top, one zone first. Licence: vmangos and its world database are GPL-2.0, benilla MIT/Apache;
  taking either's code or data makes the NRO GPL, a choice to make before starting.
- **LAN, up to 4 players:** the same server on one Switch, listening on the network; the others dial its IP or find
  it by a broadcast. Keeping the party in one zone keeps the host at about a zone's work.

To test:

- **Vulkan.** Upstream now vendors wgpu-hal for its Vulkan presentation-stall patch (`third_party/wgpu-hal/BENILLA.md`,
  merged here). deko3d stays the Switch's renderer; trying Vulkan against it is on the list.
- **Releases and the updater** (`switch/release.sh`, `crates/benilla-horizon/update.c`): publish a build, then a newer
  one, and update from the first on the console.
- **The Joy-Con through benilla-pad** (`crates/benilla-pad`, from lucasvallejodev/benilla-gamepad): its layout,
  cursor and on-screen keyboard on the Switch.

Not done on purpose: no clock changes (stock CPU/GPU), no features turned off for speed.

## Rules

- `main` tracks upstream only; our work lives on `switch`.
- deko3d for graphics (Vulkan only as the test above).
- No game data in the repo: the user's own 1.12.1 client files are read from the SD card.

## Credits

benilla by [samwhosung](https://github.com/samwhosung) and contributors (MIT/Apache-2.0). The 1.12.1 reverse engineering
is [benilla-re](https://github.com/samwhosung/benilla-re). Switch port: nx-mod. Controller play:
[benilla-pad](https://github.com/lucasvallejodev/benilla-gamepad) by lucasvallejodev (MIT/Apache-2.0). The self-updater
and its HTTPS client follow [haloce-nx](https://github.com/nx-mod/haloce-nx)'s, by thelinkin3000 and nx-mod (CC0).
