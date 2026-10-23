# bevy_asset for wow-nx

bevy_asset 0.18.1 from crates.io, vendored for one change: on `target_os = "horizon"` the file asset reader takes the
path it takes on Windows (`src/io/file/file_asset.rs`, `Cargo.toml`), so async-io, whose reactor (polling, rustix) has
no Horizon support, is not built. Files are still read through async-fs on its thread pool. Nothing else differs.
