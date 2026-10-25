//! The tuned Bevy boot: the `DefaultPlugins` set every benilla binary stands on. The engine tuning
//! is shared; the `Window` is the caller's.

use bevy::app::{PluginGroupBuilder, TaskPoolOptions, TaskPoolPlugin};
use bevy::prelude::*;

use crate::thread_qos;

/// `DefaultPlugins` with benilla's engine tuning applied, around the caller's primary window.
pub fn tuned_default_plugins(primary_window: Window) -> PluginGroupBuilder {
    let plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        })
        // Deliberately no `AssetPlugin::file_path`: a baked source path resolves only on the build
        // machine. Every shader is embedded (`embedded://<crate>/shaders/…`), so no root is read.
        // Quiet wgpu/naga; the ring keeps the last stderr lines for the crash report (`log_ring`).
        .set(bevy::log::LogPlugin {
            filter: "wgpu=error,naga=warn".into(),
            custom_layer: |_| Some(Box::new(crate::log_ring::LogRing)),
            ..default()
        })
        // Asset loads parse synchronously on the IO pool, and Bevy's default 4 threads saturate on
        // a dense teleport. Workers spawn at default QoS, behind any background build:
        // compute is user-interactive, IO and async compute user-initiated, and
        // `ThreadQosPlugin` promotes the render thread from inside.
        .set(TaskPoolPlugin {
            task_pool_options: TaskPoolOptions {
                io: bevy::app::TaskPoolThreadAssignmentPolicy {
                    min_threads: 2,
                    max_threads: 8,
                    percent: 0.5,
                    on_thread_spawn: Some(std::sync::Arc::new(|| {
                        thread_qos::promote_current_thread(thread_qos::QosClass::UserInitiated)
                    })),
                    on_thread_destroy: None,
                },
                async_compute: bevy::app::TaskPoolThreadAssignmentPolicy {
                    on_thread_spawn: Some(std::sync::Arc::new(|| {
                        thread_qos::promote_current_thread(thread_qos::QosClass::UserInitiated)
                    })),
                    ..TaskPoolOptions::default().async_compute
                },
                compute: bevy::app::TaskPoolThreadAssignmentPolicy {
                    on_thread_spawn: Some(std::sync::Arc::new(|| {
                        thread_qos::promote_current_thread(thread_qos::QosClass::UserInteractive)
                    })),
                    // `WOW_THREADS=1` serialises the frame's systems, a diagnostic: a defect that
                    // survives it is not a race between two systems.
                    max_threads: match std::env::var("WOW_THREADS").ok().as_deref() {
                        Some("1") => 1,
                        _ => TaskPoolOptions::default().compute.max_threads,
                    },
                    // wow-nx: std has no CPU count on Horizon, Bevy took that as one core, and the frame's systems
                    // ran on one worker: one per core of the process (thread.c).
                    #[cfg(target_os = "horizon")]
                    min_threads: match std::env::var("WOW_THREADS").ok().as_deref() {
                        Some("1") => 1,
                        _ => horizon_cores(),
                    },
                    ..TaskPoolOptions::default().compute
                },
                ..default()
            },
        })
        // Sound is kira behind our own mixer; `bevy_audio` is off by feature (workspace
        // `Cargo.toml`). Kept though they look idle: gizmos (bowstring, fishing line), sprites (the
        // FrameXML quad pass), picking, TextPlugin (glue text), PostProcessPlugin (glow bloom) and
        // ScenePlugin (avian's collider backend reads `SceneSpawner`); the ForwardDecal family is
        // registered inside `PbrPlugin::build`, so it cannot be disabled on its own.
        //
        // M2/WMO/ADT load through our own `mpq://` loaders; there is no glTF.
        .disable::<bevy::gltf::GltfPlugin>()
        // No bevy AA: no Fxaa/TAA/SMAA/CAS component anywhere (MSAA is core render, unaffected).
        .disable::<bevy::anti_alias::AntiAliasPlugin>();
    // No gamepad input; 1.12's bindings are keyboard/mouse.
    #[cfg(not(target_os = "horizon"))]
    let plugins = plugins.disable::<bevy::gilrs::GilrsPlugin>();
    // wow-nx: the Switch has no gilrs, and its GPU is wgpu's GL slot, where the deko3d backend lives
    // (third_party/wgpu-hal). Bevy's log goes to the SD card, the only place a Switch run can be read back.
    #[cfg(target_os = "horizon")]
    let plugins = plugins
        .set(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::WgpuSettings {
                backends: Some(bevy::render::settings::Backends::GL),
                ..default()
            }
            .into(),
            ..default()
        })
        .set(bevy::log::LogPlugin {
            // (wgpu_hal::gles is the Switch's deko3d backend: its frame statistics are info)
            filter: "wgpu=error,wgpu_hal::gles=info,naga=warn".into(),
            custom_layer: |_| {
                // Created, not appended: appending fails on the Switch's SD card.
                let f = std::fs::File::create("sdmc:/wow-nx/benilla.txt").ok()?;
                Some(Box::new(
                    bevy::log::tracing_subscriber::fmt::layer().with_ansi(false).with_writer(std::sync::Mutex::new(f)),
                ))
            },
            ..default()
        });
    plugins
}

/// The cores the process may run on (benilla-horizon's thread.c reads the kernel's core mask).
#[cfg(target_os = "horizon")]
fn horizon_cores() -> usize {
    unsafe extern "C" {
        fn wownx_core_count() -> u32;
    }
    unsafe { wownx_core_count() as usize }.max(1)
}
