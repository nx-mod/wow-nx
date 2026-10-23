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
const CEROS: u32 = 64 * 1024;
/// Command memory added to an encoder at a time.
const MEMORIA_COMANDOS: u32 = 256 * 1024;
/// Shader code memory.
const MEMORIA_CODIGO: u32 = 32 << 20;
/// Descriptor set sizes: one image descriptor per sampled or storage view, one sampler descriptor per sampler.
const IMAGENES: u32 = 8192;
const MUESTREADORES: u32 = 2048;
/// The driver's own uniform buffer: the first instance at 0, push constants at 256 (pipeline.rs).
const UBO_DRIVER: u32 = 256;

pub(super) fn alinear(valor: u64, alineacion: u64) -> u64 {
    (valor + alineacion - 1) & !(alineacion - 1)
}

/// A memory block of `tamano` bytes (a multiple of `DK_MEMBLOCK_ALIGNMENT`).
pub(super) unsafe fn bloque(device: DkDevice, tamano: u32, flags: u32) -> Option<DkMemBlock> {
    let mut maker = unsafe { core::mem::zeroed::<DkMemBlockMaker>() };
    unsafe { dkMemBlockMakerDefaults(&mut maker, device, tamano) };
    maker.flags = flags;
    let bloque = unsafe { dkMemBlockCreate(&maker) };
    (!bloque.is_null()).then_some(bloque)
}

unsafe extern "C" fn depurar(
    _user: *mut c_void,
    contexto: *const core::ffi::c_char,
    resultado: DkResult,
    mensaje: *const core::ffi::c_char,
) {
    let texto = |p: *const core::ffi::c_char| {
        if p.is_null() {
            String::new()
        } else {
            unsafe { core::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned()
        }
    };
    log::error!("deko3d {}: {:?} {}", texto(contexto), resultado, texto(mensaje));
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
struct Codigo {
    bloque: DkMemBlock,
    usado: u32,
}

impl Codigo {
    /// Loads a one-program DKSH file: the control section stays on the CPU, the code goes to code memory.
    fn cargar(&mut self, dksh: &[u8]) -> Option<Box<DkShader>> {
        let palabra = |i: usize| -> Option<u32> { Some(u32::from_le_bytes(dksh.get(i * 4..i * 4 + 4)?.try_into().ok()?)) };
        let [magia, _, control, codigo, _, programas] = [palabra(0)?, palabra(1)?, palabra(2)?, palabra(3)?, palabra(4)?, palabra(5)?];
        if magia != 0x4853_4B44 || programas != 1 || control as usize + codigo as usize > dksh.len() {
            log::error!("deko3d: not a one-program DKSH ({} bytes)", dksh.len());
            return None;
        }
        let en = alinear(self.usado as u64, DK_SHADER_CODE_ALIGNMENT as u64) as u32;
        if en as u64 + codigo as u64 > (MEMORIA_CODIGO - DK_SHADER_CODE_UNUSABLE_SIZE) as u64 {
            log::error!("deko3d: shader code memory full");
            return None;
        }
        unsafe {
            let destino = dkMemBlockGetCpuAddr(self.bloque).cast::<u8>().add(en as usize);
            ptr::copy_nonoverlapping(dksh.as_ptr().add(control as usize), destino, codigo as usize);
        }
        self.usado = en + codigo;
        let mut maker = unsafe { core::mem::zeroed::<DkShaderMaker>() };
        unsafe { dkShaderMakerDefaults(&mut maker, self.bloque, en) };
        maker.control = dksh.as_ptr().cast();
        maker.programId = 0;
        let mut shader = Box::new(unsafe { core::mem::zeroed::<DkShader>() });
        unsafe { dkShaderInitialize(&mut *shader, &maker) };
        Some(shader)
    }
}

/// The image and sampler descriptor sets every command buffer binds, with free lists.
#[derive(Debug)]
struct Descriptores {
    imagenes: DkMemBlock,
    muestreadores: DkMemBlock,
    libres_imagenes: Vec<u32>,
    libres_muestreadores: Vec<u32>,
}

impl Descriptores {
    fn nueva_imagen(&mut self, vista: &DkImageView, carga_almacen: bool) -> Option<u32> {
        let i = self.libres_imagenes.pop()?;
        unsafe {
            let d = dkMemBlockGetCpuAddr(self.imagenes).cast::<DkImageDescriptor>().add(i as usize);
            dkImageDescriptorInitialize(d, vista, carga_almacen, false);
        }
        Some(i)
    }
    fn nuevo_muestreador(&mut self, m: &DkSampler) -> Option<u32> {
        let i = self.libres_muestreadores.pop()?;
        unsafe {
            let d = dkMemBlockGetCpuAddr(self.muestreadores).cast::<DkSamplerDescriptor>().add(i as usize);
            dkSamplerDescriptorInitialize(d, m);
        }
        Some(i)
    }
}

/// What the device and its queue share: deko3d's device, the one queue, a block of zeros for clears, shader code
/// memory, the descriptor sets and the driver's uniform buffer.
#[derive(Debug)]
struct Compartido {
    device: DkDevice,
    cola: DkQueue,
    ceros: DkMemBlock,
    codigo: Mutex<Codigo>,
    descriptores: Mutex<Descriptores>,
    ubo: DkMemBlock,
}

unsafe impl Send for Compartido {}
unsafe impl Sync for Compartido {}

impl Drop for Compartido {
    /// Waits for the GPU and leaves the rest to the process's exit: the device goes last in wgpu, after the surface
    /// and encoders that still hold memory blocks, and deko3d refuses to destroy a device with blocks alive.
    fn drop(&mut self) {
        unsafe { dkQueueWaitIdle(self.cola) };
    }
}

#[derive(Debug)]
pub struct Device {
    compartido: Arc<Compartido>,
}
#[derive(Debug)]
pub struct Queue {
    compartido: Arc<Compartido>,
}

/// The window's swapchain, made by `Surface::configure`.
#[derive(Debug)]
struct Cadena {
    dk: DkSwapchain,
    memoria: DkMemBlock,
    imagenes: Vec<DkImage>,
    cola: DkQueue,
    formato: wgt::TextureFormat,
    tamano: wgt::Extent3d,
}

#[derive(Debug)]
pub struct Surface {
    ventana: *mut c_void,
    cadena: Mutex<Option<Cadena>>,
}

unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

/// A deko3d image. Views point at `imagen`, which is boxed so it keeps its address when the texture moves.
#[derive(Debug)]
pub struct Texture {
    imagen: Box<DkImage>,
    /// `None` for swapchain images, whose memory belongs to the swapchain.
    bloque: Option<DkMemBlock>,
    formato: wgt::TextureFormat,
    muestras: u32,
    /// Width and height of mip 0.
    tamano: (u32, u32),
    tres_d: bool,
    /// The swapchain slot of an acquired surface texture.
    ranura: Option<i32>,
}

impl Texture {
    fn dimension_3d(&self) -> bool {
        self.tres_d
    }
}

unsafe impl Send for Texture {}
unsafe impl Sync for Texture {}

#[derive(Debug)]
pub struct TextureView {
    vista: DkImageView,
    /// Index in the image descriptor set, for views that are sampled or stored to.
    descriptor: Option<u32>,
    formato: wgt::TextureFormat,
    /// Size of the viewed mip level, for render passes.
    tamano: (u32, u32),
}

unsafe impl Send for TextureView {}
unsafe impl Sync for TextureView {}

#[derive(Debug)]
pub struct Resource;

/// Fence values, each signalled by a deko3d fence after a submission.
#[derive(Debug, Default)]
struct Pendientes {
    completado: crate::FenceValue,
    vuelo: Vec<(crate::FenceValue, Box<DkFence>)>,
}

#[derive(Debug)]
pub struct Fence {
    estado: Mutex<Pendientes>,
}

unsafe impl Send for Fence {}
unsafe impl Sync for Fence {}

impl Fence {
    /// Retires every fence that has signalled, in order, and returns the last value reached.
    fn sondear(&self) -> crate::FenceValue {
        let mut estado = self.estado.lock();
        while let Some((valor, fence)) = estado.vuelo.first_mut() {
            if unsafe { dkFenceWait(&mut **fence, 0) } != DkResult::DkResult_Success {
                break;
            }
            let valor = *valor;
            estado.completado = estado.completado.max(valor);
            estado.vuelo.remove(0);
        }
        estado.completado
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
        let ventana = match window_handle {
            raw_window_handle::RawWindowHandle::AndroidNdk(h) => h.a_native_window.as_ptr(),
            _ => unsafe { nwindowGetDefault() },
        };
        Ok(Surface {
            ventana,
            cadena: Mutex::new(None),
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
const FEATURES: wgt::Features = wgt::Features::TEXTURE_COMPRESSION_BC.union(wgt::Features::DEPTH32FLOAT_STENCIL8);

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
const FORMATOS_PANTALLA: [wgt::TextureFormat; 2] =
    [wgt::TextureFormat::Rgba8UnormSrgb, wgt::TextureFormat::Rgba8Unorm];

impl Surface {
    unsafe fn soltar(&self) {
        if let Some(c) = self.cadena.lock().take() {
            unsafe {
                dkQueueWaitIdle(c.cola);
                dkSwapchainDestroy(c.dk);
                dkMemBlockDestroy(c.memoria);
            }
        }
    }
}

impl crate::Surface for Surface {
    type A = Api;

    unsafe fn configure(&self, device: &Device, config: &crate::SurfaceConfiguration) -> Result<(), crate::SurfaceError> {
        unsafe { self.soltar() };
        let dk = device.compartido.device;
        let formato = conv::formato(config.format).ok_or(crate::SurfaceError::Other("format"))?;

        let mut maker = unsafe { core::mem::zeroed::<DkImageLayoutMaker>() };
        unsafe { dkImageLayoutMakerDefaults(&mut maker, dk) };
        maker.flags = DkImageFlags_UsageRender | DkImageFlags_UsagePresent | DkImageFlags_HwCompression;
        maker.format = formato;
        maker.dimensions = [config.extent.width, config.extent.height, 1];
        let mut layout = unsafe { core::mem::zeroed::<DkImageLayout>() };
        unsafe { dkImageLayoutInitialize(&mut layout, &maker) };
        let alineacion = unsafe { dkImageLayoutGetAlignment(&layout) } as u64;
        let tamano = alinear(unsafe { dkImageLayoutGetSize(&layout) }, alineacion.max(1));

        let cuenta = (config.maximum_frame_latency + 1).clamp(2, 3);
        let total = alinear(tamano * cuenta as u64, DK_MEMBLOCK_ALIGNMENT as u64);
        let total = u32::try_from(total).map_err(|_| crate::SurfaceError::Outdated)?;
        let memoria = unsafe { bloque(dk, total, DkMemBlockFlags_GpuCached | DkMemBlockFlags_Image) }
            .ok_or(crate::SurfaceError::Device(crate::DeviceError::OutOfMemory))?;

        let mut imagenes = vec![unsafe { core::mem::zeroed::<DkImage>() }; cuenta as usize];
        for (i, imagen) in imagenes.iter_mut().enumerate() {
            unsafe { dkImageInitialize(imagen, &layout, memoria, (i as u64 * tamano) as u32) };
        }
        let punteros: Vec<*const DkImage> = imagenes.iter().map(ptr::from_ref).collect();
        let mut sm = unsafe { core::mem::zeroed::<DkSwapchainMaker>() };
        unsafe { dkSwapchainMakerDefaults(&mut sm, dk, self.ventana, punteros.as_ptr(), cuenta) };
        let cadena = unsafe { dkSwapchainCreate(&sm) };
        if cadena.is_null() {
            unsafe { dkMemBlockDestroy(memoria) };
            return Err(crate::SurfaceError::Other("dkSwapchainCreate"));
        }
        let intervalo = if config.present_mode == wgt::PresentMode::Immediate { 0 } else { 1 };
        unsafe { dkSwapchainSetSwapInterval(cadena, intervalo) };
        *self.cadena.lock() = Some(Cadena {
            dk: cadena,
            memoria,
            imagenes,
            cola: device.compartido.cola,
            formato: config.format,
            tamano: config.extent,
        });
        Ok(())
    }

    unsafe fn unconfigure(&self, device: &Device) {
        unsafe { self.soltar() };
    }

    unsafe fn acquire_texture(
        &self,
        timeout: Option<Duration>,
        fence: &Fence,
    ) -> Result<Option<crate::AcquiredSurfaceTexture<Api>>, crate::SurfaceError> {
        let guarda = self.cadena.lock();
        let c = guarda.as_ref().ok_or(crate::SurfaceError::Outdated)?;
        let ranura = unsafe { dkQueueAcquireImage(c.cola, c.dk) };
        if ranura < 0 {
            return Err(crate::SurfaceError::Lost);
        }
        Ok(Some(crate::AcquiredSurfaceTexture {
            texture: Texture {
                imagen: Box::new(c.imagenes[ranura as usize]),
                bloque: None,
                formato: c.formato,
                muestras: 1,
                tamano: (c.tamano.width, c.tamano.height),
                tres_d: false,
                ranura: Some(ranura),
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
        dm.cbDebug = Some(depurar);
        let device = unsafe { dkDeviceCreate(&dm) };
        if device.is_null() {
            return Err(crate::DeviceError::Lost);
        }
        let mut qm = unsafe { core::mem::zeroed::<DkQueueMaker>() };
        unsafe { dkQueueMakerDefaults(&mut qm, device) };
        qm.flags = DkQueueFlags_Graphics | DkQueueFlags_Compute | DkQueueFlags_HighPrio;
        let cola = unsafe { dkQueueCreate(&qm) };
        let ceros = unsafe {
            bloque(device, CEROS, DkMemBlockFlags_CpuUncached | DkMemBlockFlags_GpuCached | DkMemBlockFlags_ZeroFillInit)
        };
        let normal = DkMemBlockFlags_CpuUncached | DkMemBlockFlags_GpuCached;
        let codigo = unsafe { bloque(device, MEMORIA_CODIGO, normal | DkMemBlockFlags_Code) };
        let tam = |n: u32, t: usize| alinear(n as u64 * t as u64, DK_MEMBLOCK_ALIGNMENT as u64) as u32;
        let imagenes = unsafe { bloque(device, tam(IMAGENES, core::mem::size_of::<DkImageDescriptor>()), normal) };
        let muestreadores = unsafe { bloque(device, tam(MUESTREADORES, core::mem::size_of::<DkSamplerDescriptor>()), normal) };
        let ubo = unsafe { bloque(device, DK_MEMBLOCK_ALIGNMENT, normal) };
        let (false, Some(ceros), Some(codigo), Some(imagenes), Some(muestreadores), Some(ubo)) =
            (cola.is_null(), ceros, codigo, imagenes, muestreadores, ubo)
        else {
            unsafe { dkDeviceDestroy(device) };
            return Err(crate::DeviceError::OutOfMemory);
        };
        let compartido = Arc::new(Compartido {
            device,
            cola,
            ceros,
            codigo: Mutex::new(Codigo { bloque: codigo, usado: 0 }),
            descriptores: Mutex::new(Descriptores {
                imagenes,
                muestreadores,
                libres_imagenes: (0..IMAGENES).rev().collect(),
                libres_muestreadores: (0..MUESTREADORES).rev().collect(),
            }),
            ubo,
        });
        Ok(crate::OpenDevice {
            device: Device {
                compartido: compartido.clone(),
            },
            queue: Queue { compartido },
        })
    }

    unsafe fn texture_format_capabilities(&self, format: wgt::TextureFormat) -> crate::TextureFormatCapabilities {
        use crate::TextureFormatCapabilities as C;
        if conv::formato(format).is_none() {
            return C::empty();
        }
        let mut caps = C::SAMPLED | C::COPY_SRC | C::COPY_DST;
        let entero = matches!(
            format.sample_type(None, None),
            Some(wgt::TextureSampleType::Uint | wgt::TextureSampleType::Sint)
        );
        let multimuestra = C::MULTISAMPLE_X2 | C::MULTISAMPLE_X4 | C::MULTISAMPLE_X8;
        if format.is_depth_stencil_format() {
            caps |= C::DEPTH_STENCIL_ATTACHMENT | multimuestra;
        } else if !format.is_compressed() {
            caps |= C::COLOR_ATTACHMENT | multimuestra | C::STORAGE_READ_ONLY | C::STORAGE_WRITE_ONLY | C::STORAGE_READ_WRITE;
            if !entero {
                caps |= C::COLOR_ATTACHMENT_BLEND | C::MULTISAMPLE_RESOLVE;
            }
        }
        if !entero {
            caps |= C::SAMPLED_LINEAR;
        }
        caps
    }

    unsafe fn surface_capabilities(&self, surface: &Surface) -> Option<crate::SurfaceCapabilities> {
        Some(crate::SurfaceCapabilities {
            formats: FORMATOS_PANTALLA.to_vec(),
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
        let cola = self.compartido.cola;
        for cb in command_buffers {
            if cb.lista != 0 {
                unsafe { dkQueueSubmitCommands(cola, cb.lista) };
            }
        }
        let mut dk = Box::new(unsafe { core::mem::zeroed::<DkFence>() });
        unsafe {
            dkQueueSignalFence(cola, &mut *dk, true);
            dkQueueFlush(cola);
        }
        #[cfg(deko3d_depurar)]
        {
            static ENVIOS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
            let n = ENVIOS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            unsafe { dkQueueWaitIdle(cola) };
            if unsafe { dkQueueIsInErrorState(cola) } {
                log::error!("deko3d: the GPU faulted in submission {n} ({} command buffers)", command_buffers.len());
            }
        }
        fence.estado.lock().vuelo.push((fence_value, dk));
        Ok(())
    }
    unsafe fn present(&self, surface: &Surface, texture: Texture) -> Result<(), crate::SurfaceError> {
        let guarda = surface.cadena.lock();
        let c = guarda.as_ref().ok_or(crate::SurfaceError::Outdated)?;
        let ranura = texture.ranura.ok_or(crate::SurfaceError::Other("not a surface texture"))?;
        unsafe { dkQueuePresentImage(c.cola, c.dk, ranura) };
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
        unsafe { Buffer::new(self.compartido.device, desc) }
    }

    unsafe fn destroy_buffer(&self, buffer: Buffer) {
        unsafe { dkMemBlockDestroy(buffer.bloque) };
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
        let dk = self.compartido.device;
        let formato = conv::formato(desc.format).ok_or(crate::DeviceError::Unexpected)?;
        let mut maker = unsafe { core::mem::zeroed::<DkImageLayoutMaker>() };
        unsafe { dkImageLayoutMakerDefaults(&mut maker, dk) };
        maker.type_ = conv::tipo_imagen(desc);
        maker.format = formato;
        maker.msMode = conv::muestras(desc.sample_count);
        maker.mipLevels = desc.mip_level_count;
        maker.dimensions = [
            desc.size.width,
            if desc.dimension == wgt::TextureDimension::D1 { 1 } else { desc.size.height },
            desc.size.depth_or_array_layers,
        ];
        let objetivo = wgt::TextureUses::COLOR_TARGET
            | wgt::TextureUses::DEPTH_STENCIL_READ
            | wgt::TextureUses::DEPTH_STENCIL_WRITE;
        if desc.usage.intersects(objetivo) || !desc.format.is_compressed() {
            // Renderable formats always get render usage, so clears to zero can be fast clears.
            maker.flags |= DkImageFlags_UsageRender;
        }
        if desc.usage.intersects(wgt::TextureUses::STORAGE_READ_ONLY | wgt::TextureUses::STORAGE_WRITE_ONLY | wgt::TextureUses::STORAGE_READ_WRITE) {
            maker.flags |= DkImageFlags_UsageLoadStore;
        }
        maker.flags |= DkImageFlags_HwCompression;

        let mut layout = unsafe { core::mem::zeroed::<DkImageLayout>() };
        unsafe { dkImageLayoutInitialize(&mut layout, &maker) };
        let alineacion = (unsafe { dkImageLayoutGetAlignment(&layout) } as u64).max(DK_MEMBLOCK_ALIGNMENT as u64);
        let tamano = alinear(unsafe { dkImageLayoutGetSize(&layout) }, alineacion);
        let tamano = u32::try_from(tamano).map_err(|_| crate::DeviceError::OutOfMemory)?;
        let memoria = unsafe { bloque(dk, tamano, DkMemBlockFlags_GpuCached | DkMemBlockFlags_Image) }
            .ok_or(crate::DeviceError::OutOfMemory)?;
        let mut imagen = Box::new(unsafe { core::mem::zeroed::<DkImage>() });
        unsafe { dkImageInitialize(&mut *imagen, &layout, memoria, 0) };
        Ok(Texture {
            imagen,
            bloque: Some(memoria),
            formato: desc.format,
            muestras: desc.sample_count,
            tamano: (desc.size.width, desc.size.height),
            tres_d: desc.dimension == wgt::TextureDimension::D3,
            ranura: None,
        })
    }
    unsafe fn destroy_texture(&self, texture: Texture) {
        if let Some(b) = texture.bloque {
            unsafe { dkMemBlockDestroy(b) };
        }
    }
    unsafe fn add_raw_texture(&self, _texture: &Texture) {}

    unsafe fn create_texture_view(&self, texture: &Texture, desc: &crate::TextureViewDescriptor) -> DeviceResult<TextureView> {
        let mut vista = unsafe { core::mem::zeroed::<DkImageView>() };
        unsafe { dkImageViewDefaults(&mut vista, &*texture.imagen) };
        vista.type_ = conv::tipo_vista(desc.dimension, texture.muestras > 1);
        if desc.format != texture.formato {
            vista.format = conv::formato(desc.format).ok_or(crate::DeviceError::Unexpected)?;
        }
        if desc.range.aspect == wgt::TextureAspect::StencilOnly {
            vista.dsSource = DkDsSource::DkDsSource_Stencil;
        }
        let mip = desc.range.base_mip_level;
        vista.mipLevelOffset = mip as u8;
        vista.mipLevelCount = desc.range.mip_level_count.unwrap_or(0) as u8;
        vista.layerOffset = desc.range.base_array_layer as u16;
        vista.layerCount = desc.range.array_layer_count.unwrap_or(0) as u16;
        let (ancho, alto) = texture.tamano;
        let almacen = wgt::TextureUses::STORAGE_READ_ONLY | wgt::TextureUses::STORAGE_WRITE_ONLY | wgt::TextureUses::STORAGE_READ_WRITE;
        let descriptor = if desc.usage.intersects(wgt::TextureUses::RESOURCE | almacen) {
            let d = self.compartido.descriptores.lock().nueva_imagen(&vista, desc.usage.intersects(almacen));
            Some(d.ok_or(crate::DeviceError::OutOfMemory)?)
        } else {
            None
        };
        Ok(TextureView {
            vista,
            descriptor,
            formato: desc.format,
            tamano: ((ancho >> mip).max(1), (alto >> mip).max(1)),
        })
    }
    unsafe fn destroy_texture_view(&self, view: TextureView) {
        if let Some(d) = view.descriptor {
            self.compartido.descriptores.lock().libres_imagenes.push(d);
        }
    }
    unsafe fn create_sampler(&self, desc: &crate::SamplerDescriptor) -> DeviceResult<Sampler> {
        self.nuevo_muestreador(desc)
    }
    unsafe fn destroy_sampler(&self, sampler: Sampler) {
        self.compartido.descriptores.lock().libres_muestreadores.push(sampler.descriptor);
    }

    unsafe fn create_command_encoder(&self, desc: &crate::CommandEncoderDescriptor<Queue>) -> DeviceResult<CommandEncoder> {
        let c = &self.compartido;
        let d = c.descriptores.lock();
        unsafe {
            CommandEncoder::new(
                c.device,
                command::Fijos {
                    ceros: (dkMemBlockGetGpuAddr(c.ceros), CEROS),
                    imagenes: (dkMemBlockGetGpuAddr(d.imagenes), IMAGENES),
                    muestreadores: (dkMemBlockGetGpuAddr(d.muestreadores), MUESTREADORES),
                    ubo: (dkMemBlockGetGpuAddr(c.ubo), UBO_DRIVER),
                },
            )
        }
    }

    unsafe fn create_bind_group_layout(&self, desc: &crate::BindGroupLayoutDescriptor) -> DeviceResult<BindGroupLayout> {
        Ok(self.nuevo_layout_grupos(desc))
    }
    unsafe fn destroy_bind_group_layout(&self, bg_layout: BindGroupLayout) {}
    unsafe fn create_pipeline_layout(&self, desc: &crate::PipelineLayoutDescriptor<BindGroupLayout>) -> DeviceResult<PipelineLayout> {
        Ok(self.nuevo_layout(desc))
    }
    unsafe fn destroy_pipeline_layout(&self, pipeline_layout: PipelineLayout) {}
    unsafe fn create_bind_group(
        &self,
        desc: &crate::BindGroupDescriptor<BindGroupLayout, Buffer, Sampler, TextureView, Resource>,
    ) -> DeviceResult<BindGroup> {
        Ok(self.nuevo_grupo(desc))
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
        self.nueva_tuberia(desc)
    }
    unsafe fn destroy_render_pipeline(&self, pipeline: RenderPipeline) {}
    unsafe fn create_compute_pipeline(
        &self,
        desc: &crate::ComputePipelineDescriptor<PipelineLayout, ShaderModule, Resource>,
    ) -> Result<ComputePipeline, crate::PipelineError> {
        self.nueva_tuberia_computo(desc)
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
            estado: Mutex::new(Pendientes::default()),
        })
    }
    unsafe fn destroy_fence(&self, fence: Fence) {}
    unsafe fn get_fence_value(&self, fence: &Fence) -> DeviceResult<crate::FenceValue> {
        Ok(fence.sondear())
    }
    unsafe fn wait(&self, fence: &Fence, value: crate::FenceValue, timeout: Option<Duration>) -> DeviceResult<bool> {
        if fence.sondear() >= value {
            return Ok(true);
        }
        let ns = timeout.map_or(-1, |t| i64::try_from(t.as_nanos()).unwrap_or(i64::MAX));
        {
            let mut estado = fence.estado.lock();
            let Some((_, dk)) = estado.vuelo.iter_mut().find(|(v, _)| *v >= value) else {
                // Not submitted yet: nothing to wait on.
                return Ok(false);
            };
            if unsafe { dkFenceWait(&mut **dk, ns) } != DkResult::DkResult_Success {
                return Ok(false);
            }
        }
        Ok(fence.sondear() >= value)
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
