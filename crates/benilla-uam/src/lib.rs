//! wow-nx: uam, deko3d's shader compiler (GLSL 4.60 in, DKSH out), on the Switch.
//!
//! uam keeps global state and recurses deeply, so every compile runs on one thread of its own with an 8 MB stack;
//! [`compile`] hands it the work and waits. [`compile_cached`] keeps the results on the SD card
//! (`sdmc:/wow-nx/shaders/<hash>.dksh`, with its GLSL as `<hash>.glsl`), so a shader compiles once per console, and leaves the GLSL of a shader that
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
/// constants (`uniform T name;`) become uniform blocks in the driver's slots; and uam has no
/// `GL_EXT_texture_shadow_lod` ([`drop_shadow_lod`]); and a depth texture that is never compared is not a shadow
/// sampler ([`unshadow_uncompared`]). Returns whether the first instance and push constants are used.
pub fn adapt_glsl(glsl: &mut String) -> (bool, bool) {
    drop_shadow_lod(glsl);
    unshadow_uncompared(glsl);
    fold_dead_shadows(glsl);
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

/// naga asks for `GL_EXT_texture_shadow_lod` to sample a shadow array or cube at an explicit level, which uam does not
/// have. The core forms without the level (`texture`, `textureOffset`) sample the implicit level instead: the same
/// for shadow maps, which have one level (Bevy's do). Every `textureLod`/`textureLodOffset` on a `*ArrayShadow` or
/// `samplerCubeShadow` loses its level argument, and the extension line goes.
/// Bevy's PBR shader decides its shadow work at run time, from values that in this client never change, so uam keeps
/// all of it and allots registers for it: the PBR fragment shader was 104 registers and 29 KB of code. Folded to
/// constants, the compiler drops the dead paths (93 registers, 12.5 KB, measured with switch/shadercheck):
/// - a light's `soft_shadow_size`, which Bevy writes as 0 unless built with `experimental_pbr_pcss` (bevy_pbr's
///   light.rs), and this workspace is not: the PCSS search and filter go;
/// - a light's shadows-enabled flag: benilla never enables Bevy's shadow maps, as 1.12.1 has none (its blob shadows
///   are benilla's own), so every shadow-map fetch goes.
fn fold_dead_shadows(glsl: &mut String) {
    const SOFT: &str = ".soft_shadow_size;";
    const FLAG: &str = "_LIGHT_FLAGS_SHADOWS_ENABLED_BIT";
    if !glsl.contains(SOFT) && !glsl.contains(FLAG) {
        return;
    }
    let mut out = String::with_capacity(glsl.len());
    for line in glsl.split_inclusive('\n') {
        let body = line.trim_end_matches('\n');
        let trimmed = body.trim_start();
        // `float _e91 = _group_0_binding_8_fs.data[light_id_13].soft_shadow_size;`
        if trimmed.starts_with("float ") && trimmed.ends_with(SOFT) {
            if let Some(eq) = body.find(" = ") {
                out.push_str(&body[..eq]);
                out.push_str(" = 0.0;");
                out.push_str(&line[body.len()..]);
                continue;
            }
        }
        // `((_e136 & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BITX_naga_oil_mod_...) != 0u)`, not the constant's declaration
        let mut rest = body.to_string();
        if !trimmed.starts_with("const ") {
            while let Some(at) = rest.find(FLAG) {
                let Some(open) = rest[..at].rfind("((_e") else { break };
                let Some(close) = rest[at..].find(") != 0u)").map(|c| at + c + ") != 0u)".len()) else { break };
                rest.replace_range(open..close, "false");
            }
        }
        out.push_str(&rest);
        out.push_str(&line[body.len()..]);
    }
    *glsl = out;
}

fn drop_shadow_lod(glsl: &mut String) {
    const EXT: &str = "#extension GL_EXT_texture_shadow_lod : require\n";
    if !glsl.contains(EXT) {
        return;
    }
    *glsl = glsl.replace(EXT, "");
    let mut shadows = Vec::new();
    for kind in ["sampler2DArrayShadow ", "samplerCubeArrayShadow ", "samplerCubeShadow "] {
        let mut from = 0;
        while let Some(i) = glsl[from..].find(kind).map(|i| i + from) {
            let name: String = glsl[i + kind.len()..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                shadows.push(name);
            }
            from = i + kind.len();
        }
    }
    for (call, plain, level_from_end) in [("textureLodOffset(", "textureOffset(", 1), ("textureLod(", "texture(", 0)] {
        let mut from = 0;
        while let Some(i) = glsl[from..].find(call).map(|i| i + from) {
            let args = i + call.len();
            let first: String = glsl[args..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !shadows.contains(&first) {
                from = args;
                continue;
            }
            // The call's top-level commas and its closing parenthesis.
            let (mut depth, mut commas, mut end) = (0i32, Vec::new(), None);
            for (k, c) in glsl[args..].char_indices() {
                match c {
                    '(' | '[' => depth += 1,
                    ')' | ']' if depth > 0 => depth -= 1,
                    ')' => {
                        end = Some(args + k);
                        break;
                    }
                    ',' if depth == 0 => commas.push(args + k),
                    _ => {}
                }
            }
            let Some(end) = end else { break };
            // The level is the last argument, or the one before the offset.
            let n = commas.len();
            if n < 1 + level_from_end {
                from = args;
                continue;
            }
            let start = commas[n - 1 - level_from_end];
            let stop = if level_from_end == 0 { end } else { commas[n - 1] };
            glsl.replace_range(start..stop, "");
            glsl.replace_range(i..args, plain);
            from = i + plain.len();
        }
    }
}

/// naga declares every depth texture as a shadow sampler, even one only gathered or fetched without a comparison
/// (Bevy's depth downsampling: `textureGatherOffset(sampler2DShadow, uv, offset, 0)`), which GLSL has no overload for.
/// A shadow sampler whose every use is a gather with a component (an integer literal last), a fetch or a size query
/// is declared as the plain sampler of its shape instead; one that is compared anywhere stays as it is.
fn unshadow_uncompared(glsl: &mut String) {
    const KINDS: [(&str, &str); 3] =
        [("sampler2DArrayShadow ", "sampler2DArray "), ("samplerCubeShadow ", "samplerCube "), ("sampler2DShadow ", "sampler2D ")];
    const PLAIN_USES: [&str; 6] = ["textureGather", "textureGatherOffset", "texelFetch", "texelFetchOffset", "textureSize", "textureQueryLevels"];
    for (shadow, plain) in KINDS {
        let mut from = 0;
        while let Some(i) = glsl[from..].find(shadow).map(|i| i + from) {
            let start = i + shadow.len();
            let name: String = glsl[start..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            from = start;
            if name.is_empty() {
                continue;
            }
            let call_arg = format!("({name},");
            let mut uses = 0;
            let mut all_plain = true;
            let mut at = 0;
            while let Some(k) = glsl[at..].find(&call_arg).map(|k| k + at) {
                at = k + call_arg.len();
                uses += 1;
                let func: String = glsl[..k].chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<Vec<_>>().into_iter().rev().collect();
                if !PLAIN_USES.contains(&func.as_str()) {
                    all_plain = false;
                    break;
                }
                if func.starts_with("textureGather") {
                    // The last argument: a component (an integer literal) for a plain gather, a depth reference for a
                    // comparing one.
                    let (mut depth, mut last, mut end) = (0i32, at, None);
                    for (n, c) in glsl[at..].char_indices() {
                        match c {
                            '(' | '[' => depth += 1,
                            ')' | ']' if depth > 0 => depth -= 1,
                            ')' => {
                                end = Some(at + n);
                                break;
                            }
                            ',' if depth == 0 => last = at + n + 1,
                            _ => {}
                        }
                    }
                    let arg = end.map(|e| glsl[last..e].trim().to_string()).unwrap_or_default();
                    if arg.is_empty() || !arg.chars().all(|c| c.is_ascii_digit()) {
                        all_plain = false;
                        break;
                    }
                }
            }
            if uses > 0 && all_plain {
                glsl.replace_range(i..start, plain);
                from = i + plain.len();
            }
        }
    }
}
