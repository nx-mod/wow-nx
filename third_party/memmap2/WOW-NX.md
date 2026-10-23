# memmap2 for wow-nx

memmap2 0.9.10 from crates.io, vendored for one change: `target_os = "horizon"` uses the crate's stub (every map
returns an error) instead of its Unix code. The Switch has no `mmap`; fontdb, the only user, then skips the file
(Bevy's own font is embedded, not mapped). Nothing else differs.
