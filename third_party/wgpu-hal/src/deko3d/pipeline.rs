//! Shaders, layouts, bind groups, samplers and pipelines.
//!
//! Slots follow the GL backend: a pipeline layout numbers each kind of binding (uniform buffers, storage buffers,
//! textures, samplers, storage images) from 0 across its groups, and naga writes those numbers into the GLSL as
//! `binding = N`. naga combines each texture with its sampler; the combined sampler takes the texture's slot, and the
//! pipeline keeps which sampler slot goes with it. uam compiles the GLSL (on the console, cached on the SD card).
//! deko3d has no pipeline objects: a pipeline is the state it binds when it is set.

use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};

use naga::back::glsl;

use super::ffi::*;

/// The vertex stage's uniform buffer slot for the draw's first instance (`naga_vs_first_instance`): Maxwell's
/// `gl_InstanceID` does not include it, and WebGPU's `instance_index` does.
pub(super) const SLOT_PRIMERA_INSTANCIA: u32 = 15;

#[derive(Debug)]
pub struct ShaderModule {
    pub(super) naga: crate::NagaShader,
}

#[derive(Debug)]
pub struct BindGroupLayout {
    pub(super) entradas: Arc<[wgt::BindGroupLayoutEntry]>,
}

#[derive(Debug)]
pub(super) struct Grupo {
    pub(super) entradas: Arc<[wgt::BindGroupLayoutEntry]>,
    /// The slot of each binding number (`!0` for numbers the group does not use).
    pub(super) ranuras: Box<[u8]>,
}

#[derive(Debug)]
pub struct PipelineLayout {
    pub(super) grupos: Box<[Grupo]>,
    mapa: glsl::BindingMap,
}

impl PipelineLayout {
    fn ranura(&self, br: &naga::ResourceBinding) -> u8 {
        self.grupos[br.group as usize].ranuras[br.binding as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Tipo {
    Uniforme { gpu: DkGpuAddr, tamano: u32, dinamico: bool },
    Almacen { gpu: DkGpuAddr, tamano: u32, dinamico: bool },
    Textura { descriptor: u32 },
    Imagen { descriptor: u32 },
    Muestreador { descriptor: u32 },
}

/// One resource of a bind group. Its slot comes from the pipeline layout it is bound with: the binding's slot plus
/// `indice` (its place in a binding array).
#[derive(Clone, Copy, Debug)]
pub(super) struct Recurso {
    pub(super) binding: u32,
    pub(super) indice: u8,
    pub(super) tipo: Tipo,
}

#[derive(Debug)]
pub struct BindGroup {
    pub(super) recursos: Box<[Recurso]>,
}

#[derive(Debug)]
pub struct Sampler {
    pub(super) descriptor: u32,
}

/// One compiled stage: deko3d's shader object, its code in the device's code memory.
#[derive(Debug)]
pub(super) struct Etapa {
    pub(super) shader: Box<DkShader>,
}

/// What a pipeline's shaders read, besides buffers: per texture slot, the sampler slot it is combined with.
#[derive(Debug, Default, Clone)]
pub(super) struct Muestreo {
    pub(super) texturas: Vec<(u8, Option<u8>)>,
    pub(super) imagenes: Vec<u8>,
}

#[derive(Debug)]
pub struct RenderPipeline {
    pub(super) vs: Etapa,
    pub(super) fs: Option<Etapa>,
    pub(super) muestreo: Muestreo,
    pub(super) primera_instancia: bool,
    pub(super) primitiva: DkPrimitive,
    pub(super) tira: bool,
    pub(super) rasterizador: DkRasterizerState,
    pub(super) multimuestra: DkMultisampleState,
    pub(super) mascara_muestras: u32,
    pub(super) color: DkColorState,
    pub(super) escritura: DkColorWriteState,
    pub(super) mezclas: Vec<DkBlendState>,
    pub(super) profundidad: DkDepthStencilState,
    pub(super) sesgo: (f32, f32, f32),
    pub(super) plantilla: (u8, u8),
    pub(super) atributos: Vec<DkVtxAttribState>,
    pub(super) buferes: Vec<DkVtxBufferState>,
}

#[derive(Debug)]
pub struct ComputePipeline {
    pub(super) cs: Etapa,
    pub(super) muestreo: Muestreo,
}

unsafe impl Send for RenderPipeline {}
unsafe impl Sync for RenderPipeline {}
unsafe impl Send for ComputePipeline {}
unsafe impl Sync for ComputePipeline {}

impl super::Device {
    pub(super) fn nuevo_layout_grupos(&self, desc: &crate::BindGroupLayoutDescriptor) -> BindGroupLayout {
        BindGroupLayout {
            entradas: Arc::from(desc.entries),
        }
    }

    pub(super) fn nuevo_layout(&self, desc: &crate::PipelineLayoutDescriptor<BindGroupLayout>) -> PipelineLayout {
        let mut cuentas = [0u8; 5];
        let mut mapa = glsl::BindingMap::default();
        let mut grupos = Vec::with_capacity(desc.bind_group_layouts.len());
        for (g, bgl) in desc.bind_group_layouts.iter().enumerate() {
            let maximo = bgl.entradas.iter().map(|e| e.binding as usize + 1).max().unwrap_or(0);
            let mut ranuras = alloc::vec![!0u8; maximo].into_boxed_slice();
            for e in bgl.entradas.iter() {
                let tipo = match e.ty {
                    wgt::BindingType::Buffer { ty: wgt::BufferBindingType::Uniform, .. } => 0,
                    wgt::BindingType::Buffer { .. } => 1,
                    wgt::BindingType::Texture { .. } => 2,
                    wgt::BindingType::Sampler(_) => 3,
                    wgt::BindingType::StorageTexture { .. } => 4,
                    _ => continue,
                };
                ranuras[e.binding as usize] = cuentas[tipo];
                mapa.insert(naga::ResourceBinding { group: g as u32, binding: e.binding }, cuentas[tipo]);
                cuentas[tipo] += e.count.map_or(1, |c| c.get() as u8);
            }
            grupos.push(Grupo {
                entradas: bgl.entradas.clone(),
                ranuras,
            });
        }
        PipelineLayout {
            grupos: grupos.into_boxed_slice(),
            mapa,
        }
    }

    pub(super) fn nuevo_grupo(
        &self,
        desc: &crate::BindGroupDescriptor<BindGroupLayout, super::Buffer, Sampler, super::TextureView, super::Resource>,
    ) -> BindGroup {
        let mut recursos = Vec::new();
        for entrada in desc.entries {
            let Some(layout) = desc.layout.entradas.iter().find(|e| e.binding == entrada.binding) else {
                continue;
            };
            for k in 0..entrada.count.max(1) {
                let i = (entrada.resource_index + k) as usize;
                let tipo = match layout.ty {
                    wgt::BindingType::Buffer { ty, has_dynamic_offset, .. } => {
                        let b = &desc.buffers[i];
                        let tamano = b.size.map_or(b.buffer.size - b.offset, |s| s.get()) as u32;
                        let gpu = b.buffer.gpu + b.offset;
                        if ty == wgt::BufferBindingType::Uniform {
                            Tipo::Uniforme { gpu, tamano: tamano.min(DK_UNIFORM_BUF_MAX_SIZE), dinamico: has_dynamic_offset }
                        } else {
                            Tipo::Almacen { gpu, tamano, dinamico: has_dynamic_offset }
                        }
                    }
                    wgt::BindingType::Texture { .. } => Tipo::Textura {
                        descriptor: desc.textures[i].view.descriptor.unwrap_or(0),
                    },
                    wgt::BindingType::StorageTexture { .. } => Tipo::Imagen {
                        descriptor: desc.textures[i].view.descriptor.unwrap_or(0),
                    },
                    wgt::BindingType::Sampler(_) => Tipo::Muestreador {
                        descriptor: desc.samplers[i].descriptor,
                    },
                    _ => continue,
                };
                recursos.push(Recurso {
                    binding: entrada.binding,
                    indice: k as u8,
                    tipo,
                });
            }
        }
        BindGroup {
            recursos: recursos.into_boxed_slice(),
        }
    }

    pub(super) fn nuevo_muestreador(&self, desc: &crate::SamplerDescriptor) -> Result<Sampler, crate::DeviceError> {
        let mut s = unsafe { core::mem::zeroed::<DkSampler>() };
        unsafe { dkSamplerDefaults(&mut s) };
        let filtro = |f: wgt::FilterMode| match f {
            wgt::FilterMode::Nearest => DkFilter::DkFilter_Nearest,
            wgt::FilterMode::Linear => DkFilter::DkFilter_Linear,
        };
        s.minFilter = filtro(desc.min_filter);
        s.magFilter = filtro(desc.mag_filter);
        s.mipFilter = match desc.mipmap_filter {
            wgt::FilterMode::Nearest => DkMipFilter::DkMipFilter_Nearest,
            wgt::FilterMode::Linear => DkMipFilter::DkMipFilter_Linear,
        };
        for (d, m) in s.wrapMode.iter_mut().zip(desc.address_modes) {
            *d = match m {
                wgt::AddressMode::ClampToEdge => DkWrapMode::DkWrapMode_ClampToEdge,
                wgt::AddressMode::Repeat => DkWrapMode::DkWrapMode_Repeat,
                wgt::AddressMode::MirrorRepeat => DkWrapMode::DkWrapMode_MirroredRepeat,
                wgt::AddressMode::ClampToBorder => DkWrapMode::DkWrapMode_ClampToBorder,
            };
        }
        s.lodClampMin = desc.lod_clamp.start;
        s.lodClampMax = desc.lod_clamp.end;
        if let Some(c) = desc.compare {
            s.compareEnable = true;
            s.compareOp = comparacion(c);
        }
        s.maxAnisotropy = desc.anisotropy_clamp.max(1) as f32;
        let borde = match desc.border_color {
            Some(wgt::SamplerBorderColor::OpaqueBlack) => [0.0, 0.0, 0.0, 1.0],
            Some(wgt::SamplerBorderColor::OpaqueWhite) => [1.0; 4],
            _ => [0.0; 4],
        };
        for (d, v) in s.borderColor.iter_mut().zip(borde) {
            d.value_f = v;
        }
        let descriptor = self.compartido.descriptores.lock().nuevo_muestreador(&s).ok_or(crate::DeviceError::OutOfMemory)?;
        Ok(Sampler { descriptor })
    }

    /// GLSL for one stage, with the texture/sampler pairs naga combined.
    fn glsl(
        &self,
        etapa: &crate::ProgrammableStage<ShaderModule>,
        naga_etapa: naga::ShaderStage,
        layout: &PipelineLayout,
        muestreo: &mut Muestreo,
    ) -> Result<String, crate::PipelineError> {
        let wgt_etapa = etapa_wgt(naga_etapa);
        let (module, info) = naga::back::pipeline_constants::process_overrides(
            &etapa.module.naga.module,
            &etapa.module.naga.info,
            Some((naga_etapa, etapa.entry_point)),
            etapa.constants,
        )
        .map_err(|e| crate::PipelineError::PipelineConstants(wgt_etapa, format!("{e}")))?;
        let opciones = glsl::Options {
            version: glsl::Version::Desktop(450),
            writer_flags: glsl::WriterFlags::FORCE_POINT_SIZE,
            binding_map: layout.mapa.clone(),
            zero_initialize_workgroup_memory: etapa.zero_initialize_workgroup_memory,
        };
        let tuberia = glsl::PipelineOptions {
            shader_stage: naga_etapa,
            entry_point: etapa.entry_point.into(),
            multiview: None,
        };
        let politicas = naga::proc::BoundsCheckPolicies {
            image_load: naga::proc::BoundsCheckPolicy::ReadZeroSkipWrite,
            ..Default::default()
        };
        let mut salida = String::new();
        let reflejo = glsl::Writer::new(&mut salida, &module, &info, &opciones, &tuberia, politicas)
            .and_then(|mut w| w.write())
            .map_err(|e| crate::PipelineError::Linkage(wgt_etapa, format!("{e}")))?;
        for (_, m) in reflejo.texture_mapping {
            let var = &module.global_variables[m.texture];
            let ranura = layout.ranura(var.binding.as_ref().unwrap());
            let almacen = matches!(module.types[var.ty].inner, naga::TypeInner::Image { class: naga::ImageClass::Storage { .. }, .. });
            if almacen {
                if !muestreo.imagenes.contains(&ranura) {
                    muestreo.imagenes.push(ranura);
                }
                continue;
            }
            let muestreador = m.sampler.map(|s| layout.ranura(module.global_variables[s].binding.as_ref().unwrap()));
            if !muestreo.texturas.iter().any(|(t, _)| *t == ranura) {
                muestreo.texturas.push((ranura, muestreador));
            }
        }
        // A loose uniform has nowhere to live in deko3d: the first instance goes in a uniform block of its own.
        let suelto = format!("uniform uint {};", glsl::FIRST_INSTANCE_BINDING);
        if salida.contains(&suelto) {
            salida = salida.replace(
                &suelto,
                &format!(
                    "layout(std140, binding = {SLOT_PRIMERA_INSTANCIA}) uniform naga_vs_first_instance_block {{ uint {}; }};",
                    glsl::FIRST_INSTANCE_BINDING
                ),
            );
        }
        Ok(salida)
    }

    fn compilar(&self, glsl: &str, naga_etapa: naga::ShaderStage) -> Result<Etapa, crate::PipelineError> {
        let etapa = match naga_etapa {
            naga::ShaderStage::Vertex => benilla_uam::Etapa::Vertices,
            naga::ShaderStage::Fragment => benilla_uam::Etapa::Fragmentos,
            _ => benilla_uam::Etapa::Computo,
        };
        let dksh = benilla_uam::compilar_con_cache(etapa, glsl).map_err(|log| {
            log::error!("uam ({naga_etapa:?}): {log}\n{glsl}");
            crate::PipelineError::Linkage(etapa_wgt(naga_etapa), log)
        })?;
        let shader = self
            .compartido
            .codigo
            .lock()
            .cargar(&dksh)
            .ok_or_else(|| crate::PipelineError::Linkage(etapa_wgt(naga_etapa), "code memory full".into()))?;
        Ok(Etapa { shader })
    }

    pub(super) fn nueva_tuberia(
        &self,
        desc: &crate::RenderPipelineDescriptor<PipelineLayout, ShaderModule, super::Resource>,
    ) -> Result<RenderPipeline, crate::PipelineError> {
        let crate::VertexProcessor::Standard { vertex_buffers, vertex_stage } = &desc.vertex_processor else {
            return Err(crate::PipelineError::EntryPoint(naga::ShaderStage::Task));
        };
        let mut muestreo = Muestreo::default();
        let glsl_vs = self.glsl(vertex_stage, naga::ShaderStage::Vertex, desc.layout, &mut muestreo)?;
        let primera_instancia = glsl_vs.contains(glsl::FIRST_INSTANCE_BINDING);
        let vs = self.compilar(&glsl_vs, naga::ShaderStage::Vertex)?;
        let fs = match &desc.fragment_stage {
            Some(f) => {
                let g = self.glsl(f, naga::ShaderStage::Fragment, desc.layout, &mut muestreo)?;
                Some(self.compilar(&g, naga::ShaderStage::Fragment)?)
            }
            None => None,
        };

        let p = &desc.primitive;
        use wgt::PrimitiveTopology as T;
        let primitiva = match p.topology {
            T::PointList => DkPrimitive::DkPrimitive_Points,
            T::LineList => DkPrimitive::DkPrimitive_Lines,
            T::LineStrip => DkPrimitive::DkPrimitive_LineStrip,
            T::TriangleList => DkPrimitive::DkPrimitive_Triangles,
            T::TriangleStrip => DkPrimitive::DkPrimitive_TriangleStrip,
        };
        let mut rasterizador = unsafe { core::mem::zeroed::<DkRasterizerState>() };
        unsafe { dkRasterizerStateDefaults(&mut rasterizador) };
        rasterizador.set_cullMode(
            match p.cull_mode {
                None => DkFace::DkFace_None,
                Some(wgt::Face::Front) => DkFace::DkFace_Front,
                Some(wgt::Face::Back) => DkFace::DkFace_Back,
            },
        );
        rasterizador.set_frontFace(
            match p.front_face {
                wgt::FrontFace::Ccw => DkFrontFace::DkFrontFace_CCW,
                wgt::FrontFace::Cw => DkFrontFace::DkFrontFace_CW,
            },
        );
        let modo = match p.polygon_mode {
            wgt::PolygonMode::Fill => DkPolygonMode::DkPolygonMode_Fill,
            wgt::PolygonMode::Line => DkPolygonMode::DkPolygonMode_Line,
            wgt::PolygonMode::Point => DkPolygonMode::DkPolygonMode_Point,
        };
        rasterizador.set_polygonModeFront(modo);
        rasterizador.set_polygonModeBack(modo);
        rasterizador.set_depthClampEnable(p.unclipped_depth as u32);

        let mut multimuestra = unsafe { core::mem::zeroed::<DkMultisampleState>() };
        unsafe { dkMultisampleStateDefaults(&mut multimuestra) };
        multimuestra.set_mode(super::conv::muestras(desc.multisample.count));
        multimuestra.set_alphaToCoverageEnable(desc.multisample.alpha_to_coverage_enabled as u32);

        let mut color = unsafe { core::mem::zeroed::<DkColorState>() };
        unsafe { dkColorStateDefaults(&mut color) };
        let mut escritura = unsafe { core::mem::zeroed::<DkColorWriteState>() };
        unsafe { dkColorWriteStateDefaults(&mut escritura) };
        let mut mezclas = Vec::with_capacity(desc.color_targets.len());
        for (i, t) in desc.color_targets.iter().enumerate() {
            let mut m = unsafe { core::mem::zeroed::<DkBlendState>() };
            unsafe { dkBlendStateDefaults(&mut m) };
            let mascara = t.as_ref().map_or(0, |t| t.write_mask.bits());
            unsafe { dkColorWriteStateSetMask(&mut escritura, i as u32, mascara) };
            if let Some(b) = t.as_ref().and_then(|t| t.blend) {
                unsafe {
                    dkColorStateSetBlendEnable(&mut color, i as u32, true);
                    dkBlendStateSetOps(&mut m, operacion(b.color.operation), operacion(b.alpha.operation));
                    dkBlendStateSetFactors(
                        &mut m,
                        factor(b.color.src_factor),
                        factor(b.color.dst_factor),
                        factor(b.alpha.src_factor),
                        factor(b.alpha.dst_factor),
                    );
                }
            }
            mezclas.push(m);
        }

        let mut profundidad = unsafe { core::mem::zeroed::<DkDepthStencilState>() };
        unsafe { dkDepthStencilStateDefaults(&mut profundidad) };
        let (mut sesgo, mut plantilla) = ((0.0, 0.0, 0.0), (0xff, 0xff));
        match &desc.depth_stencil {
            Some(ds) => {
                let compara = ds.depth_compare;
                let escribe = ds.depth_write_enabled;
                profundidad.set_depthTestEnable((compara != wgt::CompareFunction::Always || escribe) as u32);
                profundidad.set_depthWriteEnable(escribe as u32);
                profundidad.set_depthCompareOp(comparacion(compara));
                if ds.stencil.is_enabled() {
                    profundidad.set_stencilTestEnable(1);
                    let (f, b) = (&ds.stencil.front, &ds.stencil.back);
                    profundidad.set_stencilFrontFailOp(op_plantilla(f.fail_op));
                    profundidad.set_stencilFrontPassOp(op_plantilla(f.pass_op));
                    profundidad.set_stencilFrontDepthFailOp(op_plantilla(f.depth_fail_op));
                    profundidad.set_stencilFrontCompareOp(comparacion(f.compare));
                    profundidad.set_stencilBackFailOp(op_plantilla(b.fail_op));
                    profundidad.set_stencilBackPassOp(op_plantilla(b.pass_op));
                    profundidad.set_stencilBackDepthFailOp(op_plantilla(b.depth_fail_op));
                    profundidad.set_stencilBackCompareOp(comparacion(b.compare));
                    plantilla = (ds.stencil.read_mask as u8, ds.stencil.write_mask as u8);
                }
                if ds.bias.is_enabled() {
                    rasterizador.set_depthBiasEnableMask(DkPolygonFlag_All);
                    sesgo = (ds.bias.constant as f32, ds.bias.clamp, ds.bias.slope_scale);
                }
            }
            None => {
                profundidad.set_depthTestEnable(0);
                profundidad.set_depthWriteEnable(0);
            }
        }

        let mut atributos = Vec::new();
        let mut buferes = Vec::new();
        for (i, vb) in vertex_buffers.iter().enumerate() {
            buferes.push(DkVtxBufferState {
                stride: vb.array_stride as u32,
                divisor: (vb.step_mode == wgt::VertexStepMode::Instance) as u32,
            });
            for a in vb.attributes {
                let (tamano, tipo, bgra) = formato_vertice(a.format);
                let mut s = unsafe { core::mem::zeroed::<DkVtxAttribState>() };
                s.set_bufferId(i as u32);
                s.set_isFixed(0);
                s.set_offset(a.offset as u32);
                s.set_size(tamano);
                s.set_type(tipo);
                s.set_isBgra(bgra as u32);
                let l = a.shader_location as usize;
                if atributos.len() <= l {
                    // Locations the shader does not read stay fixed (constant zero).
                    let mut fijo = unsafe { core::mem::zeroed::<DkVtxAttribState>() };
                    fijo.set_isFixed(1);
                    fijo.set_size(DkVtxAttribSize::DkVtxAttribSize_1x32);
                    fijo.set_type(DkVtxAttribType::DkVtxAttribType_Float);
                    atributos.resize(l + 1, fijo);
                }
                atributos[l] = s;
            }
        }

        Ok(RenderPipeline {
            vs,
            fs,
            muestreo,
            primera_instancia,
            primitiva,
            tira: p.strip_index_format.is_some() || matches!(p.topology, T::LineStrip | T::TriangleStrip),
            rasterizador,
            multimuestra,
            mascara_muestras: desc.multisample.mask as u32,
            color,
            escritura,
            mezclas,
            profundidad,
            sesgo,
            plantilla,
            atributos,
            buferes,
        })
    }

    pub(super) fn nueva_tuberia_computo(
        &self,
        desc: &crate::ComputePipelineDescriptor<PipelineLayout, ShaderModule, super::Resource>,
    ) -> Result<ComputePipeline, crate::PipelineError> {
        let mut muestreo = Muestreo::default();
        let g = self.glsl(&desc.stage, naga::ShaderStage::Compute, desc.layout, &mut muestreo)?;
        Ok(ComputePipeline {
            cs: self.compilar(&g, naga::ShaderStage::Compute)?,
            muestreo,
        })
    }
}

fn etapa_wgt(e: naga::ShaderStage) -> wgt::ShaderStages {
    match e {
        naga::ShaderStage::Vertex => wgt::ShaderStages::VERTEX,
        naga::ShaderStage::Fragment => wgt::ShaderStages::FRAGMENT,
        _ => wgt::ShaderStages::COMPUTE,
    }
}

pub(super) fn comparacion(c: wgt::CompareFunction) -> DkCompareOp {
    use wgt::CompareFunction as C;
    use DkCompareOp as D;
    match c {
        C::Never => D::DkCompareOp_Never,
        C::Less => D::DkCompareOp_Less,
        C::Equal => D::DkCompareOp_Equal,
        C::LessEqual => D::DkCompareOp_Lequal,
        C::Greater => D::DkCompareOp_Greater,
        C::NotEqual => D::DkCompareOp_NotEqual,
        C::GreaterEqual => D::DkCompareOp_Gequal,
        C::Always => D::DkCompareOp_Always,
    }
}

fn op_plantilla(o: wgt::StencilOperation) -> DkStencilOp {
    use wgt::StencilOperation as S;
    use DkStencilOp as D;
    match o {
        S::Keep => D::DkStencilOp_Keep,
        S::Zero => D::DkStencilOp_Zero,
        S::Replace => D::DkStencilOp_Replace,
        S::Invert => D::DkStencilOp_Invert,
        S::IncrementClamp => D::DkStencilOp_Incr,
        S::DecrementClamp => D::DkStencilOp_Decr,
        S::IncrementWrap => D::DkStencilOp_IncrWrap,
        S::DecrementWrap => D::DkStencilOp_DecrWrap,
    }
}

fn operacion(o: wgt::BlendOperation) -> DkBlendOp {
    use wgt::BlendOperation as B;
    use DkBlendOp as D;
    match o {
        B::Add => D::DkBlendOp_Add,
        B::Subtract => D::DkBlendOp_Sub,
        B::ReverseSubtract => D::DkBlendOp_RevSub,
        B::Min => D::DkBlendOp_Min,
        B::Max => D::DkBlendOp_Max,
    }
}

fn factor(f: wgt::BlendFactor) -> DkBlendFactor {
    use wgt::BlendFactor as B;
    use DkBlendFactor as D;
    match f {
        B::Zero => D::DkBlendFactor_Zero,
        B::One => D::DkBlendFactor_One,
        B::Src => D::DkBlendFactor_SrcColor,
        B::OneMinusSrc => D::DkBlendFactor_InvSrcColor,
        B::SrcAlpha => D::DkBlendFactor_SrcAlpha,
        B::OneMinusSrcAlpha => D::DkBlendFactor_InvSrcAlpha,
        B::Dst => D::DkBlendFactor_DstColor,
        B::OneMinusDst => D::DkBlendFactor_InvDstColor,
        B::DstAlpha => D::DkBlendFactor_DstAlpha,
        B::OneMinusDstAlpha => D::DkBlendFactor_InvDstAlpha,
        B::SrcAlphaSaturated => D::DkBlendFactor_SrcAlphaSaturate,
        B::Constant => D::DkBlendFactor_ConstColor,
        B::OneMinusConstant => D::DkBlendFactor_InvConstColor,
        B::Src1 => D::DkBlendFactor_Src1Color,
        B::OneMinusSrc1 => D::DkBlendFactor_InvSrc1Color,
        B::Src1Alpha => D::DkBlendFactor_Src1Alpha,
        B::OneMinusSrc1Alpha => D::DkBlendFactor_InvSrc1Alpha,
    }
}

fn formato_vertice(f: wgt::VertexFormat) -> (DkVtxAttribSize, DkVtxAttribType, bool) {
    use wgt::VertexFormat as V;
    use DkVtxAttribSize as S;
    use DkVtxAttribType as T;
    let (s, t) = match f {
        V::Uint8 => (S::DkVtxAttribSize_1x8, T::DkVtxAttribType_Uint),
        V::Uint8x2 => (S::DkVtxAttribSize_2x8, T::DkVtxAttribType_Uint),
        V::Uint8x4 => (S::DkVtxAttribSize_4x8, T::DkVtxAttribType_Uint),
        V::Sint8 => (S::DkVtxAttribSize_1x8, T::DkVtxAttribType_Sint),
        V::Sint8x2 => (S::DkVtxAttribSize_2x8, T::DkVtxAttribType_Sint),
        V::Sint8x4 => (S::DkVtxAttribSize_4x8, T::DkVtxAttribType_Sint),
        V::Unorm8 => (S::DkVtxAttribSize_1x8, T::DkVtxAttribType_Unorm),
        V::Unorm8x2 => (S::DkVtxAttribSize_2x8, T::DkVtxAttribType_Unorm),
        V::Unorm8x4 => (S::DkVtxAttribSize_4x8, T::DkVtxAttribType_Unorm),
        V::Snorm8 => (S::DkVtxAttribSize_1x8, T::DkVtxAttribType_Snorm),
        V::Snorm8x2 => (S::DkVtxAttribSize_2x8, T::DkVtxAttribType_Snorm),
        V::Snorm8x4 => (S::DkVtxAttribSize_4x8, T::DkVtxAttribType_Snorm),
        V::Uint16 => (S::DkVtxAttribSize_1x16, T::DkVtxAttribType_Uint),
        V::Uint16x2 => (S::DkVtxAttribSize_2x16, T::DkVtxAttribType_Uint),
        V::Uint16x4 => (S::DkVtxAttribSize_4x16, T::DkVtxAttribType_Uint),
        V::Sint16 => (S::DkVtxAttribSize_1x16, T::DkVtxAttribType_Sint),
        V::Sint16x2 => (S::DkVtxAttribSize_2x16, T::DkVtxAttribType_Sint),
        V::Sint16x4 => (S::DkVtxAttribSize_4x16, T::DkVtxAttribType_Sint),
        V::Unorm16 => (S::DkVtxAttribSize_1x16, T::DkVtxAttribType_Unorm),
        V::Unorm16x2 => (S::DkVtxAttribSize_2x16, T::DkVtxAttribType_Unorm),
        V::Unorm16x4 => (S::DkVtxAttribSize_4x16, T::DkVtxAttribType_Unorm),
        V::Snorm16 => (S::DkVtxAttribSize_1x16, T::DkVtxAttribType_Snorm),
        V::Snorm16x2 => (S::DkVtxAttribSize_2x16, T::DkVtxAttribType_Snorm),
        V::Snorm16x4 => (S::DkVtxAttribSize_4x16, T::DkVtxAttribType_Snorm),
        V::Float16 => (S::DkVtxAttribSize_1x16, T::DkVtxAttribType_Float),
        V::Float16x2 => (S::DkVtxAttribSize_2x16, T::DkVtxAttribType_Float),
        V::Float16x4 => (S::DkVtxAttribSize_4x16, T::DkVtxAttribType_Float),
        V::Float32 => (S::DkVtxAttribSize_1x32, T::DkVtxAttribType_Float),
        V::Float32x2 => (S::DkVtxAttribSize_2x32, T::DkVtxAttribType_Float),
        V::Float32x3 => (S::DkVtxAttribSize_3x32, T::DkVtxAttribType_Float),
        V::Float32x4 => (S::DkVtxAttribSize_4x32, T::DkVtxAttribType_Float),
        V::Uint32 => (S::DkVtxAttribSize_1x32, T::DkVtxAttribType_Uint),
        V::Uint32x2 => (S::DkVtxAttribSize_2x32, T::DkVtxAttribType_Uint),
        V::Uint32x3 => (S::DkVtxAttribSize_3x32, T::DkVtxAttribType_Uint),
        V::Uint32x4 => (S::DkVtxAttribSize_4x32, T::DkVtxAttribType_Uint),
        V::Sint32 => (S::DkVtxAttribSize_1x32, T::DkVtxAttribType_Sint),
        V::Sint32x2 => (S::DkVtxAttribSize_2x32, T::DkVtxAttribType_Sint),
        V::Sint32x3 => (S::DkVtxAttribSize_3x32, T::DkVtxAttribType_Sint),
        V::Sint32x4 => (S::DkVtxAttribSize_4x32, T::DkVtxAttribType_Sint),
        V::Unorm10_10_10_2 => (S::DkVtxAttribSize_10_10_10_2, T::DkVtxAttribType_Unorm),
        V::Unorm8x4Bgra => return (S::DkVtxAttribSize_4x8, T::DkVtxAttribType_Unorm, true),
        // 64-bit attributes are not a WebGPU feature this backend offers.
        _ => (S::DkVtxAttribSize_1x32, T::DkVtxAttribType_Float),
    };
    (s, t, false)
}
