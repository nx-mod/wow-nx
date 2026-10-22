//! wow-nx render step 2 (docs/SWITCH-RENDER.md): wgpu clears the Switch's screen through the deko3d backend in the GL
//! slot. Red, green, blue, white for two seconds each, then it exits. Each step and deko3d's own messages go to
//! sdmc:/wow-nx/clear.txt, flushed line by line, so a crash leaves the last step reached.

extern crate benilla_horizon;

use std::io::Write;
use std::sync::Mutex;

#[link(name = "nx")]
unsafe extern "C" {
    fn nwindowGetDefault() -> *mut core::ffi::c_void;
}

static REGISTRO: Mutex<Option<std::fs::File>> = Mutex::new(None);

fn anotar(texto: &str) {
    if let Some(f) = REGISTRO.lock().unwrap().as_mut() {
        let _ = writeln!(f, "{texto}");
        let _ = f.sync_all();
    }
}

struct Registro;
impl log::Log for Registro {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Warn
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            anotar(&format!("{} {}: {}", r.level(), r.target(), r.args()));
        }
    }
    fn flush(&self) {}
}

fn main() {
    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    *REGISTRO.lock().unwrap() = std::fs::File::create("sdmc:/wow-nx/clear.txt").ok();
    let _ = log::set_logger(&Registro).map(|()| log::set_max_level(log::LevelFilter::Warn));

    anotar("instance");
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::GL, ..Default::default() });
    let ventana = core::ptr::NonNull::new(unsafe { nwindowGetDefault() }).expect("no default window");
    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: raw_window_handle::RawDisplayHandle::Android(raw_window_handle::AndroidDisplayHandle::new()),
            raw_window_handle: raw_window_handle::RawWindowHandle::AndroidNdk(raw_window_handle::AndroidNdkWindowHandle::new(ventana)),
        })
    }
    .expect("surface");
    anotar("adapter");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("adapter");
    anotar(&format!("adapter: {:?}", adapter.get_info()));
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("device");
    anotar("device");

    let formato = surface.get_capabilities(&adapter).formats[0];
    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: formato,
            width: 1280,
            height: 720,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        },
    );
    anotar(&format!("configured {formato:?}"));

    let colores = [wgpu::Color::RED, wgpu::Color::GREEN, wgpu::Color::BLUE, wgpu::Color::WHITE];
    for cuadro in 0..colores.len() * 120 {
        let frame = match surface.get_current_texture() {
            Ok(f) => f,
            Err(e) => {
                anotar(&format!("frame {cuadro}: {e}"));
                break;
            }
        };
        let vista = frame.texture.create_view(&Default::default());
        let mut enc = device.create_command_encoder(&Default::default());
        enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &vista,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(colores[cuadro / 120]), store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });
        queue.submit([enc.finish()]);
        frame.present();
        if cuadro % 120 == 0 {
            anotar(&format!("frame {cuadro}"));
        }
    }
    anotar("done");
}
