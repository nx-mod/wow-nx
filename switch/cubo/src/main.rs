//! wow-nx: Bevy on the Switch. A lit cube spins in front of a camera for 20 seconds, then the app exits.
//!
//! Without winit, two things are ours: the primary window gets a raw handle to libnx's default `NWindow` (the deko3d
//! backend presents to it), and the runner steps the app inside libnx's applet loop. Bevy's log and any panic go to
//! sdmc:/wow-nx/cubo.txt.

extern crate benilla_horizon;

use std::sync::Mutex;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::log::{BoxedLayer, LogPlugin};
use bevy::prelude::*;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::{RawHandleWrapper, WindowWrapper};

#[link(name = "nx")]
unsafe extern "C" {
    fn nwindowGetDefault() -> *mut core::ffi::c_void;
    fn appletMainLoop() -> bool;
}

const REGISTRO: &str = "sdmc:/wow-nx/cubo.txt";

/// libnx's default window, as raw-window-handle sees it (an Android NDK window: the deko3d backend takes its pointer).
struct VentanaNx;

impl raw_window_handle::HasWindowHandle for VentanaNx {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let v = core::ptr::NonNull::new(unsafe { nwindowGetDefault() }).ok_or(raw_window_handle::HandleError::Unavailable)?;
        let h = raw_window_handle::RawWindowHandle::AndroidNdk(raw_window_handle::AndroidNdkWindowHandle::new(v));
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(h) })
    }
}

impl raw_window_handle::HasDisplayHandle for VentanaNx {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let h = raw_window_handle::RawDisplayHandle::Android(raw_window_handle::AndroidDisplayHandle::new());
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(h) })
    }
}

/// Gives each new window the handle to libnx's.
fn dar_ventana(mut commands: Commands, nuevas: Query<Entity, (With<Window>, Without<RawHandleWrapper>)>) {
    for e in &nuevas {
        match RawHandleWrapper::new(&WindowWrapper::new(VentanaNx)) {
            Ok(h) => {
                commands.entity(e).insert(h);
            }
            Err(err) => error!("window handle: {err}"),
        }
    }
}

/// Steps the app inside libnx's applet loop (which handles HOME, sleep and exit requests).
fn bucle(mut app: App) -> AppExit {
    while app.plugins_state() == bevy::app::PluginsState::Adding {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    loop {
        if !unsafe { appletMainLoop() } {
            return AppExit::Success;
        }
        app.update();
        if let Some(salida) = app.should_exit() {
            return salida;
        }
    }
}

fn registro(_: &mut App) -> Option<BoxedLayer> {
    let f = std::fs::OpenOptions::new().append(true).create(true).open(REGISTRO).ok()?;
    Some(Box::new(bevy::log::tracing_subscriber::fmt::layer().with_ansi(false).with_writer(Mutex::new(f))))
}

fn escena(mut commands: Commands, mut mallas: ResMut<Assets<Mesh>>, mut materiales: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        Mesh3d(mallas.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materiales.add(StandardMaterial::from_color(Color::srgb(0.8, 0.5, 0.2)))),
        Transform::default(),
        Gira,
    ));
    commands.spawn((
        Mesh3d(mallas.add(Plane3d::default().mesh().size(4.0, 4.0))),
        MeshMaterial3d(materiales.add(StandardMaterial::from_color(Color::srgb(0.3, 0.5, 0.3)))),
        Transform::from_xyz(0.0, -0.75, 0.0),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(3.0, 6.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        // No tonemapping LUTs are built in; MSAA resolves come later.
        Tonemapping::None,
        Msaa::Off,
        Transform::from_xyz(-2.0, 1.8, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    info!("scene spawned");
}

#[derive(Component)]
struct Gira;

fn girar(time: Res<Time>, mut q: Query<&mut Transform, With<Gira>>) {
    for mut t in &mut q {
        t.rotate_y(time.delta_secs());
    }
}

fn contar(mut n: Local<u32>, time: Res<Time>, mut salir: MessageWriter<AppExit>) {
    *n += 1;
    if *n % 120 == 0 {
        info!("frame {} at {:.1} s", *n, time.elapsed_secs());
    }
    if time.elapsed_secs() > 20.0 {
        info!("done after {} frames", *n);
        salir.write(AppExit::Success);
    }
}

fn main() {
    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    let _ = std::fs::write(REGISTRO, "");
    std::panic::set_hook(Box::new(|p| {
        let _ = std::fs::OpenOptions::new()
            .append(true)
            .open(REGISTRO)
            .and_then(|mut f| std::io::Write::write_all(&mut f, format!("PANIC: {p}\n").as_bytes()));
    }));

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    custom_layer: registro,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions::with_num_threads(3),
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::GL),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, escena)
        .add_systems(PreUpdate, dar_ventana)
        .add_systems(Update, (girar, contar))
        .set_runner(bucle)
        .run();
}
