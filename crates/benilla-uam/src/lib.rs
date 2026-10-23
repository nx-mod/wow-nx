//! wow-nx: uam, deko3d's shader compiler (GLSL 4.60 in, DKSH out), on the Switch.
//!
//! uam keeps global state and recurses deeply, so every compile runs on one thread of its own with an 8 MB stack;
//! [`compilar`] hands it the work and waits. [`compilar_con_cache`] keeps the results on the SD card
//! (`sdmc:/wow-nx/shaders/<hash>.dksh`), so a shader compiles once per console, and leaves the GLSL of a shader that
//! fails beside them (`<hash>.error.glsl`, with uam's log at the end).

#[cfg(any(target_os = "horizon", all(target_os = "linux", feature = "host")))]
mod compilar;
#[cfg(any(target_os = "horizon", all(target_os = "linux", feature = "host")))]
pub use compilar::*;

/// The uniform buffer slot of the first instance (`naga_vs_first_instance`), in the vertex stage.
pub const SLOT_PRIMERA_INSTANCIA: u32 = 15;
/// The uniform buffer slot of push constants, in every stage.
pub const SLOT_CONSTANTES: u32 = 14;

/// What naga's desktop GLSL needs before uam takes it: deko3d has no loose uniforms, so the first instance and push
/// constants (`uniform T name;`) become uniform blocks in the driver's slots. Returns whether each one was used.
pub fn adaptar_glsl(glsl: &mut String) -> (bool, bool) {
    const PRIMERA: &str = "naga_vs_first_instance";
    let suelto = format!("uniform uint {PRIMERA};");
    let primera = glsl.contains(&suelto);
    if primera {
        *glsl = glsl.replace(
            &suelto,
            &format!("layout(std140, binding = {SLOT_PRIMERA_INSTANCIA}) uniform {PRIMERA}_block {{ uint {PRIMERA}; }};"),
        );
    }
    let mut constantes = false;
    let mut desde = 0;
    while let Some(i) = glsl[desde..].find("\nuniform ").map(|i| i + desde) {
        let fin = i + glsl[i..].find(';').unwrap_or(0);
        let decl = &glsl[i + 9..fin];
        desde = fin;
        let Some((tipo, nombre)) = decl.split_once(' ') else { continue };
        if !nombre.starts_with("_push_constant_binding_") {
            continue;
        }
        let bloque = format!("\nlayout(std140, binding = {SLOT_CONSTANTES}) uniform {nombre}_block {{ {tipo} {nombre}; }}");
        glsl.replace_range(i..fin, &bloque);
        desde = i + bloque.len();
        constantes = true;
    }
    (primera, constantes)
}

/// naga's desktop GLSL for one entry point, as uam takes it (`binding = N` from `mapa`), already through
/// [`adaptar_glsl`]. Returns the GLSL, naga's reflection, and whether the first instance and push constants are used.
pub fn escribir_glsl(
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
    etapa: naga::ShaderStage,
    entrada: &str,
    mapa: naga::back::glsl::BindingMap,
    ceros_en_grupo: bool,
) -> Result<(String, naga::back::glsl::ReflectionInfo, bool, bool), String> {
    use naga::back::glsl;
    let opciones = glsl::Options {
        version: glsl::Version::Desktop(450),
        writer_flags: glsl::WriterFlags::FORCE_POINT_SIZE,
        binding_map: mapa,
        zero_initialize_workgroup_memory: ceros_en_grupo,
    };
    let tuberia = glsl::PipelineOptions {
        shader_stage: etapa,
        entry_point: entrada.into(),
        multiview: None,
    };
    let politicas = naga::proc::BoundsCheckPolicies {
        image_load: naga::proc::BoundsCheckPolicy::ReadZeroSkipWrite,
        ..Default::default()
    };
    let mut salida = String::new();
    let reflejo = glsl::Writer::new(&mut salida, module, info, &opciones, &tuberia, politicas)
        .and_then(|mut w| w.write())
        .map_err(|e| e.to_string())?;
    let (primera, constantes) = adaptar_glsl(&mut salida);
    Ok((salida, reflejo, primera, constantes))
}
