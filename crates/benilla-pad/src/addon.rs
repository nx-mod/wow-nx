//! The BenillaPad addon ships inside the binary and is kept installed in
//! `benilla-config/AddOns/BenillaPad`, where benilla loads addons from. A file is written only
//! when its bytes differ, so an unchanged install is left alone, and a file the addon no longer
//! ships is removed (the folder is ours). `BENILLA_PAD_NO_INSTALL=1` skips the sync, for editing
//! the installed copy by hand.

use std::path::Path;

use bevy::prelude::*;
use include_dir::{include_dir, Dir};

static ADDON: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/addon/BenillaPad");

/// Startup: write the embedded addon into the AddOns folder.
pub fn install_addon() {
    if std::env::var("BENILLA_PAD_NO_INSTALL").is_ok_and(|v| v == "1") {
        info!("pad: addon sync skipped (BENILLA_PAD_NO_INSTALL=1)");
        return;
    }
    let Some(config) = benilla_app::config_dir() else {
        warn!("pad: no benilla-config folder, the BenillaPad addon is not installed");
        return;
    };
    let root = config.join("AddOns").join("BenillaPad");
    if let Err(e) = prune(&ADDON, &root) {
        warn!("pad: could not tidy the BenillaPad addon folder: {e}");
    }
    match sync(&ADDON, &root) {
        Ok(0) => info!("pad: BenillaPad addon up to date at {}", root.display()),
        Ok(n) => info!(
            "pad: BenillaPad addon installed ({n} files) at {}",
            root.display()
        ),
        Err(e) => warn!("pad: could not install the BenillaPad addon: {e}"),
    }
}

/// Write every file of `dir` under `root` whose bytes differ; the count written.
fn sync(dir: &Dir<'_>, root: &Path) -> std::io::Result<usize> {
    let mut written = 0;
    for file in dir.files() {
        let path = root.join(file.path());
        if std::fs::read(&path).is_ok_and(|have| have == file.contents()) {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, file.contents())?;
        written += 1;
    }
    for sub in dir.dirs() {
        written += sync(sub, root)?;
    }
    Ok(written)
}

/// Remove every file under `root` that `dir` does not ship; the count removed.
fn prune(dir: &Dir<'_>, root: &Path) -> std::io::Result<usize> {
    fn walk(dir: &Dir<'_>, root: &Path, here: &Path, removed: &mut usize) -> std::io::Result<()> {
        let Ok(entries) = std::fs::read_dir(here) else {
            return Ok(());
        };
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                walk(dir, root, &path, removed)?;
                continue;
            }
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            if dir.get_file(rel).is_none() {
                std::fs::remove_file(&path)?;
                *removed += 1;
            }
        }
        Ok(())
    }
    let mut removed = 0;
    walk(dir, root, root, &mut removed)?;
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_addon_has_its_toc_and_files() {
        let toc = ADDON
            .get_file("BenillaPad.toc")
            .expect("the toc is embedded");
        let toc = std::str::from_utf8(toc.contents()).expect("utf-8 toc");
        for line in toc
            .lines()
            .filter(|l| !l.starts_with("##") && !l.trim().is_empty())
        {
            assert!(
                ADDON.get_file(line.trim()).is_some(),
                "the toc lists {line}, which is not embedded"
            );
        }
        assert!(ADDON.get_file("Bindings.xml").is_some());
    }

    #[test]
    fn a_second_sync_writes_nothing() {
        let dir = std::env::temp_dir().join(format!("benilla-pad-sync-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let first = sync(&ADDON, &dir).expect("first sync");
        assert!(first > 0);
        assert_eq!(sync(&ADDON, &dir).expect("second sync"), 0);
        // A file the addon no longer ships goes; the shipped ones stay.
        std::fs::write(dir.join("Art").join("Stale.tga"), b"old").expect("stale file");
        assert_eq!(prune(&ADDON, &dir).expect("prune"), 1);
        assert!(!dir.join("Art").join("Stale.tga").exists());
        assert_eq!(prune(&ADDON, &dir).expect("second prune"), 0);
        // Nothing shipped was pruned, in a subfolder either: a sync after has nothing to write.
        assert_eq!(sync(&ADDON, &dir).expect("sync after prune"), 0);
        assert!(dir.join("BenillaPad.toc").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
