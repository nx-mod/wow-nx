//! wow-nx: uam, deko3d's shader compiler (GLSL 4.60 in, DKSH out), on the Switch.
//!
//! uam keeps global state and recurses deeply, so every compile runs on one thread of its own with an 8 MB stack;
//! [`compile`] hands it the work and waits. [`compile_cached`] keeps the results on the SD card
//! (`sdmc:/wow-nx/shaders/<hash>.dksh`), so a shader compiles once per console, and leaves the GLSL of a shader that
//! fails beside them (`<hash>.error.glsl`, with uam's log at the end).

#[cfg(any(target_os = "horizon", all(target_os = "linux", feature = "host")))]
mod compiler;
#[cfg(any(target_os = "horizon", all(target_os = "linux", feature = "host")))]
pub use compiler::*;

/// The uniform buffer slot of the first instance (`naga_vs_first_instance`), in the vertex stage.
pub const FIRST_INSTANCE_SLOT: u32 = 15;
/// The uniform buffer slot of push constants, in every stage.
pub const PUSH_CONSTANT_SLOT: u32 = 14;

/// What naga's desktop GLSL needs before uam takes it: deko3d has no loose uniforms, so the first instance and push
/// constants (`uniform T name;`) become uniform blocks in the driver's slots. Returns whether each one was used.
pub fn adapt_glsl(glsl: &mut String) -> (bool, bool) {
    const FIRST: &str = "naga_vs_first_instance";
    let loose = format!("uniform uint {FIRST};");
    let first = glsl.contains(&loose);
    if first {
        *glsl = glsl.replace(
            &loose,
            &format!("layout(std140, binding = {FIRST_INSTANCE_SLOT}) uniform {FIRST}_block {{ uint {FIRST}; }};"),
        );
    }
    let mut push_constants = false;
    let mut from = 0;
    while let Some(i) = glsl[from..].find("\nuniform ").map(|i| i + from) {
        let fin = i + glsl[i..].find(';').unwrap_or(0);
        let decl = &glsl[i + 9..fin];
        from = fin;
        let Some((kind, name)) = decl.split_once(' ') else { continue };
        if !name.starts_with("_push_constant_binding_") {
            continue;
        }
        let mem_block = format!("\nlayout(std140, binding = {PUSH_CONSTANT_SLOT}) uniform {name}_block {{ {kind} {name}; }}");
        glsl.replace_range(i..fin, &mem_block);
        from = i + mem_block.len();
        push_constants = true;
    }
    (first, push_constants)
}

/// naga's desktop GLSL for one entry point, as uam takes it (`binding = N` from `binding_map`), already through
/// [`adapt_glsl`]. Returns the GLSL, naga's reflection, and whether the first instance and push constants are used.
pub fn write_glsl(
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
    stage: naga::ShaderStage,
    entry: &str,
    binding_map: naga::back::glsl::BindingMap,
    zero_workgroup_memory: bool,
) -> Result<(String, naga::back::glsl::ReflectionInfo, bool, bool), String> {
    use naga::back::glsl;
    let options = glsl::Options {
        version: glsl::Version::Desktop(450),
        writer_flags: glsl::WriterFlags::FORCE_POINT_SIZE,
        binding_map: binding_map,
        zero_initialize_workgroup_memory: zero_workgroup_memory,
    };
    let pipeline = glsl::PipelineOptions {
        shader_stage: stage,
        entry_point: entry.into(),
        multiview: None,
    };
    let policies = naga::proc::BoundsCheckPolicies {
        image_load: naga::proc::BoundsCheckPolicy::ReadZeroSkipWrite,
        ..Default::default()
    };
    let mut output = String::new();
    let reflection = glsl::Writer::new(&mut output, module, info, &options, &pipeline, policies)
        .and_then(|mut w| w.write())
        .map_err(|e| e.to_string())?;
    let (first, push_constants) = adapt_glsl(&mut output);
    Ok((output, reflection, first, push_constants))
}
