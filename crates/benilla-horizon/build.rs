//! On the Switch: tls.c, which makes Rust's thread-locals work under libnx (it is whole-archive, so its constructor
//! and pthread_create wrapper are linked even though nothing names them).
fn main() {
    println!("cargo:rerun-if-changed=tls.c");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("horizon") {
        return;
    }
    let devkitpro = std::env::var("DEVKITPRO").unwrap_or_else(|_| "/opt/devkitpro".into());
    cc::Build::new()
        .file("tls.c")
        .include(format!("{devkitpro}/libnx/include"))
        .cargo_metadata(false)
        .compile("wownx_tls");
    let out = std::env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={out}");
    println!("cargo:rustc-link-lib=static:+whole-archive=wownx_tls");
}
