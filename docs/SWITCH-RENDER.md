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
stands in for the `gles` module. wgpu-core, wgpu and Bevy are untouched: to them it is the GL backend, picked with
`Backends::GL`. Only wgpu-hal is patched (`[patch.crates-io]`).

## Pieces

| wgpu-hal | deko3d | Dawn reference |
|---|---|---|
| Instance, Adapter | `dkDeviceCreate`, limits and features from Maxwell's | `PhysicalDeviceDeko3d` |
| Surface | `nwindowGetDefault`, `dkSwapchainCreate` | `SwapChainDeko3d` |
| Buffer, Texture | `DkMemBlock` sub-allocation, `dkImageInitialize` | `BufferDeko3d`, `TextureDeko3d`, `ImageAllocationDeko3d` |
| Sampler, BindGroup | descriptor sets (`DkImageDescriptor`, `DkSamplerDescriptor`) bound per stage | `SamplerDeko3d`, `BindGroupDeko3d` |
| ShaderModule, pipelines | WGSL → **naga** GLSL 4.60 → **uam** → DKSH, cached on the SD | `ShaderModuleDeko3d`, `UamCompilerDeko3d` |
| CommandEncoder | `DkCmdBuf`, state bound per draw (deko3d has no pipeline objects) | `CommandBufferDeko3d` |
| Queue | `dkQueueSubmitCommands`, fences | `QueueDeko3d` |

deko3d's limits per stage: 16 UBOs, 32 textures, 16 storage buffers. uam is the deko3d shader compiler as a library
(the copy in rexglue-sdk-nx, `thirdparty/uam`), linked into the NRO.

## Windowing

No winit on Horizon: the app runs Bevy without `WinitPlugin` (as `headless_client` already does) with a runner of
its own that pumps libnx's applet loop and HID, and creates the wgpu surface from the default `NWindow` (the backend
ignores the raw window handle it is given).

## Order

1. Vendored wgpu-hal with a deko3d module that compiles for Horizon (stubs), and deko3d's C API in Rust.
2. Instance, adapter, device, queue, buffers, a surface that clears: a coloured screen from wgpu on the console.
3. Shaders (naga → uam), pipelines, a triangle; then textures, bind groups, the rest of wgpu's calls.
4. Bevy's examples, then benilla.
