//! wow-nx render test (docs/SWITCH-RENDER.md): wgpu on the Switch through the deko3d backend in the GL slot.
//!
//! For five seconds it draws, on dark grey:
//! - left, a triangle from a vertex buffer: red at the top, green bottom left, blue bottom right;
//! - right, a quad with a 4x4 black and white checker texture, tinted by a uniform buffer, and placed by
//!   `instance_index` (drawn as instance 1 of 1..2, so a lost first instance puts it on the left instead).
//! Frames 0-59 only clear, 60-119 add the triangle, 120 on the quad (a GPU fault then names its draw). Frame 150 is
//! read back and written to sdmc:/wow-nx/clear.bmp. Each step and deko3d's messages go to
//! sdmc:/wow-nx/clear.txt, flushed line by line, so a crash leaves the last step reached.

extern crate benilla_horizon;

use std::io::Write;
use std::sync::Mutex;

#[link(name = "nx")]
unsafe extern "C" {
    fn nwindowGetDefault() -> *mut core::ffi::c_void;
}

static LOG_PATH: Mutex<Option<std::fs::File>> = Mutex::new(None);

fn log_line(text: &str) {
    if let Some(f) = LOG_PATH.lock().unwrap().as_mut() {
        let _ = writeln!(f, "{text}");
        let _ = f.sync_all();
    }
}

struct FileLogger;
impl log::Log for FileLogger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Warn
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            log_line(&format!("{} {}: {}", r.level(), r.target(), r.args()));
        }
    }
    fn flush(&self) {}
}

const SHADER: &str = r#"
struct VsOut { @builtin(position) pos: vec4f, @location(0) color: vec3f, @location(1) uv: vec2f };

@vertex fn vs_triangle(@location(0) pos: vec2f, @location(1) color: vec3f) -> VsOut {
    return VsOut(vec4f(pos, 0.0, 1.0), color, vec2f(0.0));
}
@fragment fn fs_triangle(e: VsOut) -> @location(0) vec4f {
    return vec4f(e.color, 1.0);
}

@group(0) @binding(0) var<uniform> tint: vec4f;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

@vertex fn vs_quad(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> VsOut {
    var corners = array(vec2f(0.0, 0.0), vec2f(1.0, 0.0), vec2f(0.0, 1.0), vec2f(1.0, 0.0), vec2f(1.0, 1.0), vec2f(0.0, 1.0));
    let c = corners[v];
    // Instance 1 lands on the right half; instance 0 would land on the left.
    let x = -0.8 + f32(i) * 1.0 + c.x * 0.6;
    let y = -0.4 + c.y * 0.8;
    return VsOut(vec4f(x, y, 0.0, 1.0), vec3f(0.0), vec2f(c.x, 1.0 - c.y));
}
@fragment fn fs_quad(e: VsOut) -> @location(0) vec4f {
    return textureSample(tex, samp, e.uv) * tint;
}
"#;

fn save_bmp(path: &str, width: u32, height: u32, row: u32, rgba: &[u8]) -> std::io::Result<()> {
    let size_of_set = 54 + width * height * 3;
    let mut b = Vec::with_capacity(size_of_set as usize);
    b.extend_from_slice(b"BM");
    for v in [size_of_set, 0, 54, 40, width, height] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&24u16.to_le_bytes());
    for v in [0u32, width * height * 3, 2835, 2835, 0, 0] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    for y in (0..height).rev() {
        let f = &rgba[(y * row) as usize..];
        for x in 0..width as usize {
            b.extend_from_slice(&[f[x * 4 + 2], f[x * 4 + 1], f[x * 4]]);
        }
    }
    std::fs::write(path, b)
}

fn main() {
    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    *LOG_PATH.lock().unwrap() = std::fs::File::create("sdmc:/wow-nx/clear.txt").ok();
    let _ = log::set_logger(&FileLogger).map(|()| log::set_max_level(log::LevelFilter::Warn));
    std::panic::set_hook(Box::new(|p| log_line(&format!("PANIC: {p}"))));

    log_line("instance");
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::GL, ..Default::default() });
    let window = core::ptr::NonNull::new(unsafe { nwindowGetDefault() }).expect("no default window");
    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: raw_window_handle::RawDisplayHandle::Android(raw_window_handle::AndroidDisplayHandle::new()),
            raw_window_handle: raw_window_handle::RawWindowHandle::AndroidNdk(raw_window_handle::AndroidNdkWindowHandle::new(window)),
        })
    }
    .expect("surface");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("adapter");
    log_line(&format!("adapter: {:?}", adapter.get_info()));
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("device");
    device.on_uncaptured_error(std::sync::Arc::new(|e| log_line(&format!("wgpu error: {e}"))));
    log_line("device");

    let (width, height) = (1280u32, 720u32);
    let format = surface.get_capabilities(&adapter).formats[0];
    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: format,
            width: width,
            height: height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        },
    );
    log_line(&format!("configured {format:?}"));

    use wgpu::util::DeviceExt;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("test"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let vertices: [f32; 15] = [-0.6, 0.6, 1.0, 0.0, 0.0, -0.9, -0.4, 0.0, 1.0, 0.0, -0.3, -0.4, 0.0, 0.0, 1.0];
    let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck_bytes(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let triangle = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("triangle"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_triangle"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 20,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_triangle"),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });
    log_line("pipeline triangle");
    let quad = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("quad"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_quad"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_quad"),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });
    log_line("pipeline quad");

    let mut texels = Vec::new();
    for y in 0..4 {
        for x in 0..4 {
            let v = if (x + y) % 2 == 0 { 255 } else { 0 };
            texels.extend_from_slice(&[v, v, v, 255]);
        }
    }
    let tex = device.create_texture_with_data(
        &queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: 4, height: 4, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        Default::default(),
        &texels,
    );
    let tint = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        // Yellow: the checker's white squares come out yellow.
        contents: bytemuck_bytes(&[1.0f32, 1.0, 0.0, 1.0]),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let sampler = device.create_sampler(&Default::default());
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &quad.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: tint.as_entire_binding() },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&tex.create_view(&Default::default())),
            },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
        ],
    });
    log_line("resources");

    let row = (width * 4).div_ceil(256) * 256;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    for frame_n in 0..300 {
        let frame = match surface.get_current_texture() {
            Ok(f) => f,
            Err(e) => {
                log_line(&format!("frame {frame_n}: {e}"));
                break;
            }
        };
        let view = frame.texture.create_view(&Default::default());
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            // In stages, so a GPU fault names its draw: frames 0-59 only clear, 60-119 add the triangle,
            // 120 on the quad too.
            if frame_n >= 60 {
                pass.set_pipeline(&triangle);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.draw(0..3, 0..1);
            }
            if frame_n >= 120 {
                pass.set_pipeline(&quad);
                pass.set_bind_group(0, &group, &[]);
                pass.draw(0..6, 1..2);
            }
        }
        if frame_n == 150 {
            enc.copy_texture_to_buffer(
                frame.texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: None },
                },
                wgpu::Extent3d { width: width, height: height, depth_or_array_layers: 1 },
            );
        }
        queue.submit([enc.finish()]);
        frame.present();
        if frame_n == 150 {
            readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            let data = readback.slice(..).get_mapped_range();
            match save_bmp("sdmc:/wow-nx/clear.bmp", width, height, row, &data) {
                Ok(()) => log_line("frame 150 saved to clear.bmp"),
                Err(e) => log_line(&format!("clear.bmp: {e}")),
            }
            drop(data);
            readback.unmap();
        }
        if frame_n < 3 || frame_n % 60 == 0 || (60..63).contains(&frame_n) || (120..123).contains(&frame_n) {
            log_line(&format!("frame {frame_n}"));
        }
    }
    log_line("done");
}

fn bytemuck_bytes(v: &[f32]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(v.as_ptr().cast(), core::mem::size_of_val(v)) }
}
