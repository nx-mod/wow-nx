fn main() {
    // wow-nx: on Horizon the GLES slot is the deko3d backend (src/deko3d): deko3d.h's inline functions as real ones
    // (bindgen's wrappers in ffi_inline.c), compiled by devkitA64's gcc, and libdeko3d from libnx.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("horizon") && std::env::var_os("CARGO_FEATURE_GLES").is_some()
    {
        let devkitpro = std::env::var("DEVKITPRO").unwrap_or_else(|_| "/opt/devkitpro".into());
        cc::Build::new()
            .file("src/deko3d/ffi_inline.c")
            .include(format!("{devkitpro}/libnx/include"))
            .define("__SWITCH__", None)
            .compile("deko3d_inline");
        println!("cargo:rustc-link-search=native={devkitpro}/libnx/lib");
        // WOWNX_DEKO3D_DEBUG=1: deko3d's debug build, which checks every call and reports through the debug
        // callback, and a wait after every submission that names the one that faulted the GPU (src/deko3d).
        println!("cargo:rerun-if-env-changed=WOWNX_DEKO3D_DEBUG");
        println!("cargo:rustc-check-cfg=cfg(deko3d_debug)");
        if std::env::var_os("WOWNX_DEKO3D_DEBUG").is_some() {
            println!("cargo:rustc-link-lib=static=deko3dd");
            println!("cargo:rustc-cfg=deko3d_debug");
        } else {
            println!("cargo:rustc-link-lib=static=deko3d");
        }
        println!("cargo:rerun-if-changed=src/deko3d/ffi_inline.c");
    }
    cfg_aliases::cfg_aliases! {
        native: { not(target_arch = "wasm32") },
        send_sync: { any(
            not(target_arch = "wasm32"),
            all(feature = "fragile-send-sync-non-atomic-wasm", not(target_feature = "atomics"))
        ) },
        webgl: { all(target_arch = "wasm32", not(target_os = "emscripten"), gles) },
        Emscripten: { all(target_os = "emscripten", gles) },
        dx12: { all(target_os = "windows", feature = "dx12") },
        gles: { all(feature = "gles") },
        // Within the GL ES backend, use `std` and be Send + Sync only if we are using a target
        // that, among the ones where the GL ES backend is supported, has `std`.
        gles_with_std: { all(
            feature = "gles",
            any(
                not(target_arch = "wasm32"),
                // Accept wasm32-unknown-unknown, which uniquely has a stub `std`
                all(target_vendor = "unknown", target_os = "unknown"),
                // Accept wasm32-unknown-emscripten and similar, which has a real `std`
                target_os = "emscripten"
            )
        ) },
        metal: { all(target_vendor = "apple", feature = "metal") },
        vulkan: { all(not(target_arch = "wasm32"), feature = "vulkan") },
        // ⚠️ Keep in sync with target.cfg() definition in Cargo.toml and cfg_alias in `wgpu` crate ⚠️
        static_dxc: { all(target_os = "windows", feature = "static-dxc", not(target_arch = "aarch64")) },
        supports_64bit_atomics: { target_has_atomic = "64" },
        supports_ptr_atomics: { target_has_atomic = "ptr" }
    }
}
