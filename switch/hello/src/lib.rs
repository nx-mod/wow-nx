//! wow-nx step 1: Rust `std` on the Switch (Horizon on devkitA64's newlib, linked with libnx).
//!
//! The checks, shared by the two test programs: `src/main.rs` (Rust's own `main`, so std's start-up runs) and `c/`
//! (libnx's C `main` calling `wow_hello` from this library as a static library, the way the hybrid test that worked
//! was linked). Each prints its result and writes it to `sdmc:/wow-nx/<name>.txt`, to be read back over FTP.

use std::collections::HashMap;
use std::io::Write;

// std's missing C functions (getrandom, sysconf), from libnx.
extern crate benilla_horizon;

mod layouts;

#[link(name = "nx")]
extern "C" {
    fn consoleUpdate(console: *mut core::ffi::c_void);
}

/// Runs the checks and writes them to `sdmc:/wow-nx/<file>`. Returns how many failed.
pub fn run_checks(file: &str) -> u32 {
    let mut report = String::new();
    let mut failures = 0;
    let mut check = |name: &str, ok: bool| {
        let line = format!("{} {}\n", if ok { "OK  " } else { "FAIL " }, name);
        print!("{line}");
        report.push_str(&line);
        failures += u32::from(!ok);
        let _ = std::io::stdout().flush();
        unsafe { consoleUpdate(core::ptr::null_mut()) };
    };

    let v: Vec<u64> = (0..1_000_000).collect();
    check("allocation (8 MB Vec)", v.iter().sum::<u64>() == 499_999_500_000);

    let worker = std::thread::spawn(|| (1..=10u32).product::<u32>());
    check("thread", worker.join().ok() == Some(3_628_800));

    let mut map = HashMap::new();
    map.insert("1.12.1", 5875);
    check("HashMap (random seed)", map.get("1.12.1") == Some(&5875));

    let path = format!("sdmc:/wow-nx/{file}");
    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    let written = std::fs::write(&path, "probe\n").is_ok();
    let read_back = std::fs::read_to_string(&path).map(|s| s == "probe\n").unwrap_or(false);
    check("file write and read back", written && read_back);

    let started = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(50));
    check("clock and sleep", started.elapsed() >= std::time::Duration::from_millis(50));

    // The game's archives, if the 1.12.1 client's Data folder is on the SD card: each opened with benilla's own MPQ
    // reader, its (listfile) read and decompressed. Skipped (not failed) without them.
    match std::fs::read_dir("sdmc:/wow-nx/Data") {
        Ok(entries) => {
            let mut mpqs: Vec<_> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().map(|x| x.eq_ignore_ascii_case("mpq")).unwrap_or(false))
                .collect();
            mpqs.sort();
            for p in &mpqs {
                let started_at = std::time::Instant::now();
                let r = benilla_mpq::Archive::open(p).and_then(|a| a.read_file("(listfile)"));
                let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                match r {
                    Ok(list) => {
                        let n = list.split(|&b| b == b'\n').filter(|l| !l.is_empty()).count();
                        check(&format!("MPQ {name}: {n} files listed ({} ms)", started_at.elapsed().as_millis()), n > 0);
                    }
                    Err(e) => check(&format!("MPQ {name}: {e}"), false),
                }
            }
            if mpqs.is_empty() {
                report.push_str("--   no .MPQ in sdmc:/wow-nx/Data\n");
            }
        }
        Err(_) => report.push_str("--   no sdmc:/wow-nx/Data (copy the 1.12.1 client's Data folder there to test MPQs)\n"),
    }

    let _ = std::fs::write(&path, &report);
    failures
}

/// For the C `main` (c/src/main.c): the same checks, from a program whose start-up is libnx's alone.
#[no_mangle]
pub extern "C" fn wow_hello() -> u32 {
    println!("wow-nx: Rust std called from C\n");
    run_checks("hello-c.txt")
}
