//! Test A: Rust's own `main`, so std's start-up runs before the checks (see lib.rs).

#[link(name = "nx")]
extern "C" {
    fn consoleInit(console: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn consoleUpdate(console: *mut core::ffi::c_void);
    fn consoleExit(console: *mut core::ffi::c_void);
}

fn main() {
    unsafe { consoleInit(core::ptr::null_mut()) };
    println!("wow-nx: Rust std on the Switch\n");
    let fallos = hello::comprobar("hello.txt");
    println!("\n{fallos} failed. Results in sdmc:/wow-nx/hello.txt. Closing in 10 s.");
    unsafe { consoleUpdate(core::ptr::null_mut()) };
    std::thread::sleep(std::time::Duration::from_secs(10));
    unsafe { consoleExit(core::ptr::null_mut()) };
}
