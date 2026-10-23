//! wow-nx: a wgpu-hal backend on deko3d, the Switch's native GPU API (docs/SWITCH-RENDER.md).
//!
//! On `target_os = "horizon"` this module is compiled as `gles` (see `lib.rs`), so wgpu-core and wgpu see the GL
//! backend and nothing above wgpu-hal changes. The map from WebGPU to deko3d follows nx-mod/dawn-nx's deko3d backend
//! (branch `switch-deko3d`, `src/dawn/native/deko3d`).
//!
//! What works: device, queue, fences, buffers, textures and views, the swapchain, render passes, copies, and
//! (untested on the console yet) shaders through naga and uam, pipelines, bind groups, samplers and draws.
//! Queries and acceleration structures are stubs.
#![allow(unused_variables)]

use alloc::{boxed::Box, string::String, sync::Arc, vec, vec::Vec};
use core::{ffi::c_void, ptr, time::Duration};

use parking_lot::Mutex;

use crate::TlasInstance;

mod buffer;
mod command;
mod conv;
mod pipeline;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unsafe_op_in_unsafe_fn,
    unused_qualifications,
    clippy::all
)]
mod ffi;

pub use buffer::Buffer;
pub use command::{CommandBuffer, CommandEncoder};
pub use pipeline::{BindGroup, BindGroupLayout, ComputePipeline, PipelineLayout, RenderPipeline, Sampler, ShaderModule};
use ffi::*;

#[link(name = "nx")]
unsafe extern "C" {
    fn nwindowGetDefault() -> *mut c_void;
}

/// Size of the zero-filled block buffer clears copy from.
const ZEROS_SIZE: u32 = 64 * 1024;
/// Command memory added to an encoder at a time.
const COMMAND_MEMORY_SIZE: u32 = 256 * 1024;
/// Shader code memory.
const CODE_MEMORY_SIZE: u32 = 32 << 20;
/// Descriptor set sizes: one image descriptor per sampled or storage view, one sampler descriptor per sampler.
const MAX_IMAGES: u32 = 8192;
const MAX_SAMPLERS: u32 = 2048;
/// The driver's own uniform buffer: the first instance at 0, push constants at 256 (pipeline.rs).
const DRIVER_UBO_SIZE: u32 = 256;

/// Live objects that own memory blocks, for finding what keeps the device alive at exit ([`live_objects`]).
pub(super) mod live {
    use core::sync::atomic::AtomicIsize;
    pub static BUFFERS: AtomicIsize = AtomicIsize::new(0);
    pub static TEXTURES: AtomicIsize = AtomicIsize::new(0);
    pub static ENCODERS: AtomicIsize = AtomicIsize::new(0);
    pub static SWAPCHAINS: AtomicIsize = AtomicIsize::new(0);
    pub static DEVICES: AtomicIsize = AtomicIsize::new(0);
}

/// What still owns deko3d memory: buffers, textures, encoders, swapchains, devices.
pub fn live_objects() -> String {
    use core::sync::atomic::Ordering::Relaxed;
    alloc::format!(
        "buffers {} textures {} encoders {} swapchains {} devices {}",
        live::BUFFERS.load(Relaxed),
        live::TEXTURES.load(Relaxed),
        live::ENCODERS.load(Relaxed),
        live::SWAPCHAINS.load(Relaxed),
        live::DEVICES.load(Relaxed)
    )
}

/// Counts one live object of a kind while it lives.
#[derive(Debug)]
pub(super) struct Counted(&'static core::sync::atomic::AtomicIsize);

impl Counted {
    pub(super) fn new(c: &'static core::sync::atomic::AtomicIsize) -> Self {
        c.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        Counted(c)
    }
}

impl Drop for Counted {
    fn drop(&mut self) {
        self.0.fetch_sub(1, core::sync::atomic::Ordering::Relaxed);
    }
}

pub(super) fn align_up(value: u64, alignment: u64) -> u64 {
    (value + alignment - 1) & !(alignment - 1)
}

/// A memory block of `size` bytes (a multiple of `DK_MEMBLOCK_ALIGNMENT`).
pub(super) unsafe fn mem_block(device: DkDevice, size: u32, flags: u32) -> Option<DkMemBlock> {
    let mut maker = unsafe { core::mem::zeroed::<DkMemBlockMaker>() };
    unsafe { dkMemBlockMakerDefaults(&mut maker, device, size) };
    maker.flags = flags;
    let mem_block = unsafe { dkMemBlockCreate(&maker) };
    (!mem_block.is_null()).then_some(mem_block)
}

unsafe extern "C" fn debug_callback(
    _user: *mut c_void,
    context: *const core::ffi::c_char,
    result: DkResult,
    message: *const core::ffi::c_char,
) {
    let text = |p: *const core::ffi::c_char| {
        if p.is_null() {
            String::new()
        } else {
            unsafe { core::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned()
        }
    };
    log::error!("deko3d {}: {:?} {}", text(context), result, text(message));
}

#[derive(Clone, Debug)]
pub struct Api;

#[derive(Debug)]
pub struct Instance;
#[derive(Debug)]
pub struct Adapter;

/// Shader code memory, filled from the start. Code is never freed.
// TODO(wow-nx): recycle the code of destroyed pipelines.
#[derive(Debug)]
struct CodeMemory {
    mem_block: DkMemBlock,
    used: u32,
}

impl CodeMemory {
    /// Loads a one-program DKSH file: the control section stays on the CPU, the code goes to code memory.
    fn load(&mut self, dksh: &[u8]) -> Option<Box<DkShader>> {
        let word = |i: usize| -> Option<u32> { Some(u32::from_le_bytes(dksh.get(i * 4..i * 4 + 4)?.try_into().ok()?)) };
        let [magic, _, control, code, _, programs] = [word(0)?, word(1)?, word(2)?, word(3)?, word(4)?, word(5)?];
        if magic != 0x4853_4B44 || programs != 1 || control as usize + code as usize > dksh.len() {
            log::error!("deko3d: not a one-program DKSH ({} bytes)", dksh.len());
            return None;
        }
        let offset = align_up(self.used as u64, DK_SHADER_CODE_ALIGNMENT as u64) as u32;
        if offset as u64 + code as u64 > (CODE_MEMORY_SIZE - DK_SHADER_CODE_UNUSABLE_SIZE) as u64 {
            log::error!("deko3d: shader code memory full");
            return None;
        }
        unsafe {
            let dst = dkMemBlockGetCpuAddr(self.mem_block).cast::<u8>().add(offset as usize);
            ptr::copy_nonoverlapping(dksh.as_ptr().add(control as usize), dst, code as usize);
        }
        self.used = offset + code;
        let mut maker = unsafe { core::mem::zeroed::<DkShaderMaker>() };
        unsafe { dkShaderMakerDefaults(&mut maker, self.mem_block, offset) };
        maker.control = dksh.as_ptr().cast();
        maker.programId = 0;
        let mut shader = Box::new(unsafe { core::mem::zeroed::<DkShader>() });
        unsafe { dkShaderInitialize(&mut *shader, &maker) };
        Some(shader)
    }
}

/// The image and sampler descriptor sets every command buffer binds, with free lists.
#[derive(Debug)]
struct Descriptors {
    images: DkMemBlock,
    samplers: DkMemBlock,
    free_images: Vec<u32>,
    free_samplers: Vec<u32>,
}

impl Descriptors {
    fn new_image(&mut self, view: &DkImageView, load_store: bool) -> Option<u32> {
        let i = self.free_images.pop()?;
        unsafe {
            let d = dkMemBlockGetCpuAddr(self.images).cast::<DkImageDescriptor>().add(i as usize);
            dkImageDescriptorInitialize(d, view, load_store, false);
        }
        Some(i)
    }
    fn new_sampler(&mut self, m: &DkSampler) -> Option<u32> {
        let i = self.free_samplers.pop()?;
        unsafe {
            let d = dkMemBlockGetCpuAddr(self.samplers).cast::<DkSamplerDescriptor>().add(i as usize);
            dkSamplerDescriptorInitialize(d, m);
        }
        Some(i)
    }
}

/// What the device and its queue share: deko3d's device, the one queue, a block of zeros for clears, shader code
/// memory, the descriptor sets and the driver's uniform buffer.
#[derive(Debug)]
pub(super) struct Shared {
    pub(super) device: DkDevice,
    queue: DkQueue,
    zeros: DkMemBlock,
    code: Mutex<CodeMemory>,
    descriptors: Mutex<Descriptors>,
    ubo: DkMemBlock,
    _counted: Counted,
}

unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

/// Everything that owns a memory block (the swapchain, encoders, buffers, textures) holds the shared state, so the
/// device goes last: deko3d refuses to destroy a device with blocks alive, and hbl cannot hand the console back to
/// the menu while a block stays mapped.
impl Drop for Shared {
    fn drop(&mut self) {
        unsafe {
            dkQueueWaitIdle(self.queue);
            dkQueueDestroy(self.queue);
            dkMemBlockDestroy(self.zeros);
            dkMemBlockDestroy(self.code.get_mut().mem_block);
            let d = self.descriptors.get_mut();
            dkMemBlockDestroy(d.images);
            dkMemBlockDestroy(d.samplers);
            dkMemBlockDestroy(self.ubo);
            dkDeviceDestroy(self.device);
        }
        log::warn!("deko3d: device destroyed");
    }
}

#[derive(Debug)]
pub struct Device {
    shared: Arc<Shared>,
}
#[derive(Debug)]
pub struct Queue {
    shared: Arc<Shared>,
}

/// The window's swapchain, made by `Surface::configure`.
#[derive(Debug)]
struct Swapchain {
    dk: DkSwapchain,
    memory: DkMemBlock,
    _shared: Arc<Shared>,
    _counted: Counted,
    images: Vec<DkImage>,
    queue: DkQueue,
    format: wgt::TextureFormat,
    size: wgt::Extent3d,
}

#[derive(Debug)]
pub struct Surface {
    window: *mut c_void,
    swapchain: Mutex<Option<Swapchain>>,
}

unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

/// A deko3d image. Views point at `image`, which is boxed so it keeps its address when the texture moves.
#[derive(Debug)]
pub struct Texture {
    image: Box<DkImage>,
    /// `None` for swapchain images, whose memory belongs to the swapchain.
    mem_block: Option<DkMemBlock>,
    _shared: Option<Arc<Shared>>,
    _counted: Option<Counted>,
    format: wgt::TextureFormat,
    samples: u32,
    /// Width and height of mip 0.
    size: (u32, u32),
    is_3d: bool,
    /// Array layers (1 for a 3D texture or one without layers).
    layers: u32,
    /// The swapchain slot of an acquired surface texture.
    slot: Option<i32>,
}

impl Texture {
    fn is_3d(&self) -> bool {
        self.is_3d
    }
}

unsafe impl Send for Texture {}
unsafe impl Sync for Texture {}

#[derive(Debug)]
pub struct TextureView {
    view: DkImageView,
    /// Index in the image descriptor set, for views that are sampled or stored to.
    descriptor: Option<u32>,
    format: wgt::TextureFormat,
    /// Size of the viewed mip level, for render passes.
    size: (u32, u32),
}

unsafe impl Send for TextureView {}
unsafe impl Sync for TextureView {}

#[derive(Debug)]
pub struct Resource;

/// Fence values, each signalled by a deko3d fence after a submission.
#[derive(Debug, Default)]
struct Pending {
    completed: crate::FenceValue,
    in_flight: Vec<(crate::FenceValue, Box<DkFence>)>,
}

#[derive(Debug)]
pub struct Fence {
    state: Mutex<Pending>,
}

unsafe impl Send for Fence {}
unsafe impl Sync for Fence {}

impl Fence {
    /// Retires every fence that has signalled, in order, and returns the last value reached.
    fn poll(&self) -> crate::FenceValue {
        let mut state = self.state.lock();
        while let Some((value, fence)) = state.in_flight.first_mut() {
            if unsafe { dkFenceWait(&mut **fence, 0) } != DkResult::DkResult_Success {
                break;
            }
            let value = *value;
            state.completed = state.completed.max(value);
            state.in_flight.remove(0);
        }
        state.completed
    }
}

type DeviceResult<T> = Result<T, crate::DeviceError>;

impl crate::Api for Api {
    const VARIANT: wgt::Backend = wgt::Backend::Gl;

    type Instance = Instance;
    type Surface = Surface;
    type Adapter = Adapter;
    type Device = Device;

    type Queue = Queue;
    type CommandEncoder = CommandEncoder;
    type CommandBuffer = CommandBuffer;

    type Buffer = Buffer;
    type Texture = Texture;
    type SurfaceTexture = Texture;
    type TextureView = TextureView;
    type Sampler = Sampler;
    type QuerySet = Resource;
    type Fence = Fence;
    type AccelerationStructure = Resource;
    type PipelineCache = Resource;

    type BindGroupLayout = BindGroupLayout;
    type BindGroup = BindGroup;
    type PipelineLayout = PipelineLayout;
    type ShaderModule = ShaderModule;
    type RenderPipeline = RenderPipeline;
    type ComputePipeline = ComputePipeline;
}

crate::impl_dyn_resource!(
    Adapter,
    BindGroup,
    BindGroupLayout,
    ComputePipeline,
    PipelineLayout,
    RenderPipeline,
    Sampler,
    ShaderModule,
    Buffer,
    CommandBuffer,
    CommandEncoder,
    Device,
    Fence,
    Instance,
    Queue,
    Resource,
    Surface,
    Texture,
    TextureView
);

impl crate::DynAccelerationStructure for Resource {}
impl crate::DynBindGroup for BindGroup {}
impl crate::DynBindGroupLayout for BindGroupLayout {}
impl crate::DynBuffer for Buffer {}
impl crate::DynCommandBuffer for CommandBuffer {}
impl crate::DynComputePipeline for ComputePipeline {}
impl crate::DynFence for Fence {}
impl crate::DynPipelineCache for Resource {}
impl crate::DynPipelineLayout for PipelineLayout {}
impl crate::DynQuerySet for Resource {}
impl crate::DynRenderPipeline for RenderPipeline {}
impl crate::DynSampler for Sampler {}
impl crate::DynShaderModule for ShaderModule {}
impl crate::DynSurfaceTexture for Texture {}
impl crate::DynTexture for Texture {}
impl crate::DynTextureView for TextureView {}

impl core::borrow::Borrow<dyn crate::DynTexture> for Texture {
    fn borrow(&self) -> &dyn crate::DynTexture {
        self
    }
}

impl crate::Instance for Instance {
    type A = Api;

    unsafe fn init(desc: &crate::InstanceDescriptor) -> Result<Self, crate::InstanceError> {
        Ok(Instance)
    }
    /// The Switch has one window, libnx's default `NWindow`; an Android NDK handle can name another.
    unsafe fn create_surface(
        &self,
        _display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<Surface, crate::InstanceError> {
        let window = match window_handle {
            raw_window_handle::RawWindowHandle::AndroidNdk(h) => h.a_native_window.as_ptr(),
            _ => unsafe { nwindowGetDefault() },
        };
        Ok(Surface {
            window,
            swapchain: Mutex::new(None),
        })
    }
    unsafe fn enumerate_adapters(&self, _surface_hint: Option<&Surface>) -> Vec<crate::ExposedAdapter<Api>> {
        vec![crate::ExposedAdapter {
            adapter: Adapter,
            info: adapter_info(),
            features: FEATURES,
            capabilities: capabilities(),
        }]
    }
}

pub fn adapter_info() -> wgt::AdapterInfo {
    wgt::AdapterInfo {
        name: String::from("NVIDIA Tegra X1 (deko3d)"),
        vendor: 0x10de,
        device: 0,
        device_type: wgt::DeviceType::IntegratedGpu,
        driver: String::from("deko3d"),
        driver_info: String::from("wow-nx wgpu-hal"),
        backend: wgt::Backend::Gl,
    }
}

// No INDIRECT_FIRST_INSTANCE: the first instance reaches shaders through the driver's uniform buffer, which an
// indirect draw does not update.
// TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES: Bevy requires it on the GL backend; texture_format_capabilities reports
// what each format does. The rest are what Maxwell and this backend do.
const FEATURES: wgt::Features = wgt::Features::TEXTURE_COMPRESSION_BC
    .union(wgt::Features::DEPTH32FLOAT_STENCIL8)
    .union(wgt::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
    .union(wgt::Features::FLOAT32_FILTERABLE)
    .union(wgt::Features::RG11B10UFLOAT_RENDERABLE)
    .union(wgt::Features::DEPTH_CLIP_CONTROL);

/// Maxwell's limits as deko3d exposes them, within WebGPU's defaults where those are lower.
fn capabilities() -> crate::Capabilities {
    let limits = wgt::Limits {
        max_texture_dimension_1d: 16384,
        max_texture_dimension_2d: 16384,
        max_texture_dimension_3d: 2048,
        max_texture_array_layers: 2048,
        // deko3d: 16 uniform buffers, 32 textures and 16 storage buffers per stage.
        max_uniform_buffers_per_shader_stage: 14,
        max_storage_buffers_per_shader_stage: 16,
        max_sampled_textures_per_shader_stage: 32,
        max_samplers_per_shader_stage: 32,
        max_storage_textures_per_shader_stage: 8,
        max_vertex_buffers: 16,
        max_vertex_attributes: 32,
        max_uniform_buffer_binding_size: 64 * 1024,
        max_buffer_size: 1 << 30,
        max_storage_buffer_binding_size: 1 << 30,
        min_uniform_buffer_offset_alignment: 256,
        min_storage_buffer_offset_alignment: 32,
        ..wgt::Limits::default()
    };
    crate::Capabilities {
        limits,
        alignments: crate::Alignments {
            buffer_copy_offset: wgt::BufferSize::new(4).unwrap(),
            buffer_copy_pitch: wgt::BufferSize::new(4).unwrap(),
            uniform_bounds_check_alignment: wgt::BufferSize::new(16).unwrap(),
            raw_tlas_instance_size: 0,
            ray_tracing_scratch_buffer_alignment: 1,
        },
        downlevel: wgt::DownlevelCapabilities::default(),
    }
}

/// Formats the swapchain can present.
const SURFACE_FORMATS: [wgt::TextureFormat; 2] =
    [wgt::TextureFormat::Rgba8UnormSrgb, wgt::TextureFormat::Rgba8Unorm];

impl Surface {
    unsafe fn release(&self) {
        if let Some(c) = self.swapchain.lock().take() {
            unsafe {
                dkQueueWaitIdle(c.queue);
                dkSwapchainDestroy(c.dk);
                dkMemBlockDestroy(c.memory);
            }
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.release() };
    }
}

impl crate::Surface for Surface {
    type A = Api;

    unsafe fn configure(&self, device: &Device, config: &crate::SurfaceConfiguration) -> Result<(), crate::SurfaceError> {
        unsafe { self.release() };
        let dk = device.shared.device;
        let format = conv::dk_format(config.format).ok_or(crate::SurfaceError::Other("format"))?;

        let mut maker = unsafe { core::mem::zeroed::<DkImageLayoutMaker>() };
        unsafe { dkImageLayoutMakerDefaults(&mut maker, dk) };
        maker.flags = DkImageFlags_UsageRender | DkImageFlags_UsagePresent | DkImageFlags_HwCompression;
        maker.format = format;
        maker.dimensions = [config.extent.width, config.extent.height, 1];
        let mut layout = unsafe { core::mem::zeroed::<DkImageLayout>() };
        unsafe { dkImageLayoutInitialize(&mut layout, &maker) };
        let alignment = unsafe { dkImageLayoutGetAlignment(&layout) } as u64;
        let size = align_up(unsafe { dkImageLayoutGetSize(&layout) }, alignment.max(1));

        let count = (config.maximum_frame_latency + 1).clamp(2, 3);
        let total = align_up(size * count as u64, DK_MEMBLOCK_ALIGNMENT as u64);
        let total = u32::try_from(total).map_err(|_| crate::SurfaceError::Outdated)?;
        let memory = unsafe { mem_block(dk, total, DkMemBlockFlags_GpuCached | DkMemBlockFlags_Image) }
            .ok_or(crate::SurfaceError::Device(crate::DeviceError::OutOfMemory))?;

        let mut images = vec![unsafe { core::mem::zeroed::<DkImage>() }; count as usize];
        for (i, image) in images.iter_mut().enumerate() {
            unsafe { dkImageInitialize(image, &layout, memory, (i as u64 * size) as u32) };
        }
        let pointers: Vec<*const DkImage> = images.iter().map(ptr::from_ref).collect();
        let mut sm = unsafe { core::mem::zeroed::<DkSwapchainMaker>() };
        unsafe { dkSwapchainMakerDefaults(&mut sm, dk, self.window, pointers.as_ptr(), count) };
        let swapchain = unsafe { dkSwapchainCreate(&sm) };
        if swapchain.is_null() {
            unsafe { dkMemBlockDestroy(memory) };
            return Err(crate::SurfaceError::Other("dkSwapchainCreate"));
        }
        let interval = if config.present_mode == wgt::PresentMode::Immediate { 0 } else { 1 };
        unsafe { dkSwapchainSetSwapInterval(swapchain, interval) };
        *self.swapchain.lock() = Some(Swapchain {
            dk: swapchain,
            memory,
            _shared: device.shared.clone(),
            _counted: Counted::new(&live::SWAPCHAINS),
            images,
            queue: device.shared.queue,
            format: config.format,
            size: config.extent,
        });
        Ok(())
    }

    unsafe fn unconfigure(&self, device: &Device) {
        unsafe { self.release() };
    }

    unsafe fn acquire_texture(
        &self,
        timeout: Option<Duration>,
        fence: &Fence,
    ) -> Result<Option<crate::AcquiredSurfaceTexture<Api>>, crate::SurfaceError> {
        let guard = self.swapchain.lock();
        let c = guard.as_ref().ok_or(crate::SurfaceError::Outdated)?;
        let slot = unsafe { dkQueueAcquireImage(c.queue, c.dk) };
        if slot < 0 {
            return Err(crate::SurfaceError::Lost);
        }
        Ok(Some(crate::AcquiredSurfaceTexture {
            texture: Texture {
                image: Box::new(c.images[slot as usize]),
                mem_block: None,
                _shared: None,
                _counted: None,
                format: c.format,
                samples: 1,
                size: (c.size.width, c.size.height),
                is_3d: false,
                layers: 1,
                slot: Some(slot),
            },
            suboptimal: false,
        }))
    }
    // deko3d cannot hand an acquired image back; the next present of that slot reuses it.
    unsafe fn discard_texture(&self, texture: Texture) {}
}

impl crate::Adapter for Adapter {
    type A = Api;

    unsafe fn open(
        &self,
        features: wgt::Features,
        _limits: &wgt::Limits,
        _memory_hints: &wgt::MemoryHints,
    ) -> DeviceResult<crate::OpenDevice<Api>> {
        let mut dm = unsafe { core::mem::zeroed::<DkDeviceMaker>() };
        // Depth [0, 1] and an upper-left origin: WebGPU's conventions.
        unsafe { dkDeviceMakerDefaults(&mut dm) };
        dm.cbDebug = Some(debug_callback);
        let device = unsafe { dkDeviceCreate(&dm) };
        if device.is_null() {
            return Err(crate::DeviceError::Lost);
        }
        let mut qm = unsafe { core::mem::zeroed::<DkQueueMaker>() };
        unsafe { dkQueueMakerDefaults(&mut qm, device) };
        qm.flags = DkQueueFlags_Graphics | DkQueueFlags_Compute | DkQueueFlags_HighPrio;
        let queue = unsafe { dkQueueCreate(&qm) };
        let zeros = unsafe {
            mem_block(device, ZEROS_SIZE, DkMemBlockFlags_CpuUncached | DkMemBlockFlags_GpuCached | DkMemBlockFlags_ZeroFillInit)
        };
        let plain = DkMemBlockFlags_CpuUncached | DkMemBlockFlags_GpuCached;
        let code = unsafe { mem_block(device, CODE_MEMORY_SIZE, plain | DkMemBlockFlags_Code) };
        let size_of_set = |n: u32, t: usize| align_up(n as u64 * t as u64, DK_MEMBLOCK_ALIGNMENT as u64) as u32;
        let images = unsafe { mem_block(device, size_of_set(MAX_IMAGES, core::mem::size_of::<DkImageDescriptor>()), plain) };
        let samplers = unsafe { mem_block(device, size_of_set(MAX_SAMPLERS, core::mem::size_of::<DkSamplerDescriptor>()), plain) };
        let ubo = unsafe { mem_block(device, DK_MEMBLOCK_ALIGNMENT, plain) };
        let (false, Some(zeros), Some(code), Some(images), Some(samplers), Some(ubo)) =
            (queue.is_null(), zeros, code, images, samplers, ubo)
        else {
            unsafe { dkDeviceDestroy(device) };
            return Err(crate::DeviceError::OutOfMemory);
        };
        let shared = Arc::new(Shared {
            device,
            queue,
            zeros,
            code: Mutex::new(CodeMemory { mem_block: code, used: 0 }),
            descriptors: Mutex::new(Descriptors {
                images,
                samplers,
                free_images: (0..MAX_IMAGES).rev().collect(),
                free_samplers: (0..MAX_SAMPLERS).rev().collect(),
            }),
            ubo,
            _counted: Counted::new(&live::DEVICES),
        });
        Ok(crate::OpenDevice {
            device: Device {
                shared: shared.clone(),
            },
            queue: Queue { shared },
        })
    }

    unsafe fn texture_format_capabilities(&self, format: wgt::TextureFormat) -> crate::TextureFormatCapabilities {
        use crate::TextureFormatCapabilities as C;
        if conv::dk_format(format).is_none() {
            return C::empty();
        }
        // What WebGPU guarantees for the format, and no more: every format Maxwell renders, stores or filters covers
        // it, and claiming more makes deko3d refuse the image (dk_image.cpp's format traits).
        use wgt::{TextureFormatFeatureFlags as F, TextureUsages as U};
        let g = format.guaranteed_format_features(FEATURES);
        let (u, f) = (g.allowed_usages, g.flags);
        let mut caps = C::empty();
        caps.set(C::SAMPLED, u.contains(U::TEXTURE_BINDING));
        caps.set(C::COPY_SRC, u.contains(U::COPY_SRC));
        caps.set(C::COPY_DST, u.contains(U::COPY_DST));
        if u.contains(U::RENDER_ATTACHMENT) {
            caps |= if format.is_depth_stencil_format() { C::DEPTH_STENCIL_ATTACHMENT } else { C::COLOR_ATTACHMENT };
        }
        caps.set(C::COLOR_ATTACHMENT_BLEND, f.contains(F::BLENDABLE));
        caps.set(C::SAMPLED_LINEAR, f.contains(F::FILTERABLE));
        caps.set(C::MULTISAMPLE_X2, f.contains(F::MULTISAMPLE_X2));
        caps.set(C::MULTISAMPLE_X4, f.contains(F::MULTISAMPLE_X4));
        caps.set(C::MULTISAMPLE_X8, f.contains(F::MULTISAMPLE_X8));
        caps.set(C::MULTISAMPLE_RESOLVE, f.contains(F::MULTISAMPLE_RESOLVE));
        caps.set(C::STORAGE_READ_ONLY, f.contains(F::STORAGE_READ_ONLY));
        caps.set(C::STORAGE_WRITE_ONLY, f.contains(F::STORAGE_WRITE_ONLY) || u.contains(U::STORAGE_BINDING));
        caps.set(C::STORAGE_READ_WRITE, f.contains(F::STORAGE_READ_WRITE));
        caps
    }

    unsafe fn surface_capabilities(&self, surface: &Surface) -> Option<crate::SurfaceCapabilities> {
        Some(crate::SurfaceCapabilities {
            formats: SURFACE_FORMATS.to_vec(),
            maximum_frame_latency: 1..=2,
            current_extent: None,
            usage: wgt::TextureUses::COLOR_TARGET | wgt::TextureUses::COPY_SRC | wgt::TextureUses::COPY_DST,
            present_modes: vec![wgt::PresentMode::Fifo, wgt::PresentMode::Immediate],
            composite_alpha_modes: vec![wgt::CompositeAlphaMode::Opaque],
        })
    }

    unsafe fn get_presentation_timestamp(&self) -> wgt::PresentationTimestamp {
        wgt::PresentationTimestamp::INVALID_TIMESTAMP
    }
}

impl crate::Queue for Queue {
    type A = Api;

    unsafe fn submit(
        &self,
        command_buffers: &[&CommandBuffer],
        surface_textures: &[&Texture],
        (fence, fence_value): (&mut Fence, crate::FenceValue),
    ) -> DeviceResult<()> {
        let queue = self.shared.queue;
        for cb in command_buffers {
            // Each list flushed as it goes: the channel's 2048 GPFIFO entries never fill (command.rs).
            for &list in &cb.lists {
                unsafe {
                    dkQueueSubmitCommands(queue, list);
                    dkQueueFlush(queue);
                }
            }
        }
        let mut dk = Box::new(unsafe { core::mem::zeroed::<DkFence>() });
        unsafe {
            dkQueueSignalFence(queue, &mut *dk, true);
            dkQueueFlush(queue);
        }
        #[cfg(deko3d_depurar)]
        {
            static SUBMISSIONS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
            let n = SUBMISSIONS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            unsafe { dkQueueWaitIdle(queue) };
            if unsafe { dkQueueIsInErrorState(queue) } {
                log::error!("deko3d: the GPU faulted in submission {n} ({} command buffers)", command_buffers.len());
            }
        }
        fence.state.lock().in_flight.push((fence_value, dk));
        Ok(())
    }
    unsafe fn present(&self, surface: &Surface, texture: Texture) -> Result<(), crate::SurfaceError> {
        let guard = surface.swapchain.lock();
        let c = guard.as_ref().ok_or(crate::SurfaceError::Outdated)?;
        let slot = texture.slot.ok_or(crate::SurfaceError::Other("not a surface texture"))?;
        unsafe { dkQueuePresentImage(c.queue, c.dk, slot) };
        Ok(())
    }

    unsafe fn get_timestamp_period(&self) -> f32 {
        // deko3d timestamps tick at 19.2 MHz (dkTimestampToNs).
        1e9 / 19_200_000.0
    }
}

impl crate::Device for Device {
    type A = Api;

    unsafe fn create_buffer(&self, desc: &crate::BufferDescriptor) -> DeviceResult<Buffer> {
        unsafe { Buffer::new(&self.shared, desc) }
    }

    unsafe fn destroy_buffer(&self, buffer: Buffer) {
        unsafe { dkMemBlockDestroy(buffer.mem_block) };
    }
    unsafe fn add_raw_buffer(&self, _buffer: &Buffer) {}

    unsafe fn map_buffer(&self, buffer: &Buffer, range: crate::MemoryRange) -> DeviceResult<crate::BufferMapping> {
        Ok(crate::BufferMapping {
            ptr: ptr::NonNull::new(buffer.ptr(&range)).unwrap(),
            is_coherent: true,
        })
    }
    unsafe fn unmap_buffer(&self, buffer: &Buffer) {}
    unsafe fn flush_mapped_ranges<I>(&self, buffer: &Buffer, ranges: I) {}
    unsafe fn invalidate_mapped_ranges<I>(&self, buffer: &Buffer, ranges: I) {}

    unsafe fn create_texture(&self, desc: &crate::TextureDescriptor) -> DeviceResult<Texture> {
        let dk = self.shared.device;
        let format = conv::dk_format(desc.format).ok_or(crate::DeviceError::Unexpected)?;
        let mut maker = unsafe { core::mem::zeroed::<DkImageLayoutMaker>() };
        unsafe { dkImageLayoutMakerDefaults(&mut maker, dk) };
        maker.type_ = conv::image_type(desc);
        maker.format = format;
        maker.msMode = conv::ms_mode(desc.sample_count);
        maker.mipLevels = desc.mip_level_count;
        maker.dimensions = [
            desc.size.width,
            if desc.dimension == wgt::TextureDimension::D1 { 1 } else { desc.size.height },
            desc.size.depth_or_array_layers,
        ];
        let render_usage = wgt::TextureUses::COLOR_TARGET
            | wgt::TextureUses::DEPTH_STENCIL_READ
            | wgt::TextureUses::DEPTH_STENCIL_WRITE;
        // Render usage only where the texture is drawn to: deko3d refuses it on a format it cannot render.
        if desc.usage.intersects(render_usage) {
            maker.flags |= DkImageFlags_UsageRender;
        }
        if desc.usage.intersects(wgt::TextureUses::STORAGE_READ_ONLY | wgt::TextureUses::STORAGE_WRITE_ONLY | wgt::TextureUses::STORAGE_READ_WRITE) {
            maker.flags |= DkImageFlags_UsageLoadStore;
        }
        // A multisample resolve (dkCmdBufResolveImage) runs on the 2D engine, which wants the flag on both images; it
        // takes no depth, compressed or 3D image (dk_image.cpp).
        if desc.usage.contains(wgt::TextureUses::COLOR_TARGET)
            && !desc.format.is_depth_stencil_format()
            && !desc.format.is_compressed()
            && desc.dimension != wgt::TextureDimension::D3
        {
            maker.flags |= DkImageFlags_Usage2DEngine;
        }
        maker.flags |= DkImageFlags_HwCompression;

        let mut layout = unsafe { core::mem::zeroed::<DkImageLayout>() };
        unsafe { dkImageLayoutInitialize(&mut layout, &maker) };
        let alignment = (unsafe { dkImageLayoutGetAlignment(&layout) } as u64).max(DK_MEMBLOCK_ALIGNMENT as u64);
        let size = align_up(unsafe { dkImageLayoutGetSize(&layout) }, alignment);
        let size = u32::try_from(size).map_err(|_| crate::DeviceError::OutOfMemory)?;
        let memory = unsafe { mem_block(dk, size, DkMemBlockFlags_GpuCached | DkMemBlockFlags_Image) }
            .ok_or(crate::DeviceError::OutOfMemory)?;
        let mut image = Box::new(unsafe { core::mem::zeroed::<DkImage>() });
        unsafe { dkImageInitialize(&mut *image, &layout, memory, 0) };
        Ok(Texture {
            image,
            mem_block: Some(memory),
            _shared: Some(self.shared.clone()),
            _counted: Some(Counted::new(&live::TEXTURES)),
            format: desc.format,
            samples: desc.sample_count,
            size: (desc.size.width, desc.size.height),
            is_3d: desc.dimension == wgt::TextureDimension::D3,
            layers: if desc.dimension == wgt::TextureDimension::D3 { 1 } else { desc.size.depth_or_array_layers },
            slot: None,
        })
    }
    unsafe fn destroy_texture(&self, texture: Texture) {
        if let Some(b) = texture.mem_block {
            unsafe { dkMemBlockDestroy(b) };
        }
    }
    unsafe fn add_raw_texture(&self, _texture: &Texture) {}

    unsafe fn create_texture_view(&self, texture: &Texture, desc: &crate::TextureViewDescriptor) -> DeviceResult<TextureView> {
        let mut view = unsafe { core::mem::zeroed::<DkImageView>() };
        unsafe { dkImageViewDefaults(&mut view, &*texture.image) };
        view.type_ = conv::view_type(desc.dimension, texture.samples > 1);
        if desc.format != texture.format {
            view.format = conv::dk_format(desc.format).ok_or(crate::DeviceError::Unexpected)?;
        }
        if desc.range.aspect == wgt::TextureAspect::StencilOnly {
            view.dsSource = DkDsSource::DkDsSource_Stencil;
        }
        let mip = desc.range.base_mip_level;
        view.mipLevelOffset = mip as u8;
        view.mipLevelCount = desc.range.mip_level_count.unwrap_or(0) as u8;
        // A layer range only on an array image (deko3d reads one on a single-layer image as an overridden type).
        if texture.layers > 1 {
            view.layerOffset = desc.range.base_array_layer as u16;
            view.layerCount = desc.range.array_layer_count.unwrap_or(0) as u16;
        }
        let (width, height) = texture.size;
        let storage = wgt::TextureUses::STORAGE_READ_ONLY | wgt::TextureUses::STORAGE_WRITE_ONLY | wgt::TextureUses::STORAGE_READ_WRITE;
        let descriptor = if desc.usage.intersects(wgt::TextureUses::RESOURCE | storage) {
            let d = self.shared.descriptors.lock().new_image(&view, desc.usage.intersects(storage));
            Some(d.ok_or(crate::DeviceError::OutOfMemory)?)
        } else {
            None
        };
        Ok(TextureView {
            view,
            descriptor,
            format: desc.format,
            size: ((width >> mip).max(1), (height >> mip).max(1)),
        })
    }
    unsafe fn destroy_texture_view(&self, view: TextureView) {
        if let Some(d) = view.descriptor {
            self.shared.descriptors.lock().free_images.push(d);
        }
    }
    unsafe fn create_sampler(&self, desc: &crate::SamplerDescriptor) -> DeviceResult<Sampler> {
        self.new_sampler(desc)
    }
    unsafe fn destroy_sampler(&self, sampler: Sampler) {
        self.shared.descriptors.lock().free_samplers.push(sampler.descriptor);
    }

    unsafe fn create_command_encoder(&self, desc: &crate::CommandEncoderDescriptor<Queue>) -> DeviceResult<CommandEncoder> {
        let c = &self.shared;
        let d = c.descriptors.lock();
        unsafe {
            CommandEncoder::new(
                c.clone(),
                command::DeviceAddrs {
                    zeros: (dkMemBlockGetGpuAddr(c.zeros), ZEROS_SIZE),
                    images: (dkMemBlockGetGpuAddr(d.images), MAX_IMAGES),
                    samplers: (dkMemBlockGetGpuAddr(d.samplers), MAX_SAMPLERS),
                    ubo: (dkMemBlockGetGpuAddr(c.ubo), DRIVER_UBO_SIZE),
                },
            )
        }
    }

    unsafe fn create_bind_group_layout(&self, desc: &crate::BindGroupLayoutDescriptor) -> DeviceResult<BindGroupLayout> {
        Ok(self.new_bind_group_layout(desc))
    }
    unsafe fn destroy_bind_group_layout(&self, bg_layout: BindGroupLayout) {}
    unsafe fn create_pipeline_layout(&self, desc: &crate::PipelineLayoutDescriptor<BindGroupLayout>) -> DeviceResult<PipelineLayout> {
        Ok(self.new_pipeline_layout(desc))
    }
    unsafe fn destroy_pipeline_layout(&self, pipeline_layout: PipelineLayout) {}
    unsafe fn create_bind_group(
        &self,
        desc: &crate::BindGroupDescriptor<BindGroupLayout, Buffer, Sampler, TextureView, Resource>,
    ) -> DeviceResult<BindGroup> {
        Ok(self.new_bind_group(desc))
    }
    unsafe fn destroy_bind_group(&self, group: BindGroup) {}

    unsafe fn create_shader_module(
        &self,
        desc: &crate::ShaderModuleDescriptor,
        shader: crate::ShaderInput,
    ) -> Result<ShaderModule, crate::ShaderError> {
        match shader {
            crate::ShaderInput::Naga(naga) => Ok(ShaderModule { naga }),
            _ => Err(crate::ShaderError::Compilation(String::from("deko3d takes naga modules only"))),
        }
    }
    unsafe fn destroy_shader_module(&self, module: ShaderModule) {}
    unsafe fn create_render_pipeline(
        &self,
        desc: &crate::RenderPipelineDescriptor<PipelineLayout, ShaderModule, Resource>,
    ) -> Result<RenderPipeline, crate::PipelineError> {
        self.new_render_pipeline(desc)
    }
    unsafe fn destroy_render_pipeline(&self, pipeline: RenderPipeline) {}
    unsafe fn create_compute_pipeline(
        &self,
        desc: &crate::ComputePipelineDescriptor<PipelineLayout, ShaderModule, Resource>,
    ) -> Result<ComputePipeline, crate::PipelineError> {
        self.new_compute_pipeline(desc)
    }
    unsafe fn destroy_compute_pipeline(&self, pipeline: ComputePipeline) {}
    unsafe fn create_pipeline_cache(&self, desc: &crate::PipelineCacheDescriptor<'_>) -> Result<Resource, crate::PipelineCacheError> {
        Ok(Resource)
    }
    unsafe fn destroy_pipeline_cache(&self, cache: Resource) {}

    unsafe fn create_query_set(&self, desc: &wgt::QuerySetDescriptor<crate::Label>) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_query_set(&self, set: Resource) {}
    unsafe fn create_fence(&self) -> DeviceResult<Fence> {
        Ok(Fence {
            state: Mutex::new(Pending::default()),
        })
    }
    unsafe fn destroy_fence(&self, fence: Fence) {}
    unsafe fn get_fence_value(&self, fence: &Fence) -> DeviceResult<crate::FenceValue> {
        Ok(fence.poll())
    }
    unsafe fn wait(&self, fence: &Fence, value: crate::FenceValue, timeout: Option<Duration>) -> DeviceResult<bool> {
        if fence.poll() >= value {
            return Ok(true);
        }
        let ns = timeout.map_or(-1, |t| i64::try_from(t.as_nanos()).unwrap_or(i64::MAX));
        {
            let mut state = fence.state.lock();
            let Some((_, dk)) = state.in_flight.iter_mut().find(|(v, _)| *v >= value) else {
                // Not submitted yet: nothing to wait on.
                return Ok(false);
            };
            if unsafe { dkFenceWait(&mut **dk, ns) } != DkResult::DkResult_Success {
                return Ok(false);
            }
        }
        Ok(fence.poll() >= value)
    }

    unsafe fn start_graphics_debugger_capture(&self) -> bool {
        false
    }
    unsafe fn stop_graphics_debugger_capture(&self) {}
    unsafe fn create_acceleration_structure(&self, desc: &crate::AccelerationStructureDescriptor) -> DeviceResult<Resource> {
        Err(crate::DeviceError::Unexpected)
    }
    unsafe fn get_acceleration_structure_build_sizes<'a>(
        &self,
        _desc: &crate::GetAccelerationStructureBuildSizesDescriptor<'a, Buffer>,
    ) -> crate::AccelerationStructureBuildSizes {
        Default::default()
    }
    unsafe fn get_acceleration_structure_device_address(&self, _acceleration_structure: &Resource) -> wgt::BufferAddress {
        Default::default()
    }
    unsafe fn destroy_acceleration_structure(&self, _acceleration_structure: Resource) {}

    fn tlas_instance_to_bytes(&self, instance: TlasInstance) -> Vec<u8> {
        vec![]
    }

    fn get_internal_counters(&self) -> wgt::HalCounters {
        Default::default()
    }

    fn check_if_oom(&self) -> DeviceResult<()> {
        Ok(())
    }
}
