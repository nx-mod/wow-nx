use alloc::{boxed::Box, vec::Vec};
use core::{ffi::c_void, ops::Range};

use super::ffi::*;
use super::pipeline::{
    BindGroup, ComputePipeline, Sampling, PipelineLayout, RenderPipeline, Kind, PUSH_CONSTANT_OFFSET, PUSH_CONSTANT_SLOT,
    FIRST_INSTANCE_SLOT, PUSH_CONSTANT_SIZE,
};
use super::{Api, Buffer, DeviceResult, Resource, Texture, TextureView};

/// The device's GPU addresses an encoder needs: zeros for clears, the descriptor sets, the driver's uniform buffer.
#[derive(Clone, Copy, Debug)]
pub(super) struct DeviceAddrs {
    pub(super) zeros: (DkGpuAddr, u32),
    pub(super) images: (DkGpuAddr, u32),
    pub(super) samplers: (DkGpuAddr, u32),
    pub(super) ubo: (DkGpuAddr, u32),
}

/// What draws and dispatches read from earlier calls.
#[derive(Debug, Default)]
struct DrawState {
    textures: [Option<u32>; DK_NUM_TEXTURE_BINDINGS as usize],
    samplers: [Option<u32>; DK_NUM_TEXTURE_BINDINGS as usize],
    images: [Option<u32>; DK_NUM_IMAGE_BINDINGS as usize],
    textures_dirty: bool,
    sampling: Sampling,
    primitive: Option<DkPrimitive>,
    strip: bool,
    first_instance: bool,
    /// The first instance last written to the driver's uniform buffer.
    first_instance_written: Option<u32>,
    stencil: (u8, u8),
    stencil_ref: u8,
    index_format: Option<DkIdxFormat>,
    compute: bool,
    /// Multisampled targets and where the pass resolves them, at its end.
    resolves: Vec<(DkImageView, DkImageView)>,
    /// The texture and image handles this encoding last bound, by stage (`DkStage`) and slot: a slot is bound
    /// again only when its handle changes. Unknown at the start of an encoding, whose lists may follow anyone's.
    bound_textures: [[Option<DkResHandle>; DK_NUM_TEXTURE_BINDINGS as usize]; STAGES],
    bound_images: [[Option<DkResHandle>; DK_NUM_IMAGE_BINDINGS as usize]; STAGES],
}

/// deko3d's stages, vertex to compute.
const STAGES: usize = 6;

/// Command memory of one encoder: deko3d asks for more through `add_memory`.
#[derive(Debug)]
struct CommandMemory {
    device: DkDevice,
    blocks: Vec<(DkMemBlock, u32)>,
}

unsafe extern "C" fn add_memory(user: *mut c_void, cmdbuf: DkCmdBuf, min_size: usize) {
    let memory = unsafe { &mut *user.cast::<CommandMemory>() };
    let size = super::align_up(min_size as u64, DK_MEMBLOCK_ALIGNMENT as u64).max(super::COMMAND_MEMORY_SIZE as u64) as u32;
    // A failure reaches the debug callback; deko3d then drops the commands that did not fit.
    let Some(b) = (unsafe { super::mem_block(memory.device, size, DkMemBlockFlags_CpuUncached | DkMemBlockFlags_GpuCached) })
    else {
        return;
    };
    memory.blocks.push((b, size));
    unsafe { dkCmdBufAddMemory(cmdbuf, b, 0, size) };
}

/// One deko3d command buffer. Its lists stay valid until `reset_all`, which recycles the memory.
#[derive(Debug)]
pub struct CommandEncoder {
    cmdbuf: DkCmdBuf,
    /// Dropped after the command memory is freed (`Drop`), so the device outlives it.
    shared: alloc::sync::Arc<super::Shared>,
    _counted: super::Counted,
    memory: Box<CommandMemory>,
    addrs: DeviceAddrs,
    empty: bool,
    /// Lists finished so far in this encoding (with their GPFIFO entries), and commands and entries in the current one.
    lists: Vec<(DkCmdList, u32)>,
    commands: u32,
    entries: u32,
    /// A barrier asked for and not yet written: consecutive transitions make one, before the next command.
    barrier_pending: bool,
    state: DrawState,
    /// The passes timed in this encoding, and the one open (its label and start slot).
    passes: Vec<super::PassTime>,
    open_pass: Option<(alloc::string::String, u32)>,
    /// The pipeline drawing since its bind, timed like a pass (its label prefixed `PIPELINE_MARK`).
    open_pipeline: Option<(alloc::string::String, u32)>,
}

unsafe impl Send for CommandEncoder {}
unsafe impl Sync for CommandEncoder {}

#[derive(Debug)]
pub struct CommandBuffer {
    /// Its passes' GPU timestamps, read once the GPU has run it (`Queue::submit`, `Queue::present`).
    pub(super) passes: Vec<super::PassTime>,
    /// Short lists (`LIST_COMMANDS`), with the GPFIFO entries each takes at most, by which `Queue::submit` keeps the
    /// GPU's ring from filling.
    pub(super) lists: Vec<(DkCmdList, u32)>,
}

/// GPFIFO entries any list takes besides the ones counted (its start, its end, a change of command memory).
const LIST_ENTRIES: u32 = 4;

/// Commands per deko3d list. libnx's GPU channel holds 2048 GPFIFO entries between flushes, and one long list (a
/// frame's texture uploads) overflowed it ("gpfifo entry append failed"): lists are kept short, so `Queue::submit` can
/// let the GPU catch up between them. GPU state carries from one list to the next on the queue.
const LIST_COMMANDS: u32 = 256;

impl CommandEncoder {
    pub(super) unsafe fn new(shared: alloc::sync::Arc<super::Shared>, addrs: DeviceAddrs) -> DeviceResult<Self> {
        let device = shared.device;
        let mut memory = Box::new(CommandMemory {
            device,
            blocks: Vec::new(),
        });
        let mut maker = unsafe { core::mem::zeroed::<DkCmdBufMaker>() };
        unsafe { dkCmdBufMakerDefaults(&mut maker, device) };
        maker.userData = core::ptr::from_mut(&mut *memory).cast();
        maker.cbAddMem = Some(add_memory);
        let cmdbuf = unsafe { dkCmdBufCreate(&maker) };
        if cmdbuf.is_null() {
            return Err(crate::DeviceError::OutOfMemory);
        }
        Ok(Self {
            cmdbuf,
            shared,
            _counted: super::Counted::new(&super::live::ENCODERS),
            memory,
            addrs,
            empty: true,
            lists: Vec::new(),
            commands: 0,
            entries: 0,
            barrier_pending: false,
            state: DrawState::default(),
            passes: Vec::new(),
            open_pass: None,
            open_pipeline: None,
        })
    }

    /// The command buffer, after the descriptor sets on a list's first command.
    fn cmd(&mut self) -> DkCmdBuf {
        let c = self.cmdbuf;
        if self.commands >= LIST_COMMANDS {
            self.finish_list();
        }
        self.commands += 1;
        if self.empty {
            self.empty = false;
            let f = self.addrs;
            unsafe {
                dkCmdBufBindImageDescriptorSet(c, f.images.0, f.images.1);
                dkCmdBufBindSamplerDescriptorSet(c, f.samplers.0, f.samplers.1);
                // Descriptors written since the last list may reuse freed indices.
                dkCmdBufBarrier(c, DkBarrier::DkBarrier_None, DkInvalidateFlags_Descriptors);
            }
        }
        if self.barrier_pending {
            self.barrier_pending = false;
            // A full barrier starts a GPFIFO entry of its own (deko3d's gpu_base.cpp).
            self.entries += 1;
            unsafe { dkCmdBufBarrier(c, DkBarrier::DkBarrier_Full, DkInvalidateFlags_Image | DkInvalidateFlags_Shader) };
        }
        c
    }

    fn finish_list(&mut self) {
        let list = unsafe { dkCmdBufFinishList(self.cmdbuf) };
        if list != 0 {
            self.lists.push((list, self.entries + LIST_ENTRIES));
        }
        self.commands = 0;
        self.entries = 0;
    }

    fn stages(&self) -> &'static [DkStage] {
        if self.state.compute {
            &[DkStage::DkStage_Compute]
        } else {
            &[DkStage::DkStage_Vertex, DkStage::DkStage_Fragment]
        }
    }

    /// Binds the textures and storage images the current pipeline reads, once per change.
    fn bind_textures(&mut self) {
        if !self.state.textures_dirty {
            return;
        }
        self.state.textures_dirty = false;
        let c = self.cmd();
        let stages = self.stages();
        let e = &mut self.state;
        for &stage in stages {
            let s = stage as usize;
            for &(t, m) in &e.sampling.textures {
                let image = e.textures.get(t as usize).copied().flatten().unwrap_or(0);
                let sampler = m.and_then(|m| e.samplers.get(m as usize).copied().flatten()).unwrap_or(0);
                let h = unsafe { dkMakeTextureHandle(image, sampler) };
                let Some(bound) = e.bound_textures[s].get_mut(t as usize) else { continue };
                if *bound != Some(h) {
                    *bound = Some(h);
                    unsafe { dkCmdBufBindTextures(c, stage, t as u32, &h, 1) };
                }
            }
            for &i in &e.sampling.images {
                let h = unsafe { dkMakeImageHandle(e.images.get(i as usize).copied().flatten().unwrap_or(0)) };
                let Some(bound) = e.bound_images[s].get_mut(i as usize) else { continue };
                if *bound != Some(h) {
                    *bound = Some(h);
                    unsafe { dkCmdBufBindImages(c, stage, i as u32, &h, 1) };
                }
            }
        }
    }

    fn write_first_instance(&mut self, first: u32) {
        if !self.state.first_instance || self.state.first_instance_written == Some(first) {
            return;
        }
        self.state.first_instance_written = Some(first);
        let (ubo, size) = self.addrs.ubo;
        let c = self.cmd();
        unsafe { dkCmdBufPushConstants(c, ubo, size, 0, 4, core::ptr::from_ref(&first).cast()) };
    }

    /// Binds the push constant block to the stages in use.
    fn bind_push_constants(&mut self) {
        let c = self.cmd();
        let gpu = self.addrs.ubo.0 + PUSH_CONSTANT_OFFSET;
        for &stage in self.stages() {
            unsafe { dkCmdBufBindUniformBuffer(c, stage, PUSH_CONSTANT_SLOT, gpu, PUSH_CONSTANT_SIZE) };
        }
    }

    /// Waits for earlier work and drops cached texels and shader data, so later reads see earlier writes.
    // TODO(wow-nx): pick the barrier from the usages (fragments only, no invalidation) once draws exist.
    fn barrier(&mut self) {
        self.barrier_pending = true;
    }
}

/// Whether a transition has to wait for earlier GPU work: when that work wrote, or when this one writes over what it
/// read. A first use (a texture uninitialized, a buffer the CPU wrote, which a flush makes visible) waits for nothing.
/// Each barrier is a GPFIFO entry, and a frame's uploads were one per copy: hundreds of entries (Queue::submit).
fn buffer_needs_barrier(t: &crate::StateTransition<wgt::BufferUses>) -> bool {
    use wgt::BufferUses as U;
    let gpu_from = t.from - (U::MAP_READ | U::MAP_WRITE);
    !gpu_from.is_empty() && (gpu_from.intersects(U::EXCLUSIVE) || t.to.intersects(U::EXCLUSIVE))
}

fn texture_needs_barrier(t: &crate::StateTransition<wgt::TextureUses>) -> bool {
    use wgt::TextureUses as U;
    let from = t.from - U::UNINITIALIZED;
    !from.is_empty() && (from.intersects(U::EXCLUSIVE) || t.to.intersects(U::EXCLUSIVE))
}

impl CommandEncoder {
    /// A GPU timestamp once the work before it is done, in a slot of the device's timing ring.
    fn timestamp(&mut self) -> u32 {
        let (slot, gpu) = self.shared.timing.lock().slot();
        let c = self.cmd();
        unsafe { dkCmdBufReportCounter(c, DkCounter::DkCounter_Timestamp, gpu) };
        slot
    }

    fn begin_timing(&mut self, label: crate::Label) {
        let start = self.timestamp();
        self.open_pass = Some((label.unwrap_or("unlabelled").into(), start));
    }

    /// A pipeline's draws from its bind to the next bind or the pass's end, under the pipeline's label.
    fn begin_pipeline_timing(&mut self, label: &str) {
        if !super::PIPELINE_TIMING.load(core::sync::atomic::Ordering::Relaxed) {
            return;
        }
        self.end_pipeline_timing();
        let start = self.timestamp();
        self.open_pipeline = Some((alloc::format!("{}{label}", super::PIPELINE_MARK), start));
    }

    fn end_pipeline_timing(&mut self) {
        if let Some((label, start)) = self.open_pipeline.take() {
            let end = self.timestamp();
            self.passes.push(super::PassTime { label, start, end });
        }
    }

    fn end_timing(&mut self) {
        self.end_pipeline_timing();
        if let Some((label, start)) = self.open_pass.take() {
            let end = self.timestamp();
            self.passes.push(super::PassTime { label, start, end });
        }
    }

    fn indirect(&mut self) {
        // An indirect draw or dispatch splits its entry and adds its parameters as another (gpu_3d_base.cpp).
        self.entries += 2;
    }
}

impl Drop for CommandEncoder {
    fn drop(&mut self) {
        unsafe {
            dkCmdBufDestroy(self.cmdbuf);
            for (b, _) in self.memory.blocks.drain(..) {
                dkMemBlockDestroy(b);
            }
        }
    }
}

/// A view of one mip level and layer, for copies.
fn subresource_view(tex: &Texture, base: &crate::TextureCopyBase, is_3d: bool) -> DkImageView {
    let mut view = unsafe { core::mem::zeroed::<DkImageView>() };
    unsafe { dkImageViewDefaults(&mut view, &*tex.image) };
    view.mipLevelOffset = base.mip_level as u8;
    view.mipLevelCount = 1;
    // Only an array image takes a layer: on a single-layer one deko3d reads a layer range as an overridden type.
    if !is_3d && tex.layers > 1 {
        view.layerOffset = base.array_layer as u16;
        view.layerCount = 1;
    }
    if base.aspect == crate::FormatAspects::STENCIL {
        view.dsSource = DkDsSource::DkDsSource_Stencil;
    }
    view
}

/// The buffer side of a buffer/texture copy: bytes per row and per image (deko3d's strides are in bytes).
fn buffer_side(buffer: &Buffer, tex: &Texture, copy: &crate::BufferTextureCopy) -> (DkCopyBuf, u64) {
    let format = tex.format;
    let (bw, bh) = format.block_dimensions();
    let mem_block = format.block_copy_size(Some(copy.texture_base.aspect.map())).unwrap_or(4) as u64;
    let row = copy
        .buffer_layout
        .bytes_per_row
        .map_or(copy.size.width.div_ceil(bw) as u64 * mem_block, u64::from);
    let rows = copy
        .buffer_layout
        .rows_per_image
        .map_or(copy.size.height.div_ceil(bh), |r| r.div_ceil(bh)) as u64;
    let image = row * rows;
    (
        DkCopyBuf {
            addr: buffer.gpu + copy.buffer_layout.offset,
            rowLength: row as u32,
            imageHeight: image as u32,
        },
        image,
    )
}

/// Copies between `buffer` and `tex`, one deko3d copy per array layer (3D textures in one).
unsafe fn copy_buffer_texture<T>(cmd: DkCmdBuf, buffer: &Buffer, tex: &Texture, regions: T, to_texture: bool)
where
    T: Iterator<Item = crate::BufferTextureCopy>,
{
    let is_3d = tex.is_3d();
    for copy in regions {
        let (mut side, per_image) = buffer_side(buffer, tex, &copy);
        let o = copy.texture_base.origin;
        let layers = if is_3d { 1 } else { copy.size.depth };
        for layer in 0..layers {
            let mut base = copy.texture_base.clone();
            base.array_layer += layer;
            let view = subresource_view(tex, &base, is_3d);
            let rect = DkImageRect {
                x: o.x,
                y: o.y,
                z: if is_3d { o.z } else { 0 },
                width: copy.size.width,
                height: copy.size.height,
                depth: if is_3d { copy.size.depth } else { 1 },
            };
            unsafe {
                if to_texture {
                    dkCmdBufCopyBufferToImage(cmd, &side, &view, &rect, 0);
                } else {
                    dkCmdBufCopyImageToBuffer(cmd, &view, &rect, &side, 0);
                }
            }
            side.addr += per_image;
        }
    }
}

impl crate::CommandEncoder for CommandEncoder {
    type A = Api;

    unsafe fn begin_encoding(&mut self, label: crate::Label) -> DeviceResult<()> {
        self.empty = true;
        self.lists.clear();
        self.commands = 0;
        self.entries = 0;
        self.barrier_pending = false;
        self.state = DrawState::default();
        self.passes.clear();
        self.open_pass = None;
        self.open_pipeline = None;
        Ok(())
    }
    unsafe fn discard_encoding(&mut self) {
        // The commands stay in memory until `reset_all`; the lists are simply never submitted.
        unsafe { dkCmdBufFinishList(self.cmdbuf) };
        self.lists.clear();
        self.commands = 0;
        self.entries = 0;
        self.barrier_pending = false;
        self.passes.clear();
        self.open_pass = None;
        self.open_pipeline = None;
    }
    unsafe fn end_encoding(&mut self) -> DeviceResult<CommandBuffer> {
        if self.barrier_pending {
            // A transition at the very end still reaches the GPU before what follows.
            self.cmd();
        }
        if !self.empty {
            self.finish_list();
        }
        self.commands = 0;
        self.entries = 0;
        Ok(CommandBuffer {
            lists: core::mem::take(&mut self.lists),
            passes: core::mem::take(&mut self.passes),
        })
    }
    unsafe fn reset_all<I>(&mut self, command_buffers: I) {
        // Every list from this encoder has executed: start over in the first block, free the rest.
        unsafe { dkCmdBufClear(self.cmdbuf) };
        let mut blocks = core::mem::take(&mut self.memory.blocks).into_iter();
        if let Some((b, size)) = blocks.next() {
            for (rest, _) in blocks {
                unsafe { dkMemBlockDestroy(rest) };
            }
            unsafe { dkCmdBufAddMemory(self.cmdbuf, b, 0, size) };
            self.memory.blocks.push((b, size));
        }
    }

    unsafe fn transition_buffers<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = crate::BufferBarrier<'a, Buffer>>,
    {
        if barriers.into_iter().any(|b| buffer_needs_barrier(&b.usage)) {
            self.barrier();
        }
    }

    unsafe fn transition_textures<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = crate::TextureBarrier<'a, Texture>>,
    {
        if barriers.into_iter().any(|b| texture_needs_barrier(&b.usage)) {
            self.barrier();
        }
    }

    unsafe fn clear_buffer(&mut self, buffer: &Buffer, range: crate::MemoryRange) {
        let (zeros, size) = self.addrs.zeros;
        let c = self.cmd();
        let mut started_at = range.start;
        while started_at < range.end {
            let n = (range.end - started_at).min(size as u64);
            unsafe { dkCmdBufCopyBuffer(c, zeros, buffer.gpu + started_at, n as u32) };
            started_at += n;
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
        let is_3d = src.is_3d();
        for r in regions {
            let layers = if is_3d { 1 } else { r.size.depth };
            for layer in 0..layers {
                let (mut a, mut b) = (r.src_base.clone(), r.dst_base.clone());
                a.array_layer += layer;
                b.array_layer += layer;
                let (va, vb) = (subresource_view(src, &a, is_3d), subresource_view(dst, &b, dst.is_3d()));
                let rect = |o: wgt::Origin3d| DkImageRect {
                    x: o.x,
                    y: o.y,
                    z: if is_3d { o.z } else { 0 },
                    width: r.size.width,
                    height: r.size.height,
                    depth: if is_3d { r.size.depth } else { 1 },
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
        unsafe { copy_buffer_texture(c, src, dst, regions, true) };
    }

    unsafe fn copy_texture_to_buffer<T>(&mut self, src: &Texture, src_usage: wgt::TextureUses, dst: &Buffer, regions: T)
    where
        T: Iterator<Item = crate::BufferTextureCopy>,
    {
        let c = self.cmd();
        unsafe { copy_buffer_texture(c, dst, src, regions, false) };
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
        self.begin_timing(desc.label);
        let c = self.cmd();
        let views: Vec<Option<DkImageView>> = desc
            .color_attachments
            .iter()
            .map(|a| {
                a.as_ref().map(|a| {
                    let mut v = a.target.view.view;
                    if let Some(slice) = a.depth_slice {
                        v.layerOffset = slice as u16;
                        v.layerCount = 1;
                    }
                    v
                })
            })
            .collect();
        // deko3d needs a dense array: an unused slot aliases a used target (pipelines give it no writes).
        let filler = views.iter().flatten().next().map(core::ptr::from_ref);
        let colors: Vec<*const DkImageView> = views
            .iter()
            .map(|v| v.as_ref().map_or(filler.unwrap_or(core::ptr::null()), core::ptr::from_ref))
            .collect();
        self.state.resolves = desc
            .color_attachments
            .iter()
            .flatten()
            .filter_map(|a| Some((a.target.view.view, a.resolve_target.as_ref()?.view.view)))
            .collect();
        let depth = desc.depth_stencil_attachment.as_ref().map(|d| {
            let mut v = d.target.view.view;
            v.dsSource = DkDsSource::DkDsSource_Depth;
            v
        });
        unsafe {
            dkCmdBufBindRenderTargets(
                c,
                colors.as_ptr(),
                colors.len() as u32,
                depth.as_ref().map_or(core::ptr::null(), core::ptr::from_ref),
            );
            let (w, h) = (desc.extent.width, desc.extent.height);
            dkCmdBufSetViewports(c, 0, &DkViewport { x: 0.0, y: 0.0, width: w as f32, height: h as f32, near: 0.0, far: 1.0 }, 1);
            dkCmdBufSetScissors(c, 0, &DkScissor { x: 0, y: 0, width: w, height: h }, 1);
            // Clears honour the colour write state: write everything first.
            let mut write_all = core::mem::zeroed::<DkColorWriteState>();
            dkColorWriteStateDefaults(&mut write_all);
            dkCmdBufBindColorWriteState(c, &write_all);
        }
        for (i, a) in desc.color_attachments.iter().enumerate() {
            let Some(a) = a else { continue };
            if a.ops.contains(crate::AttachmentOps::LOAD) {
                continue;
            }
            let k = a.clear_value;
            let i = i as u32;
            unsafe {
                match a.target.view.format.sample_type(None, None) {
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
            let clear_depth = !d.depth_ops.contains(crate::AttachmentOps::LOAD) && d.target.view.format.has_depth_aspect();
            let clear_stencil = !d.stencil_ops.contains(crate::AttachmentOps::LOAD) && d.target.view.format.has_stencil_aspect();
            if clear_depth || clear_stencil {
                unsafe {
                    let mut ds = core::mem::zeroed::<DkDepthStencilState>();
                    dkDepthStencilStateDefaults(&mut ds);
                    ds.set_depthWriteEnable(clear_depth as u32);
                    dkCmdBufBindDepthStencilState(c, &ds);
                    let (z, s) = d.clear_value;
                    dkCmdBufClearDepthStencil(c, clear_depth, z, if clear_stencil { 0xff } else { 0 }, s as u8);
                }
            }
        }
        Ok(())
    }
    unsafe fn end_render_pass(&mut self) {
        let c = self.cmd();
        let resolves = core::mem::take(&mut self.state.resolves);
        if !resolves.is_empty() {
            // A resolve is the 2D engine's, and reads what the pass's fragments wrote.
            unsafe { dkCmdBufBarrier(c, DkBarrier::DkBarrier_Fragments, DkInvalidateFlags_Image) };
        }
        for (src, dst) in resolves {
            unsafe { dkCmdBufResolveImage(c, &src, &dst) };
        }
        // No barrier after the pass: what reads or overwrites its targets next is a transition wgpu records before
        // that use (transition_textures), which waits; one here held every pass up behind the last.
        self.end_timing();
    }

    unsafe fn set_bind_group(
        &mut self,
        layout: &PipelineLayout,
        index: u32,
        group: &BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
        let c = self.cmd();
        let group_layout = &layout.groups[index as usize];
        let mut offsets = dynamic_offsets.iter();
        let mut textures = false;
        for r in group.resources.iter() {
            let slot = (group_layout.slots[r.binding as usize] + r.array_index) as u32;
            let dynamic_offset = |dynamic: bool, d: &mut core::slice::Iter<'_, u32>| {
                if dynamic { d.next().copied().unwrap_or(0) as u64 } else { 0 }
            };
            match r.kind {
                Kind::Uniform { gpu, size, dynamic } => {
                    let gpu = gpu + dynamic_offset(dynamic, &mut offsets);
                    for &stage in self.stages() {
                        unsafe { dkCmdBufBindUniformBuffer(c, stage, slot, gpu, size) };
                    }
                }
                Kind::Storage { gpu, size, dynamic } => {
                    let gpu = gpu + dynamic_offset(dynamic, &mut offsets);
                    for &stage in self.stages() {
                        unsafe { dkCmdBufBindStorageBuffer(c, stage, slot, gpu, size) };
                    }
                }
                Kind::SampledTexture { descriptor } => {
                    textures = true;
                    if let Some(t) = self.state.textures.get_mut(slot as usize) {
                        *t = Some(descriptor);
                    }
                }
                Kind::StorageImage { descriptor } => {
                    textures = true;
                    if let Some(t) = self.state.images.get_mut(slot as usize) {
                        *t = Some(descriptor);
                    }
                }
                Kind::BoundSampler { descriptor } => {
                    textures = true;
                    if let Some(t) = self.state.samplers.get_mut(slot as usize) {
                        *t = Some(descriptor);
                    }
                }
            }
        }
        // A group of uniforms alone (Bevy's view and mesh groups, at every draw) leaves the textures bound.
        self.state.textures_dirty |= textures;
    }
    unsafe fn set_push_constants(&mut self, layout: &PipelineLayout, stages: wgt::ShaderStages, offset_bytes: u32, data: &[u32]) {
        let c = self.cmd();
        let gpu = self.addrs.ubo.0 + PUSH_CONSTANT_OFFSET;
        let size = (data.len() * 4) as u32;
        if offset_bytes + size <= PUSH_CONSTANT_SIZE {
            unsafe { dkCmdBufPushConstants(c, gpu, PUSH_CONSTANT_SIZE, offset_bytes, size, data.as_ptr().cast()) };
        }
    }

    unsafe fn insert_debug_marker(&mut self, label: &str) {}
    unsafe fn begin_debug_marker(&mut self, group_label: &str) {}
    unsafe fn end_debug_marker(&mut self) {}

    unsafe fn set_render_pipeline(&mut self, p: &RenderPipeline) {
        self.begin_pipeline_timing(&p.label);
        let c = self.cmd();
        let mut shaders: arrayvec::ArrayVec<*const DkShader, 2> = arrayvec::ArrayVec::new();
        shaders.push(&*p.vs.shader);
        if let Some(fs) = &p.fs {
            shaders.push(&*fs.shader);
        }
        unsafe {
            dkCmdBufBindShaders(c, DkStageFlag_GraphicsMask, shaders.as_ptr(), shaders.len() as u32);
            dkCmdBufBindRasterizerState(c, &p.rasterizer);
            dkCmdBufBindMultisampleState(c, &p.multisample);
            dkCmdBufSetSampleMask(c, p.sample_mask);
            dkCmdBufBindColorState(c, &p.color);
            dkCmdBufBindColorWriteState(c, &p.color_write);
            if !p.blends.is_empty() {
                dkCmdBufBindBlendStates(c, 0, p.blends.as_ptr(), p.blends.len() as u32);
            }
            dkCmdBufBindDepthStencilState(c, &p.depth);
            dkCmdBufSetDepthBias(c, p.depth_bias.0, p.depth_bias.1, p.depth_bias.2);
            dkCmdBufSetStencil(c, DkFace::DkFace_FrontAndBack, p.stencil.1, self.state.stencil_ref, p.stencil.0);
            dkCmdBufBindVtxAttribState(c, p.attribs.as_ptr(), p.attribs.len() as u32);
            dkCmdBufBindVtxBufferState(c, p.vertex_buffers.as_ptr(), p.vertex_buffers.len() as u32);
            if p.first_instance {
                dkCmdBufBindUniformBuffer(c, DkStage::DkStage_Vertex, FIRST_INSTANCE_SLOT, self.addrs.ubo.0, 256);
            }
        }
        if p.push_constants {
            self.bind_push_constants();
        }
        let e = &mut self.state;
        e.sampling = p.sampling.clone();
        e.primitive = Some(p.primitive);
        e.strip = p.strip;
        e.first_instance = p.first_instance;
        e.first_instance_written = None;
        e.stencil = p.stencil;
        e.textures_dirty = true;
    }

    unsafe fn set_index_buffer<'a>(&mut self, binding: crate::BufferBinding<'a, Buffer>, format: wgt::IndexFormat) {
        let c = self.cmd();
        let f = match format {
            wgt::IndexFormat::Uint16 => DkIdxFormat::DkIdxFormat_Uint16,
            wgt::IndexFormat::Uint32 => DkIdxFormat::DkIdxFormat_Uint32,
        };
        unsafe { dkCmdBufBindIdxBuffer(c, f, binding.buffer.gpu + binding.offset) };
        self.state.index_format = Some(f);
    }
    unsafe fn set_vertex_buffer<'a>(&mut self, index: u32, binding: crate::BufferBinding<'a, Buffer>) {
        let c = self.cmd();
        let size = binding.size.map_or(binding.buffer.size - binding.offset, |s| s.get());
        unsafe { dkCmdBufBindVtxBuffer(c, index, binding.buffer.gpu + binding.offset, size as u32) };
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
        self.state.stencil_ref = value as u8;
        let (readback, color_write) = self.state.stencil;
        unsafe { dkCmdBufSetStencil(c, DkFace::DkFace_FrontAndBack, color_write, value as u8, readback) };
    }
    unsafe fn set_blend_constants(&mut self, color: &[f32; 4]) {
        let c = self.cmd();
        unsafe { dkCmdBufSetBlendConst(c, color[0], color[1], color[2], color[3]) };
    }

    unsafe fn draw(&mut self, first_vertex: u32, vertex_count: u32, first_instance: u32, instance_count: u32) {
        let Some(prim) = self.state.primitive else { return };
        self.bind_textures();
        self.write_first_instance(first_instance);
        let c = self.cmd();
        unsafe { dkCmdBufDraw(c, prim, vertex_count, instance_count, first_vertex, first_instance) };
    }
    unsafe fn draw_indexed(&mut self, first_index: u32, index_count: u32, base_vertex: i32, first_instance: u32, instance_count: u32) {
        let Some(prim) = self.state.primitive else { return };
        self.bind_textures();
        self.write_first_instance(first_instance);
        let c = self.cmd();
        let restart_index = if self.state.index_format == Some(DkIdxFormat::DkIdxFormat_Uint16) { 0xffff } else { 0xffff_ffff };
        unsafe {
            dkCmdBufSetPrimitiveRestart(c, self.state.strip, restart_index);
            dkCmdBufDrawIndexed(c, prim, index_count, instance_count, first_index, base_vertex, first_instance);
        }
    }
    unsafe fn draw_mesh_tasks(&mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {}
    unsafe fn draw_indirect(&mut self, buffer: &Buffer, offset: wgt::BufferAddress, draw_count: u32) {
        let Some(prim) = self.state.primitive else { return };
        self.bind_textures();
        self.write_first_instance(0);
        for i in 0..draw_count as u64 {
            let c = self.cmd();
            self.indirect();
            unsafe { dkCmdBufDrawIndirect(c, prim, buffer.gpu + offset + i * 16) };
        }
    }
    unsafe fn draw_indexed_indirect(&mut self, buffer: &Buffer, offset: wgt::BufferAddress, draw_count: u32) {
        let Some(prim) = self.state.primitive else { return };
        self.bind_textures();
        self.write_first_instance(0);
        let c = self.cmd();
        let restart_index = if self.state.index_format == Some(DkIdxFormat::DkIdxFormat_Uint16) { 0xffff } else { 0xffff_ffff };
        unsafe { dkCmdBufSetPrimitiveRestart(c, self.state.strip, restart_index) };
        for i in 0..draw_count as u64 {
            let c = self.cmd();
            self.indirect();
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
        self.state.compute = true;
        self.state.textures_dirty = true;
        self.begin_timing(desc.label);
    }
    unsafe fn end_compute_pass(&mut self) {
        self.state.compute = false;
        self.state.textures_dirty = true;
        // (as after a render pass, a later read of what the dispatches wrote is a transition, and waits)
        self.end_timing();
    }

    unsafe fn set_compute_pipeline(&mut self, p: &ComputePipeline) {
        self.begin_pipeline_timing(&p.label);
        let c = self.cmd();
        let cs: *const DkShader = &*p.cs.shader;
        unsafe { dkCmdBufBindShaders(c, DkStageFlag_Compute, &cs, 1) };
        if p.push_constants {
            self.bind_push_constants();
        }
        self.state.sampling = p.sampling.clone();
        self.state.textures_dirty = true;
    }

    unsafe fn dispatch(&mut self, count: [u32; 3]) {
        self.bind_textures();
        let c = self.cmd();
        unsafe { dkCmdBufDispatchCompute(c, count[0], count[1], count[2]) };
    }
    unsafe fn dispatch_indirect(&mut self, buffer: &Buffer, offset: wgt::BufferAddress) {
        self.bind_textures();
        let c = self.cmd();
        self.indirect();
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
