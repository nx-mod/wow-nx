//! Stamps the commit this binary was built from, and gives the main thread the stack the app needs; on the
//! Switch, routes deko3d's GPU kickoffs through benilla-horizon's kickoff.c, which reports a failed one, and fork
//! handlers through its tls.c.

fn main() {
    benilla_buildstamp::emit();
    main_thread_stack();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("horizon") {
        println!("cargo:rustc-link-arg-bins=-Wl,--wrap=nvGpuChannelKickoff");
        // benilla-horizon's tls.c: fork handlers register as a no-op (rand panics when newlib refuses them).
        println!("cargo:rustc-link-arg-bins=-Wl,--wrap=pthread_atfork");
        // thread.c: each new thread takes the next core as its ideal one.
        println!("cargo:rustc-link-arg-bins=-Wl,--wrap=__syscall_thread_create");
    }
}

/// The app runs on the main thread, where the windowing event loop must live, and needs more than
/// the 1 MB the MSVC linker reserves for it by default (2 MB on mingw): the login screen overflows
/// it. Linux and macOS give the main thread 8 MB, so Windows reserves the same.
fn main_thread_stack() {
    const STACK: u32 = 8 << 20;
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    match std::env::var("CARGO_CFG_TARGET_ENV").as_deref() {
        Ok("msvc") => println!("cargo:rustc-link-arg-bins=/STACK:{STACK}"),
        Ok("gnu") => println!("cargo:rustc-link-arg-bins=-Wl,--stack,{STACK}"),
        _ => {}
    }
}
