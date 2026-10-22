//! wow-nx step 1: Rust `std` on the Switch (Horizon on devkitA64's newlib, linked with libnx).
//!
//! Prints to libnx's console and writes `sdmc:/wow-nx/hello.txt` with the result of each check, so the run can be
//! read back over FTP: allocation, a thread, a HashMap (std's random seed), and a file written and read back.

use std::collections::HashMap;
use std::io::Write;

mod horizon;

// libnx's text console (switch/runtime/devices/console.h): stdout goes to the screen once it is initialized. Linked
// here, from the program itself, so libnx comes before newlib's libc on the link line: both define fcntl, and libc first
// pulled its own and then clashed with libnx's sockets.
#[link(name = "nx")]
extern "C" {
    fn consoleInit(console: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn consoleUpdate(console: *mut core::ffi::c_void);
    fn consoleExit(console: *mut core::ffi::c_void);
}

fn main() {
    unsafe { consoleInit(core::ptr::null_mut()) };
    let mut informe = String::new();
    let mut paso = |nombre: &str, ok: bool| {
        let linea = format!("{} {}\n", if ok { "OK  " } else { "MAL " }, nombre);
        print!("{linea}");
        informe.push_str(&linea);
        let _ = std::io::stdout().flush();
        unsafe { consoleUpdate(core::ptr::null_mut()) };
    };

    println!("wow-nx: Rust std on the Switch\n");

    let v: Vec<u64> = (0..1_000_000).collect();
    paso("allocation (8 MB Vec)", v.iter().sum::<u64>() == 499_999_500_000);

    let hilo = std::thread::spawn(|| (1..=10u32).product::<u32>());
    paso("thread", hilo.join().ok() == Some(3_628_800));

    let mut mapa = HashMap::new();
    mapa.insert("1.12.1", 5875);
    paso("HashMap (random seed)", mapa.get("1.12.1") == Some(&5875));

    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    let escrito = std::fs::write("sdmc:/wow-nx/hello.txt", "probe\n").is_ok();
    let leido = std::fs::read_to_string("sdmc:/wow-nx/hello.txt").map(|s| s == "probe\n").unwrap_or(false);
    paso("file write and read back", escrito && leido);

    let _ = std::fs::write("sdmc:/wow-nx/hello.txt", &informe);
    println!("\nResults in sdmc:/wow-nx/hello.txt. Closing in 10 s.");
    unsafe { consoleUpdate(core::ptr::null_mut()) };
    std::thread::sleep(std::time::Duration::from_secs(10));
    unsafe { consoleExit(core::ptr::null_mut()) };
}
