# wgpu for wow-nx

wgpu 27.0.1 from crates.io, vendored for one change: its GL backend is compiled on
`target_os = "horizon"` too (`build.rs`, the `gles` alias).
On the Switch that slot is the deko3d backend in `third_party/wgpu-hal`. Nothing else differs from upstream.
