//! shadercheck: WGSL (or the backend's GLSL, `.glsl`) through naga's desktop GLSL and uam, as the Switch's deko3d backend compiles it
//! (benilla_uam::write_glsl), on the host. Bindings are numbered per kind across groups, as a pipeline layout made
//! from the shader's own bindings would number them. A failure's GLSL and uam's log go to target/shadercheck/.

use std::path::{Path, PathBuf};

fn collect_shaders(path: &Path, output: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut children: Vec<_> = std::fs::read_dir(path).into_iter().flatten().flatten().map(|e| e.path()).collect();
        children.sort();
        for h in children {
            collect_shaders(&h, output);
        }
    } else if path.extension().is_some_and(|e| e == "wgsl" || e == "glsl") {
        output.push(path.to_path_buf());
    }
}

/// Slots per kind (uniform, storage, texture, sampler, storage image), counted over (group, binding) in order.
fn binding_map(module: &naga::Module) -> naga::back::glsl::BindingMap {
    let mut bindings: Vec<(naga::ResourceBinding, usize)> = Vec::new();
    for (_, v) in module.global_variables.iter() {
        let Some(b) = v.binding.clone() else { continue };
        let kind = match v.space {
            naga::AddressSpace::Uniform => 0,
            naga::AddressSpace::Storage { .. } => 1,
            naga::AddressSpace::Handle => match module.types[v.ty].inner {
                naga::TypeInner::Sampler { .. } => 3,
                naga::TypeInner::Image { class: naga::ImageClass::Storage { .. }, .. } => 4,
                _ => 2,
            },
            _ => continue,
        };
        bindings.push((b, kind));
    }
    bindings.sort_by_key(|(b, _)| (b.group, b.binding));
    let mut counts = [0u8; 5];
    let mut binding_map = naga::back::glsl::BindingMap::default();
    for (b, t) in bindings {
        binding_map.insert(b, counts[t]);
        counts[t] += 1;
    }
    binding_map
}

/// The program's registers a thread, scratch a warp (registers spilled to local memory) and code size, from the DKSH
/// headers (deko3d's dksh.h): what decides how many warps a GPU core keeps in flight.
fn program_cost(dksh: &[u8]) -> String {
    let word = |at: usize| dksh.get(at..at + 4).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap()));
    let (code, programs) = (word(12), word(16) as usize);
    format!("{} registers, {} scratch, {code} code", word(programs + 8), word(programs + 20))
}

fn main() {
    let mut files = Vec::new();
    for a in std::env::args().skip(1) {
        collect_shaders(Path::new(&a), &mut files);
    }
    let _ = std::fs::create_dir_all("target/shadercheck");
    let (mut passed, mut failed) = (0, 0);
    for f in &files {
        let source = std::fs::read_to_string(f).unwrap_or_default();
        if f.extension().is_some_and(|e| e == "glsl") {
            // GLSL as the backend wrote it (a console `.error.glsl`): uam's log appended as a comment is cut off,
            // the fix-ups run again, and the stage comes from the source.
            let mut glsl = source.split("\n/* uam:").next().unwrap_or("").to_string();
            benilla_uam::adapt_glsl(&mut glsl);
            let stage = if glsl.contains("layout(local_size_x") {
                benilla_uam::Stage::Compute
            } else if glsl.contains("gl_Position") {
                benilla_uam::Stage::Vertex
            } else {
                benilla_uam::Stage::Fragment
            };
            match benilla_uam::compile(stage, &glsl) {
                Ok(dksh) => {
                    println!("ok   {} ({stage:?}, {} bytes, {})", f.display(), dksh.len(), program_cost(&dksh));
                    passed += 1;
                }
                Err(log) => {
                    println!("FAIL {} ({stage:?}): {}", f.display(), log.lines().find(|l| !l.trim().is_empty()).unwrap_or(""));
                    failed += 1;
                }
            }
            continue;
        }
        let module = match naga::front::wgsl::parse_str(&source) {
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
            let stage = match ep.stage {
                naga::ShaderStage::Vertex => benilla_uam::Stage::Vertex,
                naga::ShaderStage::Fragment => benilla_uam::Stage::Fragment,
                naga::ShaderStage::Compute => benilla_uam::Stage::Compute,
                _ => continue,
            };
            let name = format!("{}:{}", f.display(), ep.name);
            // Overrides without a default are 0.
            let mut values = naga::back::PipelineConstants::default();
            for (_, o) in module.overrides.iter() {
                if o.init.is_none() {
                    if let Some(n) = &o.name {
                        values.insert(n.clone(), 0.0);
                    }
                }
            }
            let (module, info) = match naga::back::pipeline_constants::process_overrides(
                &module,
                &info,
                Some((ep.stage, &ep.name)),
                &values,
            ) {
                Ok(r) => r,
                Err(e) => {
                    println!("SKIP {name} (overrides need values: {e})");
                    continue;
                }
            };
            let glsl = match benilla_uam::write_glsl(&module, &info, ep.stage, &ep.name, binding_map(&module), true) {
                Ok((g, ..)) => g,
                Err(e) => {
                    println!("FAIL {name} (naga GLSL: {e})");
                    failed += 1;
                    continue;
                }
            };
            match benilla_uam::compile(stage, &glsl) {
                Ok(dksh) => {
                    println!("ok   {name} ({} bytes, {})", dksh.len(), program_cost(&dksh));
                    passed += 1;
                }
                Err(log) => {
                    let file = format!("target/shadercheck/{}_{}.glsl", f.file_stem().unwrap().to_string_lossy(), ep.name);
                    let _ = std::fs::write(&file, format!("{glsl}\n/* uam:\n{log}\n*/\n"));
                    println!("FAIL {name}: {} -> {file}", log.lines().next().unwrap_or(""));
                    failed += 1;
                }
            }
        }
    }
    println!("{passed} ok, {failed} failed");
    std::process::exit(if failed > 0 { 1 } else { 0 });
}
