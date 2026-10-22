//! Builds uam (third_party/uam) for the Switch, mirroring its CMakeLists.txt: the sources it names plus the plain
//! file lists of mesa-imported/*/meson.build. Elsewhere nothing is built.
use std::path::PathBuf;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("horizon") {
        return;
    }
    let uam = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../third_party/uam");
    let mesa = uam.join("mesa-imported");
    let gen = uam.join("generated");
    let mut c = vec![uam.join("source/mini-os.c"), gen.join("glsl/glcpp/glcpp-parse.c"), gen.join("glsl/glcpp/glcpp-lex.c"), mesa.join("glsl/glcpp/pp.c")];
    let mut cpp = vec![
        uam.join("source/compiler_iface.cpp"),
        uam.join("source/glsl_frontend.cpp"),
        uam.join("source/tgsi_support.cpp"),
        uam.join("uam_api.cpp"),
        gen.join("glsl/glsl_parser.cpp"),
        gen.join("glsl/glsl_lexer.cpp"),
        PathBuf::from("uam_c.cpp"),
    ];
    for sub in ["codegen", "tgsi", "util", "cso_cache", "glsl", "compiler", "program", "state_tracker", "main"] {
        let mb = std::fs::read_to_string(mesa.join(sub).join("meson.build")).unwrap();
        println!("cargo:rerun-if-changed={}", mesa.join(sub).join("meson.build").display());
        // every 'name' inside files(...)
        for bloque in mb.split("files(").skip(1) {
            let bloque = &bloque[..bloque.find(')').unwrap_or(bloque.len())];
            for nombre in bloque.split('\'').skip(1).step_by(2) {
                let f = mesa.join(sub).join(nombre);
                match f.extension().and_then(|e| e.to_str()) {
                    Some("c") => c.push(f),
                    Some("cpp" | "cc") => cpp.push(f),
                    _ => {}
                }
            }
        }
    }
    let incluir = [uam.clone(), uam.join("source"), mesa.clone(), gen.clone(), gen.join("glsl"), gen.join("glsl/glcpp")];
    let comun = |b: &mut cc::Build| {
        b.includes(&incluir)
            .define("PACKAGE_STRING", "\"uam 1.1.0\"")
            .define("DESKTOP", None)
            .define("_USE_MATH_DEFINES", None)
            .define("_GNU_SOURCE", None)
            .define("HAVE_POSIX_MEMALIGN", None)
            // always: Mesa's asserts abort
            .define("NDEBUG", None)
            .flag("-ffunction-sections")
            .flag("-fdata-sections")
            .flag("-w")
            .opt_level(2)
            .warnings(false);
    };
    let mut bc = cc::Build::new();
    comun(&mut bc);
    bc.std("c99").flag("-Werror=implicit-function-declaration").files(&c).compile("uam_c");
    let mut bx = cc::Build::new();
    comun(&mut bx);
    // c++11: the flex output uses 'register', gone in C++17
    bx.cpp(true).std("c++11").cpp_link_stdlib("stdc++").files(&cpp).compile("uam");
    println!("cargo:rerun-if-changed=uam_c.cpp");
}
