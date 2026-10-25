//! On the Switch: tls.c, which makes Rust's thread-locals work under libnx; kickoff.c, which reports a failed GPU
//! kickoff; keyboard.c, the system keyboard; screen.c, the black screen at launch; net.c, the socket session;
//! thread.c, threads across the cores; update.c and https.c, the self-updater; sampler.c, the profiler
//! (whole-archive, so the constructor and link-time wrappers are linked even though nothing names them).
fn main() {
    println!("cargo:rerun-if-changed=tls.c");
    println!("cargo:rerun-if-changed=kickoff.c");
    println!("cargo:rerun-if-changed=keyboard.c");
    println!("cargo:rerun-if-changed=screen.c");
    println!("cargo:rerun-if-changed=net.c");
    println!("cargo:rerun-if-changed=thread.c");
    println!("cargo:rerun-if-changed=update.c");
    println!("cargo:rerun-if-changed=update.h");
    println!("cargo:rerun-if-changed=https.c");
    println!("cargo:rerun-if-changed=sampler.c");
    // A release's number (switch/release.sh), which names its GitHub release; a build without one never updates.
    println!("cargo:rerun-if-env-changed=WOWNX_BUILD_NUMBER");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("horizon") {
        return;
    }
    let devkitpro = std::env::var("DEVKITPRO").unwrap_or_else(|_| "/opt/devkitpro".into());
    cc::Build::new()
        .file("tls.c")
        .file("kickoff.c")
        .file("keyboard.c")
        .file("screen.c")
        .file("net.c")
        .file("thread.c")
        .file("update.c")
        .file("https.c")
        .file("sampler.c")
        .define("WOWNX_BUILD_NUMBER", build_number().as_str())
        .include(format!("{devkitpro}/libnx/include"))
        .cargo_metadata(false)
        .compile("wownx_tls");
    let out = std::env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={out}");
    println!("cargo:rustc-link-lib=static:+whole-archive=wownx_tls");
}

/// `WOWNX_BUILD_NUMBER` as a C integer: digits only, else 0.
fn build_number() -> String {
    let n = std::env::var("WOWNX_BUILD_NUMBER").unwrap_or_default();
    if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) { n } else { "0".into() }
}
