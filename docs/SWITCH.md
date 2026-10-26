# wow-nx: the Switch port

Status (2026-10-25): **playable, in progress.** It logs in to a 1.12.1 server (Kronos), enters the world and plays
with the Joy-Con through benilla-pad, touch, or a USB keyboard and mouse, at 17–22 fps in the world at 720p on the
first release build. Upstream is merged to 0bd7d053. The main thread's speed is the open work. `main` mirrors
upstream untouched.

## How the port works

| Piece | On the PC | On the Switch |
|---|---|---|
| Rust `std` | yes | a custom target, `aarch64-nintendo-switch-newlib` (`target_os = "horizon"`, the code path the 3DS target uses in `std`), built with `-Zbuild-std` against devkitA64's newlib and libnx; `third_party/libc` has Horizon's 64-bit and socket layouts (`third_party/libc/WOW-NX.md`) |
| Rendering | Bevy on wgpu (Vulkan, Metal, DX12) | Bevy on wgpu on our deko3d wgpu-hal backend, in wgpu's GL slot ([SWITCH-RENDER.md](SWITCH-RENDER.md)); shaders go naga → GLSL → uam, cached on the SD card |
| Audio | kira on cpal | kira on audout, at 48 kHz |
| Window, input | winit | `crates/benilla-app/src/switch.rs`: libnx's default window, the applet loop, HID (touch, USB keyboard and mouse, the Joy-Con as Bevy's gamepad), the system keyboard |
| Controller play | — | benilla-pad (`crates/benilla-pad`), with Nintendo button names and a Switch layout |
| Files | the WoW folder | `sdmc:/wow-nx/Data` (the user's client) and `sdmc:/wow-nx/benilla-config`; `sdmc:/wow-nx/benilla.env` sets the login and switches |
| Threads, memory | OS | libnx threads spread over the 3 cores, a compute worker per core, and a thread-caching allocator in front of newlib's malloc (`crates/benilla-horizon`) |
| Updates | — | a numbered release looks for a newer `build-<n>` on GitHub while it loads (`update.c`, `switch/release.sh`) |

## Steps taken

1. `std` on Horizon: a Rust hello world as an NRO (`switch/hello/`).
2. The non-graphics crates on the target, opening the user's MPQs on the console.
3. The deko3d wgpu-hal backend: a cleared screen, then shaders, pipelines and Bevy (`switch/clear/`, `switch/cube/`).
4. benilla: the login screen, a server login (Kronos), the world.
5. The Joy-Con (benilla-pad), the launch picture, the updater, and the speed work in Open work.

## Open work

Measured on the console: the backend logs fps and GPU time per pass every 300 frames (`WOWNX_GPU_PIPELINES=1` adds
each pipeline), and `WOWNX_PROFILE=1` samples every thread 250 times a second into `sdmc:/wow-nx/samples.txt`, which
`switch/profile.py` names from the ELF, with the lock a blocked thread waits on.

Fixed on 2026-10-25, for the record: a world-load crash (the GPU ring full under a loading burst; a timed-out
kickoff is now retried while the channel is healthy, `kickoff.c`) and status bars with chunks missing (deko3d's tile
height for an 8-row texture was taller than the image; `create_texture` picks the smallest that covers it).

Speed: 17–22 fps in the world at 720p on the first release build (`switch-ship`: dependencies in one codegen unit
each). Profiles from test builds: in the world the main
thread is the frame (~88 % busy); the workers and the render thread idle. Done: Bevy's compute pool has a worker per
core and threads spread over the cores (`thread.c`); a thread-caching allocator took the main thread's waits on
newlib's malloc lock from ~21 % to ~3 % of its samples. Next:

- **Inlining.** The profile's top entries were small Bevy functions (parameter checks, table and resource lookups)
  that a full build inlines. Releases now build with `switch-ship` (dependencies in one codegen unit each); our own
  crates stay at 16 units without LTO, since `benilla-app` in one unit does not fit the phone. A CI build has the
  memory for more (thin LTO, our crates at fewer units): worth measuring there.
- **The call host's parameter checks** (`Appliers::validate_param`, ~4 %): every system that lends the host validates
  its ~70 parts each frame.
- **Multi-threaded Update/PostUpdate** (`WOW_MT_UPDATE=1`, `WOW_MT_POSTUPDATE=1`): upstream's switches; on the first
  try the world load crashed, which turned out to happen without them too. Measure again once the crash is fixed.
- **Start-up tables load one at a time**: each Startup loader holds the MPQ chain's mutex through its read and parse;
  a lock-free shared chain would let them run together. The first frame spends ~10 s on them.
- **GPU:** `pbr_opaque_mesh_pipeline` is ~9–11 of ~14–18 ms in the world, but terrain, models and the sky share that
  label (their materials extend `StandardMaterial`): label them apart to see which to work on. Bevy's PBR fragment
  shader is 93 registers after folding its dead shadow paths (from 104).
- Pipelines built in the first frames are hidden behind the loading picture; a warm cache of translated shaders would
  shorten the start.
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
- **benilla-pad on the Switch** (`crates/benilla-pad`, from lucasvallejodev/benilla-gamepad): the gamepad bar and the
  controller menu answer since the pad's events go through `BenillaPad_Event` (1.12's `RegisterEvent` takes only
  its table's names, so `BENILLAPAD_*` registered nothing). Still to try: the wheels, the cursor in every window,
  the on-screen keyboard and chat tiles.

To finish:

- **Nintendo button names everywhere.** The bar, hint lines, keyboard, wheels, chat and keybind screen say B/A/Y/X,
  L/R, ZL/ZR, LS/RS and +/− under the Nintendo symbols (`P.Name`, `P.Hint` in the BenillaPad addon's Core.lua). Still
  in Xbox names: the controller menu's descriptions (Menu.lua; a word-for-word swap would turn the article "A" into
  "B", so they need Nintendo wording of their own), the README's tables, and any other text that names a button.
- **Round cooldown sweeps.** The gamepad bar's round buttons draw the stock square cooldown model at the largest
  square inside the circle (benilla-pad's Bar.lua), because 1.12 has no mask for a model. True circles: let a Model
  pane's quad take the circle mask the round portraits use (`circular` in the UI quad shader), and the addon mark its
  cooldowns with it.

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
