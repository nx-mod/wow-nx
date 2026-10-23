//! wow-nx: Bevy on the Switch. A lit cube spins in front of a camera for 20 seconds, then the app exits.
//!
//! Without winit, two things are ours: the primary window gets a raw handle to libnx's default `NWindow` (the deko3d
//! backend presents to it), and the runner steps the app inside libnx's applet loop. Bevy's log and any panic go to
//! sdmc:/wow-nx/cube.txt.

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

const LOG_PATH: &str = "sdmc:/wow-nx/cube.txt";

/// The log, created once and held: appending to a file fails on the Switch's SD card.
static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

/// Writes to [`LOG_FILE`], for Bevy's log layer.
struct LogWriter;

impl std::io::Write for LogWriter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if let Some(f) = LOG_FILE.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            f.write_all(b)?;
            let _ = f.sync_all();
        }
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn log_line(text: &str) {
    let _ = std::io::Write::write_all(&mut LogWriter, format!("{text}\n").as_bytes());
}

/// libnx's default window, as raw-window-handle sees it (an Android NDK window: the deko3d backend takes its pointer).
struct NxWindow;

impl raw_window_handle::HasWindowHandle for NxWindow {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let v = core::ptr::NonNull::new(unsafe { nwindowGetDefault() }).ok_or(raw_window_handle::HandleError::Unavailable)?;
        let h = raw_window_handle::RawWindowHandle::AndroidNdk(raw_window_handle::AndroidNdkWindowHandle::new(v));
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(h) })
    }
}

impl raw_window_handle::HasDisplayHandle for NxWindow {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let h = raw_window_handle::RawDisplayHandle::Android(raw_window_handle::AndroidDisplayHandle::new());
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(h) })
    }
}

/// Gives each new window the handle to libnx's.
fn give_window_handle(mut commands: Commands, new_windows: Query<Entity, (With<Window>, Without<RawHandleWrapper>)>) {
    for e in &new_windows {
        match RawHandleWrapper::new(&WindowWrapper::new(NxWindow)) {
            Ok(h) => {
                commands.entity(e).insert(h);
            }
            Err(err) => error!("window handle: {err}"),
        }
    }
}

/// Steps the app inside libnx's applet loop (which handles HOME, sleep and exit requests).
fn run_loop(mut app: App) -> AppExit {
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
        if let Some(output) = app.should_exit() {
            return output;
        }
    }
}

fn log_layer(_: &mut App) -> Option<BoxedLayer> {
    Some(Box::new(bevy::log::tracing_subscriber::fmt::layer().with_ansi(false).with_writer(|| LogWriter)))
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::srgb(0.8, 0.5, 0.2)))),
        Transform::default(),
        Spin,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4.0, 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::srgb(0.3, 0.5, 0.3)))),
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
struct Spin;

fn spin(time: Res<Time>, mut q: Query<&mut Transform, With<Spin>>) {
    for mut t in &mut q {
        t.rotate_y(time.delta_secs());
    }
}

fn count_frames(mut n: Local<u32>, time: Res<Time>, mut exit: MessageWriter<AppExit>) {
    *n += 1;
    if *n % 120 == 0 {
        info!("frame {} at {:.1} s", *n, time.elapsed_secs());
    }
    if time.elapsed_secs() > 20.0 {
        info!("done after {} frames", *n);
        exit.write(AppExit::Success);
    }
}

fn main() {
    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    *LOG_FILE.lock().unwrap() = std::fs::File::create(LOG_PATH).ok();
    std::panic::set_hook(Box::new(|p| log_line(&format!("PANIC: {p}"))));
    log_line("start");
    // No executable path on the Switch: bevy_asset's root.
    std::env::set_var("BEVY_ASSET_ROOT", "sdmc:/wow-nx");

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    custom_layer: log_layer,
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
        .add_systems(Startup, setup_scene)
        .add_systems(PreUpdate, give_window_handle)
        .add_systems(Update, (spin, count_frames))
        .set_runner(run_loop)
        .run();
}
