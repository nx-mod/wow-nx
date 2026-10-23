use alloc::{boxed::Box, vec::Vec};
use core::{ffi::c_void, ops::Range};

use super::ffi::*;
use super::pipeline::{
    BindGroup, ComputePipeline, Muestreo, PipelineLayout, RenderPipeline, Tipo, DESPLAZAMIENTO_CONSTANTES, SLOT_CONSTANTES,
    SLOT_PRIMERA_INSTANCIA, TAMANO_CONSTANTES,
};
use super::{Api, Buffer, DeviceResult, Resource, Texture, TextureView};

/// The device's GPU addresses an encoder needs: zeros for clears, the descriptor sets, the driver's uniform buffer.
#[derive(Clone, Copy, Debug)]
pub(super) struct Fijos {
    pub(super) ceros: (DkGpuAddr, u32),
    pub(super) imagenes: (DkGpuAddr, u32),
    pub(super) muestreadores: (DkGpuAddr, u32),
    pub(super) ubo: (DkGpuAddr, u32),
}

/// What draws and dispatches read from earlier calls.
#[derive(Debug, Default)]
struct Estado {
    texturas: [Option<u32>; DK_NUM_TEXTURE_BINDINGS as usize],
    muestreadores: [Option<u32>; DK_NUM_TEXTURE_BINDINGS as usize],
    imagenes: [Option<u32>; DK_NUM_IMAGE_BINDINGS as usize],
    texturas_sucias: bool,
    muestreo: Muestreo,
    primitiva: Option<DkPrimitive>,
    tira: bool,
    primera_instancia: bool,
    /// The first instance last written to the driver's uniform buffer.
    primera_escrita: Option<u32>,
    plantilla: (u8, u8),
    referencia: u8,
    indice: Option<DkIdxFormat>,
    computo: bool,
}

/// Command memory of one encoder: deko3d asks for more through `anadir_memoria`.
#[derive(Debug)]
struct Memoria {
    device: DkDevice,
    bloques: Vec<(DkMemBlock, u32)>,
}

unsafe extern "C" fn anadir_memoria(user: *mut c_void, cmdbuf: DkCmdBuf, minimo: usize) {
    let memoria = unsafe { &mut *user.cast::<Memoria>() };
    let tamano = super::alinear(minimo as u64, DK_MEMBLOCK_ALIGNMENT as u64).max(super::MEMORIA_COMANDOS as u64) as u32;
    // A failure reaches the debug callback; deko3d then drops the commands that did not fit.
    let Some(b) = (unsafe { super::bloque(memoria.device, tamano, DkMemBlockFlags_CpuUncached | DkMemBlockFlags_GpuCached) })
    else {
        return;
    };
    memoria.bloques.push((b, tamano));
    unsafe { dkCmdBufAddMemory(cmdbuf, b, 0, tamano) };
}

/// One deko3d command buffer. Its lists stay valid until `reset_all`, which recycles the memory.
#[derive(Debug)]
pub struct CommandEncoder {
    cmdbuf: DkCmdBuf,
    memoria: Box<Memoria>,
    fijos: Fijos,
    vacio: bool,
    estado: Estado,
}

unsafe impl Send for CommandEncoder {}
unsafe impl Sync for CommandEncoder {}

#[derive(Debug)]
pub struct CommandBuffer {
    pub(super) lista: DkCmdList,
}

impl CommandEncoder {
    pub(super) unsafe fn new(device: DkDevice, fijos: Fijos) -> DeviceResult<Self> {
        let mut memoria = Box::new(Memoria {
            device,
            bloques: Vec::new(),
        });
        let mut maker = unsafe { core::mem::zeroed::<DkCmdBufMaker>() };
        unsafe { dkCmdBufMakerDefaults(&mut maker, device) };
        maker.userData = core::ptr::from_mut(&mut *memoria).cast();
        maker.cbAddMem = Some(anadir_memoria);
        let cmdbuf = unsafe { dkCmdBufCreate(&maker) };
        if cmdbuf.is_null() {
            return Err(crate::DeviceError::OutOfMemory);
        }
        Ok(Self {
            cmdbuf,
            memoria,
            fijos,
            vacio: true,
            estado: Estado::default(),
        })
    }

    /// The command buffer, after the descriptor sets on a list's first command.
    fn cmd(&mut self) -> DkCmdBuf {
        let c = self.cmdbuf;
        if self.vacio {
            self.vacio = false;
            let f = self.fijos;
            unsafe {
                dkCmdBufBindImageDescriptorSet(c, f.imagenes.0, f.imagenes.1);
                dkCmdBufBindSamplerDescriptorSet(c, f.muestreadores.0, f.muestreadores.1);
                // Descriptors written since the last list may reuse freed indices.
                dkCmdBufBarrier(c, DkBarrier::DkBarrier_None, DkInvalidateFlags_Descriptors);
            }
        }
        c
    }

    fn etapas(&self) -> &'static [DkStage] {
        if self.estado.computo {
            &[DkStage::DkStage_Compute]
        } else {
            &[DkStage::DkStage_Vertex, DkStage::DkStage_Fragment]
        }
    }

    /// Binds the textures and storage images the current pipeline reads, once per change.
    fn texturas(&mut self) {
        if !self.estado.texturas_sucias {
            return;
        }
        self.estado.texturas_sucias = false;
        let c = self.cmd();
        let e = &self.estado;
        for &etapa in self.etapas() {
            for &(t, m) in &e.muestreo.texturas {
                let imagen = e.texturas.get(t as usize).copied().flatten().unwrap_or(0);
                let muestreador = m.and_then(|m| e.muestreadores.get(m as usize).copied().flatten()).unwrap_or(0);
                let h = unsafe { dkMakeTextureHandle(imagen, muestreador) };
                unsafe { dkCmdBufBindTextures(c, etapa, t as u32, &h, 1) };
            }
            for &i in &e.muestreo.imagenes {
                let h = unsafe { dkMakeImageHandle(e.imagenes.get(i as usize).copied().flatten().unwrap_or(0)) };
                unsafe { dkCmdBufBindImages(c, etapa, i as u32, &h, 1) };
            }
        }
    }

    fn primera_instancia(&mut self, primera: u32) {
        if !self.estado.primera_instancia || self.estado.primera_escrita == Some(primera) {
            return;
        }
        self.estado.primera_escrita = Some(primera);
        let (ubo, tamano) = self.fijos.ubo;
        let c = self.cmd();
        unsafe { dkCmdBufPushConstants(c, ubo, tamano, 0, 4, core::ptr::from_ref(&primera).cast()) };
    }

    /// Binds the push constant block to the stages in use.
    fn enlazar_constantes(&mut self) {
        let c = self.cmd();
        let gpu = self.fijos.ubo.0 + DESPLAZAMIENTO_CONSTANTES;
        for &etapa in self.etapas() {
            unsafe { dkCmdBufBindUniformBuffer(c, etapa, SLOT_CONSTANTES, gpu, TAMANO_CONSTANTES) };
        }
    }

    /// Waits for earlier work and drops cached texels and shader data, so later reads see earlier writes.
    // TODO(wow-nx): pick the barrier from the usages (fragments only, no invalidation) once draws exist.
    fn barrera(&mut self) {
        let c = self.cmd();
        unsafe { dkCmdBufBarrier(c, DkBarrier::DkBarrier_Full, DkInvalidateFlags_Image | DkInvalidateFlags_Shader) };
    }
}

impl Drop for CommandEncoder {
    fn drop(&mut self) {
        unsafe {
            dkCmdBufDestroy(self.cmdbuf);
            for (b, _) in self.memoria.bloques.drain(..) {
                dkMemBlockDestroy(b);
            }
        }
    }
}

/// A view of one mip level and layer, for copies.
fn subrecurso(textura: &Texture, base: &crate::TextureCopyBase, tres_d: bool) -> DkImageView {
    let mut vista = unsafe { core::mem::zeroed::<DkImageView>() };
    unsafe { dkImageViewDefaults(&mut vista, &*textura.imagen) };
    vista.mipLevelOffset = base.mip_level as u8;
    vista.mipLevelCount = 1;
    if !tres_d {
        vista.layerOffset = base.array_layer as u16;
        vista.layerCount = 1;
    }
    if base.aspect == crate::FormatAspects::STENCIL {
        vista.dsSource = DkDsSource::DkDsSource_Stencil;
    }
    vista
}

/// The buffer side of a buffer/texture copy: bytes per row and per image (deko3d's strides are in bytes).
fn lado_buffer(buffer: &Buffer, textura: &Texture, copia: &crate::BufferTextureCopy) -> (DkCopyBuf, u64) {
    let formato = textura.formato;
    let (bw, bh) = formato.block_dimensions();
    let bloque = formato.block_copy_size(Some(copia.texture_base.aspect.map())).unwrap_or(4) as u64;
    let fila = copia
        .buffer_layout
        .bytes_per_row
        .map_or(copia.size.width.div_ceil(bw) as u64 * bloque, u64::from);
    let filas = copia
        .buffer_layout
        .rows_per_image
        .map_or(copia.size.height.div_ceil(bh), |r| r.div_ceil(bh)) as u64;
    let imagen = fila * filas;
    (
        DkCopyBuf {
            addr: buffer.gpu + copia.buffer_layout.offset,
            rowLength: fila as u32,
            imageHeight: imagen as u32,
        },
        imagen,
    )
}

/// Copies between `buffer` and `textura`, one deko3d copy per array layer (3D textures in one).
unsafe fn copiar_buffer_textura<T>(cmd: DkCmdBuf, buffer: &Buffer, textura: &Texture, regiones: T, hacia_textura: bool)
where
    T: Iterator<Item = crate::BufferTextureCopy>,
{
    let tres_d = textura.dimension_3d();
    for copia in regiones {
        let (mut lado, por_imagen) = lado_buffer(buffer, textura, &copia);
        let o = copia.texture_base.origin;
        let capas = if tres_d { 1 } else { copia.size.depth };
        for capa in 0..capas {
            let mut base = copia.texture_base.clone();
            base.array_layer += capa;
            let vista = subrecurso(textura, &base, tres_d);
            let rect = DkImageRect {
                x: o.x,
                y: o.y,
                z: if tres_d { o.z } else { 0 },
                width: copia.size.width,
                height: copia.size.height,
                depth: if tres_d { copia.size.depth } else { 1 },
            };
            unsafe {
                if hacia_textura {
                    dkCmdBufCopyBufferToImage(cmd, &lado, &vista, &rect, 0);
                } else {
                    dkCmdBufCopyImageToBuffer(cmd, &vista, &rect, &lado, 0);
                }
            }
            lado.addr += por_imagen;
        }
    }
}

impl crate::CommandEncoder for CommandEncoder {
    type A = Api;

    unsafe fn begin_encoding(&mut self, label: crate::Label) -> DeviceResult<()> {
        self.vacio = true;
        self.estado = Estado::default();
        Ok(())
    }
    unsafe fn discard_encoding(&mut self) {
        // The commands stay in memory until `reset_all`; the list is simply never submitted.
        unsafe { dkCmdBufFinishList(self.cmdbuf) };
    }
    unsafe fn end_encoding(&mut self) -> DeviceResult<CommandBuffer> {
        let lista = if self.vacio { 0 } else { unsafe { dkCmdBufFinishList(self.cmdbuf) } };
        Ok(CommandBuffer { lista })
    }
    unsafe fn reset_all<I>(&mut self, command_buffers: I) {
        // Every list from this encoder has executed: start over in the first block, free the rest.
        unsafe { dkCmdBufClear(self.cmdbuf) };
        let mut bloques = core::mem::take(&mut self.memoria.bloques).into_iter();
        if let Some((b, tamano)) = bloques.next() {
            for (resto, _) in bloques {
                unsafe { dkMemBlockDestroy(resto) };
            }
            unsafe { dkCmdBufAddMemory(self.cmdbuf, b, 0, tamano) };
            self.memoria.bloques.push((b, tamano));
        }
    }

    unsafe fn transition_buffers<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = crate::BufferBarrier<'a, Buffer>>,
    {
        if barriers.into_iter().next().is_some() {
            self.barrera();
        }
    }

    unsafe fn transition_textures<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = crate::TextureBarrier<'a, Texture>>,
    {
        if barriers.into_iter().next().is_some() {
            self.barrera();
        }
    }

    unsafe fn clear_buffer(&mut self, buffer: &Buffer, range: crate::MemoryRange) {
        let (ceros, tamano) = self.fijos.ceros;
        let c = self.cmd();
        let mut inicio = range.start;
        while inicio < range.end {
            let n = (range.end - inicio).min(tamano as u64);
            unsafe { dkCmdBufCopyBuffer(c, ceros, buffer.gpu + inicio, n as u32) };
            inicio += n;
        }
    }

    unsafe fn copy_buffer_to_buffer<T>(&mut self, src: &Buffer, dst: &Buffer, regions: T)
    where
        T: Iterator<Item = crate::BufferCopy>,
    {
        let c = self.cmd();
        for r in regions {
            unsafe { dkCmdBufCopyBuffer(c, src.gpu + r.src_offset, dst.gpu + r.dst_offset, r.size.get() as u32) };
        }
    }

    #[cfg(webgl)]
    unsafe fn copy_external_image_to_texture<T>(
        &mut self,
        src: &wgt::CopyExternalImageSourceInfo,
        dst: &Texture,
        dst_premultiplication: bool,
        regions: T,
    ) where
        T: Iterator<Item = crate::TextureCopy>,
    {
    }

    unsafe fn copy_texture_to_texture<T>(&mut self, src: &Texture, src_usage: wgt::TextureUses, dst: &Texture, regions: T)
    where
        T: Iterator<Item = crate::TextureCopy>,
    {
        let c = self.cmd();
        let tres_d = src.dimension_3d();
        for r in regions {
            let capas = if tres_d { 1 } else { r.size.depth };
            for capa in 0..capas {
                let (mut a, mut b) = (r.src_base.clone(), r.dst_base.clone());
                a.array_layer += capa;
                b.array_layer += capa;
                let (va, vb) = (subrecurso(src, &a, tres_d), subrecurso(dst, &b, dst.dimension_3d()));
                let rect = |o: wgt::Origin3d| DkImageRect {
                    x: o.x,
                    y: o.y,
                    z: if tres_d { o.z } else { 0 },
                    width: r.size.width,
                    height: r.size.height,
                    depth: if tres_d { r.size.depth } else { 1 },
                };
                let (ra, rb) = (rect(a.origin), rect(b.origin));
                unsafe { dkCmdBufCopyImage(c, &va, &ra, &vb, &rb, 0) };
            }
        }
    }

    unsafe fn copy_buffer_to_texture<T>(&mut self, src: &Buffer, dst: &Texture, regions: T)
    where
        T: Iterator<Item = crate::BufferTextureCopy>,
    {
        let c = self.cmd();
        unsafe { copiar_buffer_textura(c, src, dst, regions, true) };
    }

    unsafe fn copy_texture_to_buffer<T>(&mut self, src: &Texture, src_usage: wgt::TextureUses, dst: &Buffer, regions: T)
    where
        T: Iterator<Item = crate::BufferTextureCopy>,
    {
        let c = self.cmd();
        unsafe { copiar_buffer_textura(c, dst, src, regions, false) };
    }

    unsafe fn begin_query(&mut self, set: &Resource, index: u32) {}
    unsafe fn end_query(&mut self, set: &Resource, index: u32) {}
    unsafe fn write_timestamp(&mut self, set: &Resource, index: u32) {}
    unsafe fn read_acceleration_structure_compact_size(&mut self, acceleration_structure: &Resource, buf: &Buffer) {}
    unsafe fn reset_queries(&mut self, set: &Resource, range: Range<u32>) {}
    unsafe fn copy_query_results(
        &mut self,
        set: &Resource,
        range: Range<u32>,
        buffer: &Buffer,
        offset: wgt::BufferAddress,
        stride: wgt::BufferSize,
    ) {
    }

    // render

    unsafe fn begin_render_pass(&mut self, desc: &crate::RenderPassDescriptor<Resource, TextureView>) -> DeviceResult<()> {
        let c = self.cmd();
        let vistas: Vec<Option<DkImageView>> = desc
            .color_attachments
            .iter()
            .map(|a| {
                a.as_ref().map(|a| {
                    let mut v = a.target.view.vista;
                    if let Some(rebanada) = a.depth_slice {
                        v.layerOffset = rebanada as u16;
                        v.layerCount = 1;
                    }
                    v
                })
            })
            .collect();
        // deko3d needs a dense array: an unused slot aliases a used target (pipelines give it no writes).
        let relleno = vistas.iter().flatten().next().map(core::ptr::from_ref);
        let colores: Vec<*const DkImageView> = vistas
            .iter()
            .map(|v| v.as_ref().map_or(relleno.unwrap_or(core::ptr::null()), core::ptr::from_ref))
            .collect();
        let profundidad = desc.depth_stencil_attachment.as_ref().map(|d| {
            let mut v = d.target.view.vista;
            v.dsSource = DkDsSource::DkDsSource_Depth;
            v
        });
        unsafe {
            dkCmdBufBindRenderTargets(
                c,
                colores.as_ptr(),
                colores.len() as u32,
                profundidad.as_ref().map_or(core::ptr::null(), core::ptr::from_ref),
            );
            let (w, h) = (desc.extent.width, desc.extent.height);
            dkCmdBufSetViewports(c, 0, &DkViewport { x: 0.0, y: 0.0, width: w as f32, height: h as f32, near: 0.0, far: 1.0 }, 1);
            dkCmdBufSetScissors(c, 0, &DkScissor { x: 0, y: 0, width: w, height: h }, 1);
            // Clears honour the colour write state: write everything first.
            let mut escribir = core::mem::zeroed::<DkColorWriteState>();
            dkColorWriteStateDefaults(&mut escribir);
            dkCmdBufBindColorWriteState(c, &escribir);
        }
        for (i, a) in desc.color_attachments.iter().enumerate() {
            let Some(a) = a else { continue };
            if a.ops.contains(crate::AttachmentOps::LOAD) {
                continue;
            }
            let k = a.clear_value;
            let i = i as u32;
            unsafe {
                match a.target.view.formato.sample_type(None, None) {
                    Some(wgt::TextureSampleType::Uint) => {
                        dkCmdBufClearColorUint(c, i, DkColorMask_RGBA, k.r as u32, k.g as u32, k.b as u32, k.a as u32)
                    }
                    Some(wgt::TextureSampleType::Sint) => {
                        dkCmdBufClearColorSint(c, i, DkColorMask_RGBA, k.r as i32, k.g as i32, k.b as i32, k.a as i32)
                    }
                    _ => dkCmdBufClearColorFloat(c, i, DkColorMask_RGBA, k.r as f32, k.g as f32, k.b as f32, k.a as f32),
                }
            }
        }
        if let Some(d) = &desc.depth_stencil_attachment {
            let borrar_z = !d.depth_ops.contains(crate::AttachmentOps::LOAD) && d.target.view.formato.has_depth_aspect();
            let borrar_s = !d.stencil_ops.contains(crate::AttachmentOps::LOAD) && d.target.view.formato.has_stencil_aspect();
            if borrar_z || borrar_s {
                unsafe {
                    let mut ds = core::mem::zeroed::<DkDepthStencilState>();
                    dkDepthStencilStateDefaults(&mut ds);
                    ds.set_depthWriteEnable(borrar_z as u32);
                    dkCmdBufBindDepthStencilState(c, &ds);
                    let (z, s) = d.clear_value;
                    dkCmdBufClearDepthStencil(c, borrar_z, z, if borrar_s { 0xff } else { 0 }, s as u8);
                }
            }
        }
        Ok(())
    }
    unsafe fn end_render_pass(&mut self) {
        // What the pass rendered may be sampled next.
        let c = self.cmd();
        unsafe { dkCmdBufBarrier(c, DkBarrier::DkBarrier_Fragments, DkInvalidateFlags_Image) };
    }

    unsafe fn set_bind_group(
        &mut self,
        layout: &PipelineLayout,
        index: u32,
        group: &BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
        let c = self.cmd();
        let grupo = &layout.grupos[index as usize];
        let mut dinamicos = dynamic_offsets.iter();
        for r in group.recursos.iter() {
            let ranura = (grupo.ranuras[r.binding as usize] + r.indice) as u32;
            let desplazamiento = |dinamico: bool, d: &mut core::slice::Iter<'_, u32>| {
                if dinamico { d.next().copied().unwrap_or(0) as u64 } else { 0 }
            };
            match r.tipo {
                Tipo::Uniforme { gpu, tamano, dinamico } => {
                    let gpu = gpu + desplazamiento(dinamico, &mut dinamicos);
                    for &etapa in self.etapas() {
                        unsafe { dkCmdBufBindUniformBuffer(c, etapa, ranura, gpu, tamano) };
                    }
                }
                Tipo::Almacen { gpu, tamano, dinamico } => {
                    let gpu = gpu + desplazamiento(dinamico, &mut dinamicos);
                    for &etapa in self.etapas() {
                        unsafe { dkCmdBufBindStorageBuffer(c, etapa, ranura, gpu, tamano) };
                    }
                }
                Tipo::Textura { descriptor } => {
                    if let Some(t) = self.estado.texturas.get_mut(ranura as usize) {
                        *t = Some(descriptor);
                    }
                }
                Tipo::Imagen { descriptor } => {
                    if let Some(t) = self.estado.imagenes.get_mut(ranura as usize) {
                        *t = Some(descriptor);
                    }
                }
                Tipo::Muestreador { descriptor } => {
                    if let Some(t) = self.estado.muestreadores.get_mut(ranura as usize) {
                        *t = Some(descriptor);
                    }
                }
            }
        }
        self.estado.texturas_sucias = true;
    }
    unsafe fn set_push_constants(&mut self, layout: &PipelineLayout, stages: wgt::ShaderStages, offset_bytes: u32, data: &[u32]) {
        let c = self.cmd();
        let gpu = self.fijos.ubo.0 + DESPLAZAMIENTO_CONSTANTES;
        let tamano = (data.len() * 4) as u32;
        if offset_bytes + tamano <= TAMANO_CONSTANTES {
            unsafe { dkCmdBufPushConstants(c, gpu, TAMANO_CONSTANTES, offset_bytes, tamano, data.as_ptr().cast()) };
        }
    }

    unsafe fn insert_debug_marker(&mut self, label: &str) {}
    unsafe fn begin_debug_marker(&mut self, group_label: &str) {}
    unsafe fn end_debug_marker(&mut self) {}

    unsafe fn set_render_pipeline(&mut self, p: &RenderPipeline) {
        let c = self.cmd();
        let mut shaders: arrayvec::ArrayVec<*const DkShader, 2> = arrayvec::ArrayVec::new();
        shaders.push(&*p.vs.shader);
        if let Some(fs) = &p.fs {
            shaders.push(&*fs.shader);
        }
        unsafe {
            dkCmdBufBindShaders(c, DkStageFlag_GraphicsMask, shaders.as_ptr(), shaders.len() as u32);
            dkCmdBufBindRasterizerState(c, &p.rasterizador);
            dkCmdBufBindMultisampleState(c, &p.multimuestra);
            dkCmdBufSetSampleMask(c, p.mascara_muestras);
            dkCmdBufBindColorState(c, &p.color);
            dkCmdBufBindColorWriteState(c, &p.escritura);
            if !p.mezclas.is_empty() {
                dkCmdBufBindBlendStates(c, 0, p.mezclas.as_ptr(), p.mezclas.len() as u32);
            }
            dkCmdBufBindDepthStencilState(c, &p.profundidad);
            dkCmdBufSetDepthBias(c, p.sesgo.0, p.sesgo.1, p.sesgo.2);
            dkCmdBufSetStencil(c, DkFace::DkFace_FrontAndBack, p.plantilla.1, self.estado.referencia, p.plantilla.0);
            dkCmdBufBindVtxAttribState(c, p.atributos.as_ptr(), p.atributos.len() as u32);
            dkCmdBufBindVtxBufferState(c, p.buferes.as_ptr(), p.buferes.len() as u32);
            if p.primera_instancia {
                dkCmdBufBindUniformBuffer(c, DkStage::DkStage_Vertex, SLOT_PRIMERA_INSTANCIA, self.fijos.ubo.0, 256);
            }
        }
        if p.constantes {
            self.enlazar_constantes();
        }
        let e = &mut self.estado;
        e.muestreo = p.muestreo.clone();
        e.primitiva = Some(p.primitiva);
        e.tira = p.tira;
        e.primera_instancia = p.primera_instancia;
        e.primera_escrita = None;
        e.plantilla = p.plantilla;
        e.texturas_sucias = true;
    }

    unsafe fn set_index_buffer<'a>(&mut self, binding: crate::BufferBinding<'a, Buffer>, format: wgt::IndexFormat) {
        let c = self.cmd();
        let f = match format {
            wgt::IndexFormat::Uint16 => DkIdxFormat::DkIdxFormat_Uint16,
            wgt::IndexFormat::Uint32 => DkIdxFormat::DkIdxFormat_Uint32,
        };
        unsafe { dkCmdBufBindIdxBuffer(c, f, binding.buffer.gpu + binding.offset) };
        self.estado.indice = Some(f);
    }
    unsafe fn set_vertex_buffer<'a>(&mut self, index: u32, binding: crate::BufferBinding<'a, Buffer>) {
        let c = self.cmd();
        let tamano = binding.size.map_or(binding.buffer.size - binding.offset, |s| s.get());
        unsafe { dkCmdBufBindVtxBuffer(c, index, binding.buffer.gpu + binding.offset, tamano as u32) };
    }
    unsafe fn set_viewport(&mut self, rect: &crate::Rect<f32>, depth_range: Range<f32>) {
        let c = self.cmd();
        let v = DkViewport {
            x: rect.x,
            y: rect.y,
            width: rect.w,
            height: rect.h,
            near: depth_range.start,
            far: depth_range.end,
        };
        unsafe { dkCmdBufSetViewports(c, 0, &v, 1) };
    }
    unsafe fn set_scissor_rect(&mut self, rect: &crate::Rect<u32>) {
        let c = self.cmd();
        let s = DkScissor {
            x: rect.x,
            y: rect.y,
            width: rect.w,
            height: rect.h,
        };
        unsafe { dkCmdBufSetScissors(c, 0, &s, 1) };
    }
    unsafe fn set_stencil_reference(&mut self, value: u32) {
        let c = self.cmd();
        self.estado.referencia = value as u8;
        let (lectura, escritura) = self.estado.plantilla;
        unsafe { dkCmdBufSetStencil(c, DkFace::DkFace_FrontAndBack, escritura, value as u8, lectura) };
    }
    unsafe fn set_blend_constants(&mut self, color: &[f32; 4]) {
        let c = self.cmd();
        unsafe { dkCmdBufSetBlendConst(c, color[0], color[1], color[2], color[3]) };
    }

    unsafe fn draw(&mut self, first_vertex: u32, vertex_count: u32, first_instance: u32, instance_count: u32) {
        let Some(prim) = self.estado.primitiva else { return };
        self.texturas();
        self.primera_instancia(first_instance);
        let c = self.cmd();
        unsafe { dkCmdBufDraw(c, prim, vertex_count, instance_count, first_vertex, first_instance) };
    }
    unsafe fn draw_indexed(&mut self, first_index: u32, index_count: u32, base_vertex: i32, first_instance: u32, instance_count: u32) {
        let Some(prim) = self.estado.primitiva else { return };
        self.texturas();
        self.primera_instancia(first_instance);
        let c = self.cmd();
        let reinicio = if self.estado.indice == Some(DkIdxFormat::DkIdxFormat_Uint16) { 0xffff } else { 0xffff_ffff };
        unsafe {
            dkCmdBufSetPrimitiveRestart(c, self.estado.tira, reinicio);
            dkCmdBufDrawIndexed(c, prim, index_count, instance_count, first_index, base_vertex, first_instance);
        }
    }
    unsafe fn draw_mesh_tasks(&mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {}
    unsafe fn draw_indirect(&mut self, buffer: &Buffer, offset: wgt::BufferAddress, draw_count: u32) {
        let Some(prim) = self.estado.primitiva else { return };
        self.texturas();
        self.primera_instancia(0);
        let c = self.cmd();
        for i in 0..draw_count as u64 {
            unsafe { dkCmdBufDrawIndirect(c, prim, buffer.gpu + offset + i * 16) };
        }
    }
    unsafe fn draw_indexed_indirect(&mut self, buffer: &Buffer, offset: wgt::BufferAddress, draw_count: u32) {
        let Some(prim) = self.estado.primitiva else { return };
        self.texturas();
        self.primera_instancia(0);
        let c = self.cmd();
        let reinicio = if self.estado.indice == Some(DkIdxFormat::DkIdxFormat_Uint16) { 0xffff } else { 0xffff_ffff };
        unsafe { dkCmdBufSetPrimitiveRestart(c, self.estado.tira, reinicio) };
        for i in 0..draw_count as u64 {
            unsafe { dkCmdBufDrawIndexedIndirect(c, prim, buffer.gpu + offset + i * 20) };
        }
    }
    unsafe fn draw_mesh_tasks_indirect(&mut self, buffer: &Buffer, offset: wgt::BufferAddress, draw_count: u32) {}
    unsafe fn draw_indirect_count(
        &mut self,
        buffer: &Buffer,
        offset: wgt::BufferAddress,
        count_buffer: &Buffer,
        count_offset: wgt::BufferAddress,
        max_count: u32,
    ) {
    }
    unsafe fn draw_indexed_indirect_count(
        &mut self,
        buffer: &Buffer,
        offset: wgt::BufferAddress,
        count_buffer: &Buffer,
        count_offset: wgt::BufferAddress,
        max_count: u32,
    ) {
    }
    unsafe fn draw_mesh_tasks_indirect_count(
        &mut self,
        buffer: &Buffer,
        offset: wgt::BufferAddress,
        count_buffer: &Buffer,
        count_offset: wgt::BufferAddress,
        max_count: u32,
    ) {
    }

    // compute

    unsafe fn begin_compute_pass(&mut self, desc: &crate::ComputePassDescriptor<Resource>) {
        self.estado.computo = true;
        self.estado.texturas_sucias = true;
    }
    unsafe fn end_compute_pass(&mut self) {
        self.estado.computo = false;
        self.estado.texturas_sucias = true;
        let c = self.cmd();
        unsafe { dkCmdBufBarrier(c, DkBarrier::DkBarrier_Primitives, DkInvalidateFlags_Image | DkInvalidateFlags_Shader) };
    }

    unsafe fn set_compute_pipeline(&mut self, p: &ComputePipeline) {
        let c = self.cmd();
        let cs: *const DkShader = &*p.cs.shader;
        unsafe { dkCmdBufBindShaders(c, DkStageFlag_Compute, &cs, 1) };
        if p.constantes {
            self.enlazar_constantes();
        }
        self.estado.muestreo = p.muestreo.clone();
        self.estado.texturas_sucias = true;
    }

    unsafe fn dispatch(&mut self, count: [u32; 3]) {
        self.texturas();
        let c = self.cmd();
        unsafe { dkCmdBufDispatchCompute(c, count[0], count[1], count[2]) };
    }
    unsafe fn dispatch_indirect(&mut self, buffer: &Buffer, offset: wgt::BufferAddress) {
        self.texturas();
        let c = self.cmd();
        unsafe { dkCmdBufDispatchComputeIndirect(c, buffer.gpu + offset) };
    }

    unsafe fn build_acceleration_structures<'a, T>(&mut self, _descriptor_count: u32, descriptors: T)
    where
        Api: 'a,
        T: IntoIterator<Item = crate::BuildAccelerationStructureDescriptor<'a, Buffer, Resource>>,
    {
    }

    unsafe fn place_acceleration_structure_barrier(&mut self, _barriers: crate::AccelerationStructureBarrier) {}

    unsafe fn copy_acceleration_structure_to_acceleration_structure(
        &mut self,
        src: &Resource,
        dst: &Resource,
        copy: wgt::AccelerationStructureCopy,
    ) {
    }
}
