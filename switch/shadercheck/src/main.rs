//! shadercheck: WGSL through naga's desktop GLSL and uam, as the Switch's deko3d backend compiles it
//! (benilla_uam::escribir_glsl), on the host. Bindings are numbered per kind across groups, as a pipeline layout made
//! from the shader's own bindings would number them. A failure's GLSL and uam's log go to target/shadercheck/.

use std::path::{Path, PathBuf};

fn wgsl_en(ruta: &Path, salida: &mut Vec<PathBuf>) {
    if ruta.is_dir() {
        let mut hijos: Vec<_> = std::fs::read_dir(ruta).into_iter().flatten().flatten().map(|e| e.path()).collect();
        hijos.sort();
        for h in hijos {
            wgsl_en(&h, salida);
        }
    } else if ruta.extension().is_some_and(|e| e == "wgsl") {
        salida.push(ruta.to_path_buf());
    }
}

/// Slots per kind (uniform, storage, texture, sampler, storage image), counted over (group, binding) in order.
fn mapa(module: &naga::Module) -> naga::back::glsl::BindingMap {
    let mut enlaces: Vec<(naga::ResourceBinding, usize)> = Vec::new();
    for (_, v) in module.global_variables.iter() {
        let Some(b) = v.binding.clone() else { continue };
        let tipo = match v.space {
            naga::AddressSpace::Uniform => 0,
            naga::AddressSpace::Storage { .. } => 1,
            naga::AddressSpace::Handle => match module.types[v.ty].inner {
                naga::TypeInner::Sampler { .. } => 3,
                naga::TypeInner::Image { class: naga::ImageClass::Storage { .. }, .. } => 4,
                _ => 2,
            },
            _ => continue,
        };
        enlaces.push((b, tipo));
    }
    enlaces.sort_by_key(|(b, _)| (b.group, b.binding));
    let mut cuentas = [0u8; 5];
    let mut mapa = naga::back::glsl::BindingMap::default();
    for (b, t) in enlaces {
        mapa.insert(b, cuentas[t]);
        cuentas[t] += 1;
    }
    mapa
}

fn main() {
    let mut ficheros = Vec::new();
    for a in std::env::args().skip(1) {
        wgsl_en(Path::new(&a), &mut ficheros);
    }
    let _ = std::fs::create_dir_all("target/shadercheck");
    let (mut bien, mut mal) = (0, 0);
    for f in &ficheros {
        let fuente = std::fs::read_to_string(f).unwrap_or_default();
        let module = match naga::front::wgsl::parse_str(&fuente) {
            Ok(m) => m,
            Err(e) => {
                println!("SKIP {} (WGSL: {})", f.display(), e.message());
                continue;
            }
        };
        let info = match naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
        {
            Ok(i) => i,
            Err(e) => {
                println!("SKIP {} (validation: {e})", f.display());
                continue;
            }
        };
        for ep in &module.entry_points {
            let etapa = match ep.stage {
                naga::ShaderStage::Vertex => benilla_uam::Etapa::Vertices,
                naga::ShaderStage::Fragment => benilla_uam::Etapa::Fragmentos,
                naga::ShaderStage::Compute => benilla_uam::Etapa::Computo,
                _ => continue,
            };
            let nombre = format!("{}:{}", f.display(), ep.name);
            // Overrides without a default are 0.
            let mut valores = naga::back::PipelineConstants::default();
            for (_, o) in module.overrides.iter() {
                if o.init.is_none() {
                    if let Some(n) = &o.name {
                        valores.insert(n.clone(), 0.0);
                    }
                }
            }
            let (module, info) = match naga::back::pipeline_constants::process_overrides(
                &module,
                &info,
                Some((ep.stage, &ep.name)),
                &valores,
            ) {
                Ok(r) => r,
                Err(e) => {
                    println!("SKIP {nombre} (overrides need values: {e})");
                    continue;
                }
            };
            let glsl = match benilla_uam::escribir_glsl(&module, &info, ep.stage, &ep.name, mapa(&module), true) {
                Ok((g, ..)) => g,
                Err(e) => {
                    println!("FAIL {nombre} (naga GLSL: {e})");
                    mal += 1;
                    continue;
                }
            };
            match benilla_uam::compilar(etapa, &glsl) {
                Ok(dksh) => {
                    println!("ok   {nombre} ({} bytes)", dksh.len());
                    bien += 1;
                }
                Err(log) => {
                    let archivo = format!("target/shadercheck/{}_{}.glsl", f.file_stem().unwrap().to_string_lossy(), ep.name);
                    let _ = std::fs::write(&archivo, format!("{glsl}\n/* uam:\n{log}\n*/\n"));
                    println!("FAIL {nombre}: {} -> {archivo}", log.lines().next().unwrap_or(""));
                    mal += 1;
                }
            }
        }
    }
    println!("{bien} ok, {mal} failed");
    std::process::exit(if mal > 0 { 1 } else { 0 });
}
