# wow-nx rendering: wgpu on deko3d

benilla draws through Bevy, which draws through wgpu. On the Switch that has to be deko3d (no Vulkan in our Switch
projects). benilla uses a lot of the GPU stack: 17 WGSL shaders of its own, 12 files with custom render pipelines,
render-graph nodes, compute, storage buffers, texture arrays, on top of Bevy's 3D, 2D and UI renderers. Replacing Bevy's
renderer would mean rewriting all of that and following upstream by hand, so the port goes under wgpu instead.

## Where it plugs in

wgpu-core validates and tracks resources and calls a **wgpu-hal** backend: 125 methods over `Instance`, `Surface`,
`Adapter`, `Device`, `Queue` and `CommandEncoder`. Dawn is built the same way, and nx-mod's Dawn fork already has a
deko3d backend under its own validation layer (`nx-mod/dawn-nx`, branch `switch-deko3d`, `src/dawn/native/deko3d`,
~6,000 lines of C++): the map from WebGPU concepts to deko3d is worked out there.

The deko3d backend is a module of a vendored **wgpu-hal 27** (`third_party/wgpu-hal`), and on `target_os = "horizon"` it
stands in for the `gles` module. To wgpu-core, wgpu and Bevy it is the GL backend, picked with `Backends::GL`. wgpu and
wgpu-core are vendored too, for one line each: upstream builds their GL backend only on Windows, Linux, Android and
FreeBSD, so on Horizon they had none (`third_party/wgpu*/WOW-NX.md`). Bevy is untouched.

## Pieces

| wgpu-hal | deko3d | Dawn reference |
|---|---|---|
| Instance, Adapter | `dkDeviceCreate`, limits and features from Maxwell's | `PhysicalDeviceDeko3d` |
| Surface | `nwindowGetDefault`, `dkSwapchainCreate` | `SwapChainDeko3d` |
| Buffer, Texture | `DkMemBlock` sub-allocation, `dkImageInitialize` | `BufferDeko3d`, `TextureDeko3d`, `ImageAllocationDeko3d` |
| Sampler, BindGroup | descriptor sets (`DkImageDescriptor`, `DkSamplerDescriptor`) bound per stage | `SamplerDeko3d`, `BindGroupDeko3d` |
| ShaderModule, pipelines | WGSL → **naga** GLSL 450 → **uam** → DKSH, cached on the SD | `ShaderModuleDeko3d`, `UamCompilerDeko3d` |
| CommandEncoder | `DkCmdBuf`, state bound per draw (deko3d has no pipeline objects) | `CommandBufferDeko3d` |
| Queue | `dkQueueSubmitCommands`, fences | `QueueDeko3d` |

deko3d's limits per stage: 16 UBOs, 32 textures, 16 storage buffers. uam is the deko3d shader compiler as a library
(the copy in rexglue-sdk-nx, `thirdparty/uam`), linked into the NRO.

## Windowing

No winit on Horizon: the app runs Bevy without `WinitPlugin`, with a runner of its own that steps the app inside
libnx's applet loop, reads HID, and gives the primary window a raw handle to the default `NWindow`
(`crates/benilla-app/src/switch.rs`). The window is held at the screen's 1280x720 whatever the video settings ask, and
that hold sends `WindowResized`: a camera recomputes its projection only on that message, which winit sends elsewhere,
and without it the UI camera kept the settings' 1600x900 and drew the UI at 80 % of the screen.

## What the backend has to get right

Found on the console; each is in the code where it applies.

- **The GPU ring.** libnx's channel and the GPU's ring are 2048 GPFIFO entries each; a kickoff that does not fit times
  out (`0xd5c`, "gpu channel kickoff failed"), as does one behind a hung GPU. Entries in flight are counted per list
  and reclaimed through per-submission fences, the queue waits before an overrun, and a submission is one flush.
- **Barriers** only where a transition needs one: a full barrier opens a new ring entry, and one per copy overran the
  ring at the login screen.
- **Small buffers** come from a pool of 4 MiB blocks by power-of-two class: each `dkMemBlockCreate` is several
  kernel calls, and wgpu makes a staging buffer for every `write_buffer`.
- **Compressed copies in blocks.** `dkCmdBufCopyImage` measures a BC image in 4x4 blocks and does not convert a texel
  rect itself (its buffer copies do): a texel rect copied 16 times the area and hung the GPU on world entry.
- **One texture, one sampler** per GLSL sampler: naga's GLSL writer cannot sample a texture with two samplers, so a
  shader that does gets the texture split into one per sampler before translation.
- **The launch picture.** Until the first frame, a framebuffer on the default window shows `sdmc:/wow-nx/loading.rgba`
  (or black, not the HOME menu's white); the backend takes the window over when it makes its swapchain and keeps the
  picture up while the first frames' pipelines build.
- **One frame in flight.** Presenting a frame waits for the one before it to finish on the GPU. Without that the CPU
  ran frames ahead into the swapchain's three images and the queue's command memory, so where the GPU is the limit
  (the character screen) an input showed several frames late and the motion stuttered.
- **Timing.** Each pass is timed by GPU timestamps into a report ring; the log prints the passes (and, with
  `WOWNX_GPU_PIPELINES=1`, the pipelines) every 300 frames. `WOWNX_DEKO3D_DEBUG=1` builds against deko3d's debug
  library and names the passes in a submission that faults.

## Done

1. Vendored wgpu-hal with a deko3d module, and deko3d's C API in Rust.
2. Instance, adapter, device, queue, buffers, a surface that clears (`switch/clear`).
3. Shaders (naga → uam), pipelines, textures, bind groups, the rest of wgpu's calls (`switch/cube`).
4. Bevy and benilla.
