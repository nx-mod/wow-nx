# wgpu-hal 27.0.4 for wow-nx

Upstream wgpu-hal 27.0.4 (crates.io) with a deko3d backend for the Switch (`src/deko3d/`). On `target_os = "horizon"` it
is compiled as the `gles` module, so wgpu-core and wgpu treat it as `Backend::Gl` with no changes of their own; EGL and
libloading are not built there. Everywhere else this crate is upstream's. Plan: `docs/SWITCH-RENDER.md`.
