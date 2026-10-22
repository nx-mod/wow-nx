# wgpu-core for wow-nx

wgpu-core 27.0.3 from crates.io, vendored for one change: its GL backend is compiled on
`target_os = "horizon"` too (`build.rs`, the `gles` alias; `Cargo.toml`, the `gles` feature enables `wgpu-hal/gles` directly, since the helper crate that does it elsewhere is Windows/Linux/Android only).
On the Switch that slot is the deko3d backend in `third_party/wgpu-hal`. Nothing else differs from upstream.
