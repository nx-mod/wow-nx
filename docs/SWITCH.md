# wow-nx: the Switch port

Status (2026-10-22): **step 1 runs on the console** (`switch/hello`: allocation, threads, HashMap, files, clock).
**Step 2 builds:** the 14 crates that do not draw, with `switch/build-crates.sh`. **Rendering:** wgpu clears the screen
through the deko3d backend on the console (`switch/clear`); shaders and draws are next.
`main` mirrors upstream untouched.

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

## Rules

- `main` tracks upstream only; our work lives on `switch`.
- deko3d for graphics, no Vulkan.
- No game data in the repo: the user's own 1.12.1 client files are read from the SD card.

## Credits

benilla by [samwhosung](https://github.com/samwhosung) and contributors (MIT/Apache-2.0). The 1.12.1 reverse engineering
is [benilla-re](https://github.com/samwhosung/benilla-re). Switch port: nx-mod.
