//! wow-nx: what benilla needs on the Nintendo Switch that winit gives it elsewhere.
//!
//! - Paths: the install is `sdmc:/wow-nx/Data` and the state folder `sdmc:/wow-nx/benilla-config`, unless `WOW_DATA`
//!   or `BENILLA_HOME` say otherwise (the Switch has no executable path to resolve them from).
//! - The window: the primary window gets a raw handle to libnx's default `NWindow`, which wgpu's deko3d backend
//!   presents to (`third_party/wgpu-hal`).
//! - The main loop: the runner steps the app inside libnx's applet loop, which handles HOME, sleep and exit.
//!
//! Input is not mapped yet: the client boots to its login screen.

// Links what libnx gives std (getrandom, sysconf) and the getrandom crates' backends.
extern crate benilla_horizon;

use bevy::prelude::*;
use bevy::window::{RawHandleWrapper, WindowWrapper};

#[link(name = "nx")]
unsafe extern "C" {
    fn nwindowGetDefault() -> *mut core::ffi::c_void;
    fn appletMainLoop() -> bool;
}

/// Before anything reads the environment: the SD card paths, and an empty log.
pub(crate) fn prepare() {
    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    let _ = std::fs::write("sdmc:/wow-nx/benilla.txt", "");
    if std::env::var_os("WOW_DATA").is_none() {
        std::env::set_var("WOW_DATA", "sdmc:/wow-nx/Data");
    }
    if std::env::var_os("BENILLA_HOME").is_none() {
        std::env::set_var("BENILLA_HOME", "sdmc:/wow-nx/benilla-config");
    }
}

pub(crate) struct SwitchPlugin;

impl Plugin for SwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, give_window_handle).set_runner(run);
    }
}

/// libnx's default window, as raw-window-handle sees it (an Android NDK window: the backend takes its pointer).
struct NxWindow;

impl raw_window_handle::HasWindowHandle for NxWindow {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let w = core::ptr::NonNull::new(unsafe { nwindowGetDefault() })
            .ok_or(raw_window_handle::HandleError::Unavailable)?;
        let h = raw_window_handle::RawWindowHandle::AndroidNdk(raw_window_handle::AndroidNdkWindowHandle::new(w));
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
fn give_window_handle(mut commands: Commands, new: Query<Entity, (With<Window>, Without<RawHandleWrapper>)>) {
    for e in &new {
        match RawHandleWrapper::new(&WindowWrapper::new(NxWindow)) {
            Ok(h) => {
                commands.entity(e).insert(h);
            }
            Err(err) => error!("switch: window handle: {err}"),
        }
    }
}

/// Steps the app inside libnx's applet loop.
fn run(mut app: App) -> AppExit {
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
        if let Some(exit) = app.should_exit() {
            return exit;
        }
    }
}
