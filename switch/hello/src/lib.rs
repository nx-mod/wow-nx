//! wow-nx step 1: Rust `std` on the Switch (Horizon on devkitA64's newlib, linked with libnx).
//!
//! The checks, shared by the two test programs: `src/main.rs` (Rust's own `main`, so std's start-up runs) and `c/`
//! (libnx's C `main` calling `wow_hello` from this library as a static library, the way the hybrid test that worked
//! was linked). Each prints its result and writes it to `sdmc:/wow-nx/<name>.txt`, to be read back over FTP.

use std::collections::HashMap;
use std::io::Write;

// std's missing C functions (getrandom, sysconf), from libnx.
extern crate benilla_horizon;

mod tipos;

#[link(name = "nx")]
extern "C" {
    fn consoleUpdate(console: *mut core::ffi::c_void);
}

/// Runs the checks and writes them to `sdmc:/wow-nx/<archivo>`. Returns how many failed.
pub fn comprobar(archivo: &str) -> u32 {
    let mut informe = String::new();
    let mut fallos = 0;
    let mut paso = |nombre: &str, ok: bool| {
        let linea = format!("{} {}\n", if ok { "OK  " } else { "MAL " }, nombre);
        print!("{linea}");
        informe.push_str(&linea);
        fallos += u32::from(!ok);
        let _ = std::io::stdout().flush();
        unsafe { consoleUpdate(core::ptr::null_mut()) };
    };

    let v: Vec<u64> = (0..1_000_000).collect();
    paso("allocation (8 MB Vec)", v.iter().sum::<u64>() == 499_999_500_000);

    let hilo = std::thread::spawn(|| (1..=10u32).product::<u32>());
    paso("thread", hilo.join().ok() == Some(3_628_800));

    let mut mapa = HashMap::new();
    mapa.insert("1.12.1", 5875);
    paso("HashMap (random seed)", mapa.get("1.12.1") == Some(&5875));

    let ruta = format!("sdmc:/wow-nx/{archivo}");
    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    let escrito = std::fs::write(&ruta, "probe\n").is_ok();
    let leido = std::fs::read_to_string(&ruta).map(|s| s == "probe\n").unwrap_or(false);
    paso("file write and read back", escrito && leido);

    let tiempo = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(50));
    paso("clock and sleep", tiempo.elapsed() >= std::time::Duration::from_millis(50));

    // The game's archives, if the 1.12.1 client's Data folder is on the SD card: each opened with benilla's own MPQ
    // reader, its (listfile) read and decompressed. Skipped (not failed) without them.
    match std::fs::read_dir("sdmc:/wow-nx/Data") {
        Ok(entradas) => {
            let mut mpqs: Vec<_> = entradas
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().map(|x| x.eq_ignore_ascii_case("mpq")).unwrap_or(false))
                .collect();
            mpqs.sort();
            for p in &mpqs {
                let inicio = std::time::Instant::now();
                let r = benilla_mpq::Archive::open(p).and_then(|a| a.read_file("(listfile)"));
                let nombre = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                match r {
                    Ok(lista) => {
                        let n = lista.split(|&b| b == b'\n').filter(|l| !l.is_empty()).count();
                        paso(&format!("MPQ {nombre}: {n} files listed ({} ms)", inicio.elapsed().as_millis()), n > 0);
                    }
                    Err(e) => paso(&format!("MPQ {nombre}: {e}"), false),
                }
            }
            if mpqs.is_empty() {
                informe.push_str("--   no .MPQ in sdmc:/wow-nx/Data\n");
            }
        }
        Err(_) => informe.push_str("--   no sdmc:/wow-nx/Data (copy the 1.12.1 client's Data folder there to test MPQs)\n"),
    }

    let _ = std::fs::write(&ruta, &informe);
    fallos
}

/// For the C `main` (c/src/main.c): the same checks, from a program whose start-up is libnx's alone.
#[no_mangle]
pub extern "C" fn wow_hello() -> u32 {
    println!("wow-nx: Rust std called from C\n");
    comprobar("hello-c.txt")
}
