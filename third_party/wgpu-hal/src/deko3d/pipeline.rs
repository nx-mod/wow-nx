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
pub(super) const FIRST_INSTANCE_SLOT: u32 = benilla_uam::FIRST_INSTANCE_SLOT;
/// Every stage's uniform buffer slot for push constants, which naga writes as a loose uniform.
pub(super) const PUSH_CONSTANT_SLOT: u32 = benilla_uam::PUSH_CONSTANT_SLOT;
/// Where they live in the driver's uniform buffer (the first instance is at 0).
pub(super) const PUSH_CONSTANT_OFFSET: u64 = 256;
pub(super) const PUSH_CONSTANT_SIZE: u32 = 256;

#[derive(Debug)]
pub struct ShaderModule {
    pub(super) naga: crate::NagaShader,
}

#[derive(Debug)]
pub struct BindGroupLayout {
    pub(super) entries: Arc<[wgt::BindGroupLayoutEntry]>,
}

#[derive(Debug)]
pub(super) struct GroupLayout {
    pub(super) entries: Arc<[wgt::BindGroupLayoutEntry]>,
    /// The slot of each binding number (`!0` for numbers the group does not use).
    pub(super) slots: Box<[u8]>,
}

#[derive(Debug)]
pub struct PipelineLayout {
    pub(super) groups: Box<[GroupLayout]>,
    binding_map: glsl::BindingMap,
}

impl PipelineLayout {
    fn slot(&self, br: &naga::ResourceBinding) -> u8 {
        self.groups[br.group as usize].slots[br.binding as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Kind {
    Uniform { gpu: DkGpuAddr, size: u32, dynamic: bool },
    Storage { gpu: DkGpuAddr, size: u32, dynamic: bool },
    SampledTexture { descriptor: u32 },
    StorageImage { descriptor: u32 },
    BoundSampler { descriptor: u32 },
}

/// One resource of a bind group. Its slot comes from the pipeline layout it is bound with: the binding's slot plus
/// `array_index` (its place in a binding array).
#[derive(Clone, Copy, Debug)]
pub(super) struct BoundResource {
    pub(super) binding: u32,
    pub(super) array_index: u8,
    pub(super) kind: Kind,
}

#[derive(Debug)]
pub struct BindGroup {
    pub(super) resources: Box<[BoundResource]>,
}

#[derive(Debug)]
pub struct Sampler {
    pub(super) descriptor: u32,
}

/// One compiled stage: deko3d's shader object, its code in the device's code memory.
#[derive(Debug)]
pub(super) struct CompiledStage {
    pub(super) shader: Box<DkShader>,
}

/// What a pipeline's shaders read, besides buffers: per texture slot, the sampler slot it is combined with.
#[derive(Debug, Default, Clone)]
pub(super) struct Sampling {
    pub(super) textures: Vec<(u8, Option<u8>)>,
    pub(super) images: Vec<u8>,
}

#[derive(Debug)]
pub struct RenderPipeline {
    pub(super) vs: CompiledStage,
    pub(super) fs: Option<CompiledStage>,
    pub(super) sampling: Sampling,
    pub(super) first_instance: bool,
    pub(super) push_constants: bool,
    pub(super) primitive: DkPrimitive,
    pub(super) strip: bool,
    pub(super) rasterizer: DkRasterizerState,
    pub(super) multisample: DkMultisampleState,
    pub(super) sample_mask: u32,
    pub(super) color: DkColorState,
    pub(super) color_write: DkColorWriteState,
    pub(super) blends: Vec<DkBlendState>,
    pub(super) depth: DkDepthStencilState,
    pub(super) depth_bias: (f32, f32, f32),
    pub(super) stencil: (u8, u8),
    pub(super) attribs: Vec<DkVtxAttribState>,
    pub(super) vertex_buffers: Vec<DkVtxBufferState>,
    /// Its label, for the per-pipeline GPU times (`CommandEncoder::begin_pipeline_timing`).
    pub(super) label: alloc::string::String,
}

#[derive(Debug)]
pub struct ComputePipeline {
    pub(super) cs: CompiledStage,
    pub(super) push_constants: bool,
    pub(super) sampling: Sampling,
    pub(super) label: alloc::string::String,
}

unsafe impl Send for RenderPipeline {}
unsafe impl Sync for RenderPipeline {}
unsafe impl Send for ComputePipeline {}
unsafe impl Sync for ComputePipeline {}

impl super::Device {
    pub(super) fn new_bind_group_layout(&self, desc: &crate::BindGroupLayoutDescriptor) -> BindGroupLayout {
        BindGroupLayout {
            entries: Arc::from(desc.entries),
        }
    }

    pub(super) fn new_pipeline_layout(&self, desc: &crate::PipelineLayoutDescriptor<BindGroupLayout>) -> PipelineLayout {
        let mut counts = [0u8; 5];
        let mut binding_map = glsl::BindingMap::default();
        let mut groups = Vec::with_capacity(desc.bind_group_layouts.len());
        for (g, bgl) in desc.bind_group_layouts.iter().enumerate() {
            let max_binding = bgl.entries.iter().map(|e| e.binding as usize + 1).max().unwrap_or(0);
            let mut slots = alloc::vec![!0u8; max_binding].into_boxed_slice();
            for e in bgl.entries.iter() {
                let kind = match e.ty {
                    wgt::BindingType::Buffer { ty: wgt::BufferBindingType::Uniform, .. } => 0,
                    wgt::BindingType::Buffer { .. } => 1,
                    wgt::BindingType::Texture { .. } => 2,
                    wgt::BindingType::Sampler(_) => 3,
                    wgt::BindingType::StorageTexture { .. } => 4,
                    _ => continue,
                };
                slots[e.binding as usize] = counts[kind];
                binding_map.insert(naga::ResourceBinding { group: g as u32, binding: e.binding }, counts[kind]);
                counts[kind] += e.count.map_or(1, |c| c.get() as u8);
            }
            groups.push(GroupLayout {
                entries: bgl.entries.clone(),
                slots,
            });
        }
        PipelineLayout {
            groups: groups.into_boxed_slice(),
            binding_map,
        }
    }

    pub(super) fn new_bind_group(
        &self,
        desc: &crate::BindGroupDescriptor<BindGroupLayout, super::Buffer, Sampler, super::TextureView, super::Resource>,
    ) -> BindGroup {
        let mut resources = Vec::new();
        for entry in desc.entries {
            let Some(layout) = desc.layout.entries.iter().find(|e| e.binding == entry.binding) else {
                continue;
            };
            for k in 0..entry.count.max(1) {
                let i = (entry.resource_index + k) as usize;
                let kind = match layout.ty {
                    wgt::BindingType::Buffer { ty, has_dynamic_offset, .. } => {
                        let b = &desc.buffers[i];
                        let size = b.size.map_or(b.buffer.size - b.offset, |s| s.get()) as u32;
                        let gpu = b.buffer.gpu + b.offset;
                        if ty == wgt::BufferBindingType::Uniform {
                            Kind::Uniform { gpu, size: size.min(DK_UNIFORM_BUF_MAX_SIZE), dynamic: has_dynamic_offset }
                        } else {
                            Kind::Storage { gpu, size, dynamic: has_dynamic_offset }
                        }
                    }
                    wgt::BindingType::Texture { .. } => Kind::SampledTexture {
                        descriptor: desc.textures[i].view.descriptor.unwrap_or(0),
                    },
                    wgt::BindingType::StorageTexture { .. } => Kind::StorageImage {
                        descriptor: desc.textures[i].view.descriptor.unwrap_or(0),
                    },
                    wgt::BindingType::Sampler(_) => Kind::BoundSampler {
                        descriptor: desc.samplers[i].descriptor,
                    },
                    _ => continue,
                };
                resources.push(BoundResource {
                    binding: entry.binding,
                    array_index: k as u8,
                    kind,
                });
            }
        }
        BindGroup {
            resources: resources.into_boxed_slice(),
        }
    }

    pub(super) fn new_sampler(&self, desc: &crate::SamplerDescriptor) -> Result<Sampler, crate::DeviceError> {
        // Not zeroed: DkFilter and DkCompareOp have no 0. deko3d's defaults fill every field.
        let mut s = unsafe {
            let mut m = core::mem::MaybeUninit::<DkSampler>::uninit();
            dkSamplerDefaults(m.as_mut_ptr());
            m.assume_init()
        };
        let filter = |f: wgt::FilterMode| match f {
            wgt::FilterMode::Nearest => DkFilter::DkFilter_Nearest,
            wgt::FilterMode::Linear => DkFilter::DkFilter_Linear,
        };
        s.minFilter = filter(desc.min_filter);
        s.magFilter = filter(desc.mag_filter);
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
            s.compareOp = compare_op(c);
        }
        s.maxAnisotropy = desc.anisotropy_clamp.max(1) as f32;
        let border = match desc.border_color {
            Some(wgt::SamplerBorderColor::OpaqueBlack) => [0.0, 0.0, 0.0, 1.0],
            Some(wgt::SamplerBorderColor::OpaqueWhite) => [1.0; 4],
            _ => [0.0; 4],
        };
        for (d, v) in s.borderColor.iter_mut().zip(border) {
            d.value_f = v;
        }
        let descriptor = self.shared.descriptors.lock().new_sampler(&s).ok_or(crate::DeviceError::OutOfMemory)?;
        Ok(Sampler { descriptor })
    }

    /// GLSL for one stage, with the texture/sampler pairs naga combined.
    fn glsl(
        &self,
        stage: &crate::ProgrammableStage<ShaderModule>,
        naga_stage: naga::ShaderStage,
        layout: &PipelineLayout,
        sampling: &mut Sampling,
    ) -> Result<String, crate::PipelineError> {
        let wgt_stage_flags = wgt_stage(naga_stage);
        let (module, info) = naga::back::pipeline_constants::process_overrides(
            &stage.module.naga.module,
            &stage.module.naga.info,
            Some((naga_stage, stage.entry_point)),
            stage.constants,
        )
        .map_err(|e| crate::PipelineError::PipelineConstants(wgt_stage_flags, format!("{e}")))?;
        let (output, reflection, _, _) = benilla_uam::write_glsl(
            &module,
            &info,
            naga_stage,
            stage.entry_point,
            layout.binding_map.clone(),
            stage.zero_initialize_workgroup_memory,
        )
        .map_err(|e| crate::PipelineError::Linkage(wgt_stage_flags, e))?;
        for (_, m) in reflection.texture_mapping {
            let var = &module.global_variables[m.texture];
            let slot = layout.slot(var.binding.as_ref().unwrap());
            let storage = matches!(module.types[var.ty].inner, naga::TypeInner::Image { class: naga::ImageClass::Storage { .. }, .. });
            if storage {
                if !sampling.images.contains(&slot) {
                    sampling.images.push(slot);
                }
                continue;
            }
            let sampler = m.sampler.map(|s| layout.slot(module.global_variables[s].binding.as_ref().unwrap()));
            if !sampling.textures.iter().any(|(t, _)| *t == slot) {
                sampling.textures.push((slot, sampler));
            }
        }
        Ok(output)
    }

    fn compile(&self, glsl: &str, naga_stage: naga::ShaderStage) -> Result<CompiledStage, crate::PipelineError> {
        let stage = match naga_stage {
            naga::ShaderStage::Vertex => benilla_uam::Stage::Vertex,
            naga::ShaderStage::Fragment => benilla_uam::Stage::Fragment,
            _ => benilla_uam::Stage::Compute,
        };
        let dksh = benilla_uam::compile_cached(stage, glsl).map_err(|log| {
            log::error!("uam ({naga_stage:?}): {log}\n{glsl}");
            crate::PipelineError::Linkage(wgt_stage(naga_stage), log)
        })?;
        let shader = self
            .shared
            .code
            .lock()
            .load(&dksh)
            .ok_or_else(|| crate::PipelineError::Linkage(wgt_stage(naga_stage), "code memory full".into()))?;
        Ok(CompiledStage { shader })
    }

    pub(super) fn new_render_pipeline(
        &self,
        desc: &crate::RenderPipelineDescriptor<PipelineLayout, ShaderModule, super::Resource>,
    ) -> Result<RenderPipeline, crate::PipelineError> {
        let crate::VertexProcessor::Standard { vertex_buffers, vertex_stage } = &desc.vertex_processor else {
            return Err(crate::PipelineError::EntryPoint(naga::ShaderStage::Task));
        };
        let mut sampling = Sampling::default();
        let glsl_vs = self.glsl(vertex_stage, naga::ShaderStage::Vertex, desc.layout, &mut sampling)?;
        let first_instance = glsl_vs.contains(glsl::FIRST_INSTANCE_BINDING);
        let mut push_constants = glsl_vs.contains("_push_constant_binding_");
        let vs = self.compile(&glsl_vs, naga::ShaderStage::Vertex)?;
        let fs = match &desc.fragment_stage {
            Some(f) => {
                let g = self.glsl(f, naga::ShaderStage::Fragment, desc.layout, &mut sampling)?;
                push_constants |= g.contains("_push_constant_binding_");
                Some(self.compile(&g, naga::ShaderStage::Fragment)?)
            }
            None => None,
        };

        let p = &desc.primitive;
        use wgt::PrimitiveTopology as T;
        let primitive = match p.topology {
            T::PointList => DkPrimitive::DkPrimitive_Points,
            T::LineList => DkPrimitive::DkPrimitive_Lines,
            T::LineStrip => DkPrimitive::DkPrimitive_LineStrip,
            T::TriangleList => DkPrimitive::DkPrimitive_Triangles,
            T::TriangleStrip => DkPrimitive::DkPrimitive_TriangleStrip,
        };
        let mut rasterizer = unsafe { core::mem::zeroed::<DkRasterizerState>() };
        unsafe { dkRasterizerStateDefaults(&mut rasterizer) };
        rasterizer.set_cullMode(
            match p.cull_mode {
                None => DkFace::DkFace_None,
                Some(wgt::Face::Front) => DkFace::DkFace_Front,
                Some(wgt::Face::Back) => DkFace::DkFace_Back,
            },
        );
        rasterizer.set_frontFace(
            match p.front_face {
                wgt::FrontFace::Ccw => DkFrontFace::DkFrontFace_CCW,
                wgt::FrontFace::Cw => DkFrontFace::DkFrontFace_CW,
            },
        );
        let mode = match p.polygon_mode {
            wgt::PolygonMode::Fill => DkPolygonMode::DkPolygonMode_Fill,
            wgt::PolygonMode::Line => DkPolygonMode::DkPolygonMode_Line,
            wgt::PolygonMode::Point => DkPolygonMode::DkPolygonMode_Point,
        };
        rasterizer.set_polygonModeFront(mode);
        rasterizer.set_polygonModeBack(mode);
        rasterizer.set_depthClampEnable(p.unclipped_depth as u32);

        let mut multisample = unsafe { core::mem::zeroed::<DkMultisampleState>() };
        unsafe { dkMultisampleStateDefaults(&mut multisample) };
        multisample.set_mode(super::conv::ms_mode(desc.multisample.count));
        multisample.set_alphaToCoverageEnable(desc.multisample.alpha_to_coverage_enabled as u32);

        let mut color = unsafe { core::mem::zeroed::<DkColorState>() };
        unsafe { dkColorStateDefaults(&mut color) };
        let mut color_write = unsafe { core::mem::zeroed::<DkColorWriteState>() };
        unsafe { dkColorWriteStateDefaults(&mut color_write) };
        let mut blends = Vec::with_capacity(desc.color_targets.len());
        for (i, t) in desc.color_targets.iter().enumerate() {
            let mut m = unsafe { core::mem::zeroed::<DkBlendState>() };
            unsafe { dkBlendStateDefaults(&mut m) };
            let mascara = t.as_ref().map_or(0, |t| t.write_mask.bits());
            unsafe { dkColorWriteStateSetMask(&mut color_write, i as u32, mascara) };
            if let Some(b) = t.as_ref().and_then(|t| t.blend) {
                unsafe {
                    dkColorStateSetBlendEnable(&mut color, i as u32, true);
                    dkBlendStateSetOps(&mut m, blend_op(b.color.operation), blend_op(b.alpha.operation));
                    dkBlendStateSetFactors(
                        &mut m,
                        blend_factor(b.color.src_factor),
                        blend_factor(b.color.dst_factor),
                        blend_factor(b.alpha.src_factor),
                        blend_factor(b.alpha.dst_factor),
                    );
                }
            }
            blends.push(m);
        }

        let mut depth = unsafe { core::mem::zeroed::<DkDepthStencilState>() };
        unsafe { dkDepthStencilStateDefaults(&mut depth) };
        let (mut depth_bias, mut stencil) = ((0.0, 0.0, 0.0), (0xff, 0xff));
        match &desc.depth_stencil {
            Some(ds) => {
                let compare = ds.depth_compare;
                let writes = ds.depth_write_enabled;
                depth.set_depthTestEnable((compare != wgt::CompareFunction::Always || writes) as u32);
                depth.set_depthWriteEnable(writes as u32);
                depth.set_depthCompareOp(compare_op(compare));
                if ds.stencil.is_enabled() {
                    depth.set_stencilTestEnable(1);
                    let (f, b) = (&ds.stencil.front, &ds.stencil.back);
                    depth.set_stencilFrontFailOp(stencil_op(f.fail_op));
                    depth.set_stencilFrontPassOp(stencil_op(f.pass_op));
                    depth.set_stencilFrontDepthFailOp(stencil_op(f.depth_fail_op));
                    depth.set_stencilFrontCompareOp(compare_op(f.compare));
                    depth.set_stencilBackFailOp(stencil_op(b.fail_op));
                    depth.set_stencilBackPassOp(stencil_op(b.pass_op));
                    depth.set_stencilBackDepthFailOp(stencil_op(b.depth_fail_op));
                    depth.set_stencilBackCompareOp(compare_op(b.compare));
                    stencil = (ds.stencil.read_mask as u8, ds.stencil.write_mask as u8);
                }
                if ds.bias.is_enabled() {
                    rasterizer.set_depthBiasEnableMask(DkPolygonFlag_All);
                    depth_bias = (ds.bias.constant as f32, ds.bias.clamp, ds.bias.slope_scale);
                }
            }
            None => {
                depth.set_depthTestEnable(0);
                depth.set_depthWriteEnable(0);
            }
        }

        let mut attribs = Vec::new();
        let mut buffer_states = Vec::new();
        for (i, vb) in vertex_buffers.iter().enumerate() {
            buffer_states.push(DkVtxBufferState {
                stride: vb.array_stride as u32,
                divisor: (vb.step_mode == wgt::VertexStepMode::Instance) as u32,
            });
            for a in vb.attributes {
                let (size, kind, bgra) = vertex_format(a.format);
                let mut s = unsafe { core::mem::zeroed::<DkVtxAttribState>() };
                s.set_bufferId(i as u32);
                s.set_isFixed(0);
                s.set_offset(a.offset as u32);
                s.set_size(size);
                s.set_type(kind);
                s.set_isBgra(bgra as u32);
                let l = a.shader_location as usize;
                if attribs.len() <= l {
                    // Locations the shader does not read stay fixed (constant zero).
                    let mut fixed = unsafe { core::mem::zeroed::<DkVtxAttribState>() };
                    fixed.set_isFixed(1);
                    fixed.set_size(DkVtxAttribSize::DkVtxAttribSize_1x32);
                    fixed.set_type(DkVtxAttribType::DkVtxAttribType_Float);
                    attribs.resize(l + 1, fixed);
                }
                attribs[l] = s;
            }
        }

        Ok(RenderPipeline {
            vs,
            fs,
            sampling,
            first_instance,
            push_constants,
            primitive,
            strip: p.strip_index_format.is_some() || matches!(p.topology, T::LineStrip | T::TriangleStrip),
            rasterizer,
            multisample,
            // WebGPU's mask is 64 bits of ones by default; Maxwell's method takes 16 (a GPU method error otherwise).
            sample_mask: (desc.multisample.mask & 0xffff) as u32,
            color,
            color_write,
            blends,
            depth,
            depth_bias,
            stencil,
            attribs,
            vertex_buffers: buffer_states,
            label: desc.label.unwrap_or("unlabelled").into(),
        })
    }

    pub(super) fn new_compute_pipeline(
        &self,
        desc: &crate::ComputePipelineDescriptor<PipelineLayout, ShaderModule, super::Resource>,
    ) -> Result<ComputePipeline, crate::PipelineError> {
        let mut sampling = Sampling::default();
        let g = self.glsl(&desc.stage, naga::ShaderStage::Compute, desc.layout, &mut sampling)?;
        Ok(ComputePipeline {
            push_constants: g.contains("_push_constant_binding_"),
            cs: self.compile(&g, naga::ShaderStage::Compute)?,
            sampling,
            label: desc.label.unwrap_or("unlabelled").into(),
        })
    }
}

fn wgt_stage(e: naga::ShaderStage) -> wgt::ShaderStages {
    match e {
        naga::ShaderStage::Vertex => wgt::ShaderStages::VERTEX,
        naga::ShaderStage::Fragment => wgt::ShaderStages::FRAGMENT,
        _ => wgt::ShaderStages::COMPUTE,
    }
}

pub(super) fn compare_op(c: wgt::CompareFunction) -> DkCompareOp {
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

fn stencil_op(o: wgt::StencilOperation) -> DkStencilOp {
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

fn blend_op(o: wgt::BlendOperation) -> DkBlendOp {
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

fn blend_factor(f: wgt::BlendFactor) -> DkBlendFactor {
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

fn vertex_format(f: wgt::VertexFormat) -> (DkVtxAttribSize, DkVtxAttribType, bool) {
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
