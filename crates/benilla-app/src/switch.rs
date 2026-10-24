//! wow-nx: what benilla needs on the Nintendo Switch that winit gives it elsewhere.
//!
//! - Paths: the install is `sdmc:/wow-nx/Data` and the state folder `sdmc:/wow-nx/benilla-config`, unless `WOW_DATA`
//!   or `BENILLA_HOME` say otherwise (the Switch has no executable path to resolve them from).
//! - The window: the primary window gets a raw handle to libnx's default `NWindow`, which wgpu's deko3d backend
//!   presents to (`third_party/wgpu-hal`), and is held at the window's own 1280x720, windowed, at scale 1
//!   ([`pin_window`]): benilla's video settings (`gxResolution`, 1600x900 by default; borderless fullscreen) would
//!   otherwise size the swapchain for a monitor the Switch does not have.
//! - The main loop: the runner steps the app inside libnx's applet loop, which handles HOME, sleep and exit.
//!
//! - Settings: `sdmc:/wow-nx/benilla.env`, `KEY=VALUE` lines read into the environment at start (`WOW_HOST`,
//!   `WOW_USER`, `WOW_PASS`...), so the login can be filled without a keyboard.
//! - Input ([`input`]): touch, a USB keyboard and mouse, and the Joy-Con, as the keyboard and mouse events winit sends.

// Links what libnx gives std (getrandom, sysconf) and the getrandom crates' backends.
extern crate benilla_horizon;

use bevy::prelude::*;
use bevy::window::{RawHandleWrapper, WindowWrapper};

#[link(name = "nx")]
unsafe extern "C" {
    fn nwindowGetDefault() -> *mut core::ffi::c_void;
    fn appletMainLoop() -> bool;
}

/// Before anything reads the environment: the SD card paths and settings.
pub(crate) fn prepare() {
    since_start_ms();
    unsafe extern "C" {
        /// benilla-horizon's screen.c.
        fn wownx_black_screen();
    }
    unsafe { wownx_black_screen() };
    let _ = std::fs::create_dir_all("sdmc:/wow-nx");
    // A panic aborts (the target's panic strategy), and newlib's abort leaves through libnx's exit: no crash report,
    // and the message went to a stderr nothing reads. The first panic's message, place and thread go to
    // sdmc:/wow-nx/panic.txt (created: appending fails on the SD card) and to the log.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        static FIRST: std::sync::Once = std::sync::Once::new();
        let thread = std::thread::current().name().unwrap_or("unnamed").to_owned();
        let text = format!("panic on thread {thread}: {info}\n");
        FIRST.call_once(|| {
            let _ = std::fs::write("sdmc:/wow-nx/panic.txt", &text);
        });
        error!("{text}");
        previous(info);
    }));
    if let Ok(text) = std::fs::read_to_string("sdmc:/wow-nx/benilla.env") {
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            if let Some((k, v)) = line.split_once('=') {
                if std::env::var_os(k.trim()).is_none() {
                    std::env::set_var(k.trim(), v.trim());
                }
            }
        }
    }
    std::env::set_var("BEVY_ASSET_ROOT", "sdmc:/wow-nx");
    if std::env::var_os("WOW_DATA").is_none() {
        std::env::set_var("WOW_DATA", "sdmc:/wow-nx/Data");
    }
    if std::env::var_os("BENILLA_HOME").is_none() {
        std::env::set_var("BENILLA_HOME", "sdmc:/wow-nx/benilla-config");
    }
}

/// When the process started, for the start's timings.
static STARTED: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn since_start_ms() -> u128 {
    STARTED.get_or_init(std::time::Instant::now).elapsed().as_millis()
}

/// A mark in the plugin order: its `finish` logs the time, so the log shows which plugins' `finish` took the start's
/// seconds (lib.rs places them).
pub(crate) struct FinishMark(pub(crate) &'static str);

impl Plugin for FinishMark {
    fn build(&self, _app: &mut App) {}
    fn finish(&self, _app: &mut App) {
        info!("switch: finish reached the {} mark at {} ms", self.0, since_start_ms());
    }
    fn is_unique(&self) -> bool {
        false
    }
}

pub(crate) struct SwitchPlugin;

impl Plugin for SwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, give_window_handle)
            .add_systems(PostUpdate, pin_window)
            .add_systems(PreUpdate, input::read.before(bevy::input::InputSystems))
            .add_systems(Update, input::system_keyboard.after(crate::ui_script::UiInput))
            .add_systems(Update, input::fps_overlay.in_set(crate::ui_pass::UiQuadAppend))
            .init_non_send_resource::<input::Hid>()
            .set_runner(run);
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

/// The screen libnx's default window has (docked, the console scales it to the TV).
const SCREEN: (u32, u32) = (1280, 720);

/// Holds every window at the screen's size, windowed, at scale 1, whatever the video settings asked.
fn pin_window(mut windows: Query<&mut Window>) {
    for mut w in &mut windows {
        let size_ok = w.resolution.physical_width() == SCREEN.0 && w.resolution.physical_height() == SCREEN.1;
        let scale_ok = w.resolution.scale_factor() == 1.0;
        if !size_ok || !scale_ok || w.mode != bevy::window::WindowMode::Windowed {
            w.mode = bevy::window::WindowMode::Windowed;
            w.resolution.set_scale_factor_override(Some(1.0));
            w.resolution.set_physical_resolution(SCREEN.0, SCREEN.1);
        }
    }
}

/// Steps the app inside libnx's applet loop.
///
/// The process ends on the exit frame, with no teardown: dropping the world (its assets, the GPU's objects) took
/// seconds, and the system frees all of it anyway (benilla-horizon's exit). The exit frame has saved the config,
/// and the log file is written unbuffered.
fn run(mut app: App) -> AppExit {
    let started = std::time::Instant::now();
    while app.plugins_state() == bevy::app::PluginsState::Adding {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    info!("switch: plugins ready at {} ms (process {} ms)", started.elapsed().as_millis(), since_start_ms());
    app.finish();
    info!("switch: plugins finished at {} ms", started.elapsed().as_millis());
    app.cleanup();
    info!("switch: plugins cleaned up at {} ms", started.elapsed().as_millis());
    let mut frames = 0u32;
    let exit = loop {
        if !unsafe { appletMainLoop() } {
            break AppExit::Success;
        }
        app.update();
        frames += 1;
        if frames == 1 || frames == 2 || frames == 60 {
            info!("switch: frame {frames} done at {} ms", started.elapsed().as_millis());
        }
        if let Some(exit) = app.should_exit() {
            break exit;
        }
    };
    info!("switch: exit after {frames} frames");
    // The render thread (pipelined rendering) may be presenting: take the render world back, which waits for its
    // frame, so nothing draws while the exit closes the display and GPU services under it.
    if let Some(mut channels) = app.world_mut().remove_resource::<bevy::render::pipelined_rendering::RenderAppChannels>() {
        let render_app = bevy::tasks::block_on(channels.recv());
        core::mem::forget((channels, render_app));
    }
    std::process::exit(match exit {
        AppExit::Success => 0,
        AppExit::Error(code) => code.get().into(),
    })
}

/// libnx's HID, as the events winit would send: a USB keyboard and mouse pass straight through; touch moves the
/// cursor and taps the left button; the Joy-Con play the keys and the mouse.
///
/// | Joy-Con | as | | Joy-Con | as |
/// |---|---|---|---|---|
/// | left stick | W A S D | | right stick | the cursor; with ZL held, the camera (right-drag) |
/// | ZR | left button | | ZL | right button |
/// | A | Space | | B | Escape |
/// | X | Tab | | Y | 1 |
/// | L / R | Q / E | | + | Enter |
/// | D-pad up, right, down, left | 2, 3, 4, 5 | | − | M |
/// | right stick press | the system keyboard, for the focused text box | | both sticks pressed | the fps overlay |
///
/// A tap or click on a text box opens the system keyboard too ([`input::system_keyboard`]).
mod input {
    use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKey};
    use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};
    use bevy::input::ButtonState;
    use bevy::prelude::*;
    use bevy::window::{CursorMoved, PrimaryWindow};

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct TouchState {
        delta_time: u64,
        attributes: u32,
        finger_id: u32,
        x: u32,
        y: u32,
        diameter_x: u32,
        diameter_y: u32,
        rotation_angle: u32,
        reserved: u32,
    }
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct TouchScreenState {
        sampling_number: u64,
        count: i32,
        reserved: u32,
        touches: [TouchState; 16],
    }
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct KeyboardState {
        sampling_number: u64,
        modifiers: u64,
        keys: [u64; 4],
    }
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct MouseState {
        sampling_number: u64,
        x: i32,
        y: i32,
        delta_x: i32,
        delta_y: i32,
        wheel_delta_x: i32,
        wheel_delta_y: i32,
        buttons: u32,
        attributes: u32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct PadState {
        id_mask: u8,
        active_id_mask: u8,
        read_handheld: bool,
        active_handheld: bool,
        style_set: u32,
        attributes: u32,
        buttons_cur: u64,
        buttons_old: u64,
        sticks: [[i32; 2]; 2],
        gc_triggers: [u32; 2],
    }

    #[link(name = "nx")]
    unsafe extern "C" {
        fn hidInitializeTouchScreen();
        fn hidInitializeKeyboard();
        fn hidInitializeMouse();
        fn hidGetTouchScreenStates(states: *mut TouchScreenState, count: usize) -> usize;
        fn hidGetKeyboardStates(states: *mut KeyboardState, count: usize) -> usize;
        fn hidGetMouseStates(states: *mut MouseState, count: usize) -> usize;
        fn padConfigureInput(max_players: u32, style_set: u32);
        fn padInitializeWithMask(pad: *mut PadState, mask: u64);
        fn padUpdate(pad: *mut PadState);
    }

    // libnx's HidNpadButton bits.
    const A: u64 = 1 << 0;
    const B: u64 = 1 << 1;
    const X: u64 = 1 << 2;
    const Y: u64 = 1 << 3;
    const L: u64 = 1 << 6;
    const R: u64 = 1 << 7;
    const ZL: u64 = 1 << 8;
    const ZR: u64 = 1 << 9;
    const PLUS: u64 = 1 << 10;
    const MINUS: u64 = 1 << 11;
    const LEFT: u64 = 1 << 12;
    const UP: u64 = 1 << 13;
    const RIGHT: u64 = 1 << 14;
    const DOWN: u64 = 1 << 15;
    const STICK_L: u64 = 1 << 4;
    const STICK_R: u64 = 1 << 5;
    /// Buttons that play a key.
    const KEYS: [(u64, KeyCode); 10] = [
        (A, KeyCode::Space),
        (B, KeyCode::Escape),
        (X, KeyCode::Tab),
        (Y, KeyCode::Digit1),
        (L, KeyCode::KeyQ),
        (R, KeyCode::KeyE),
        (PLUS, KeyCode::Enter),
        (MINUS, KeyCode::KeyM),
        (UP, KeyCode::Digit2),
        (RIGHT, KeyCode::Digit3),
    ];
    const MORE_KEYS: [(u64, KeyCode); 2] = [(DOWN, KeyCode::Digit4), (LEFT, KeyCode::Digit5)];
    /// A stick past this (of ±32767) is pressed.
    const DEAD: i32 = 9000;
    /// Cursor speed of the right stick at full tilt, logical px per second.
    const CURSOR_SPEED: f32 = 900.0;

    /// What was down last frame, to send presses and releases as winit does.
    pub(super) struct Hid {
        pad: PadState,
        keys: [u64; 4],
        mouse_buttons: u32,
        pad_keys: Vec<KeyCode>,
        pad_buttons: u64,
        touching: bool,
        cursor: Vec2,
        last_mouse: (i32, i32),
        /// A left press this frame (a tap, ZR, the mouse), and the right stick pressed in: either may open the
        /// system keyboard.
        tapped: bool,
        keyboard_asked: bool,
        /// The fps overlay ([`fps_overlay`]), both sticks pressed together.
        pub(super) overlay: bool,
    }

    impl Default for Hid {
        fn default() -> Self {
            let mut pad = PadState::default();
            unsafe {
                hidInitializeTouchScreen();
                hidInitializeKeyboard();
                hidInitializeMouse();
                // FullKey, Handheld, JoyDual, JoyLeft, JoyRight; player 1 and handheld.
                padConfigureInput(1, 0x1f);
                padInitializeWithMask(&mut pad, 1 | (1 << 0x20));
            }
            Hid {
                pad,
                keys: [0; 4],
                mouse_buttons: 0,
                pad_keys: Vec::new(),
                pad_buttons: 0,
                touching: false,
                cursor: Vec2::new(640.0, 360.0),
                last_mouse: (0, 0),
                tapped: false,
                keyboard_asked: false,
                overlay: false,
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn read(
        mut hid: NonSendMut<Hid>,
        time: Res<Time>,
        mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
        mut keys: MessageWriter<KeyboardInput>,
        mut buttons: MessageWriter<MouseButtonInput>,
        mut motion: MessageWriter<MouseMotion>,
        mut wheel: MessageWriter<MouseWheel>,
        mut moved: MessageWriter<CursorMoved>,
    ) {
        let Ok((window, mut win)) = windows.single_mut() else { return };
        let hid = &mut *hid;
        let size = Vec2::new(win.width(), win.height());
        let start = hid.cursor;
        let mut key = |code: KeyCode, pressed: bool, shift: bool| {
            let (logical, text) = logical(code, shift);
            keys.write(KeyboardInput {
                key_code: code,
                logical_key: logical,
                state: if pressed { ButtonState::Pressed } else { ButtonState::Released },
                text: if pressed { text.map(Into::into) } else { None },
                repeat: false,
                window,
            });
        };

        // USB keyboard: HID usages, one bit each.
        let mut kb = KeyboardState::default();
        if unsafe { hidGetKeyboardStates(&mut kb, 1) } > 0 {
            let shift = (kb.keys[3] >> (0xe1 - 0xc0)) & 1 != 0 || (kb.keys[3] >> (0xe5 - 0xc0)) & 1 != 0;
            for usage in 0..256usize {
                let (now, was) = ((kb.keys[usage / 64] >> (usage % 64)) & 1, (hid.keys[usage / 64] >> (usage % 64)) & 1);
                if now != was {
                    if let Some(code) = from_usage(usage as u8) {
                        key(code, now != 0, shift);
                    }
                }
            }
            hid.keys = kb.keys;
        }

        // Joy-Con.
        unsafe { padUpdate(&mut hid.pad) };
        let held = hid.pad.buttons_cur;
        let [ls, rs] = hid.pad.sticks;
        let mut want: Vec<KeyCode> = KEYS.iter().chain(MORE_KEYS.iter()).filter(|(b, _)| held & b != 0).map(|(_, k)| *k).collect();
        if ls[1] > DEAD {
            want.push(KeyCode::KeyW);
        }
        if ls[1] < -DEAD {
            want.push(KeyCode::KeyS);
        }
        if ls[0] < -DEAD {
            want.push(KeyCode::KeyA);
        }
        if ls[0] > DEAD {
            want.push(KeyCode::KeyD);
        }
        for k in hid.pad_keys.clone() {
            if !want.contains(&k) {
                key(k, false, false);
            }
        }
        for k in &want {
            if !hid.pad_keys.contains(k) {
                key(*k, true, false);
            }
        }
        hid.pad_keys = want;
        for (bit, button) in [(ZR, MouseButton::Left), (ZL, MouseButton::Right)] {
            if (held ^ hid.pad_buttons) & bit != 0 {
                let state = if held & bit != 0 { ButtonState::Pressed } else { ButtonState::Released };
                buttons.write(MouseButtonInput { button, state, window });
            }
        }
        let pressed = held & !hid.pad_buttons;
        hid.tapped = pressed & ZR != 0;
        let sticks = STICK_L | STICK_R;
        if held & sticks == sticks && pressed & sticks != 0 {
            hid.overlay = !hid.overlay;
        } else {
            hid.keyboard_asked = pressed & STICK_R != 0 && held & STICK_L == 0;
        }
        hid.pad_buttons = held;
        let tilt = |v: i32| if v.abs() > DEAD / 2 { v as f32 / 32767.0 } else { 0.0 };
        let (rx, ry) = (tilt(rs[0]), -tilt(rs[1]));
        if rx != 0.0 || ry != 0.0 {
            let step = Vec2::new(rx, ry) * CURSOR_SPEED * time.delta_secs();
            if held & ZL != 0 {
                // Right button held: the camera turns, the cursor stays.
                motion.write(MouseMotion { delta: step });
            } else {
                hid.cursor = (hid.cursor + step).clamp(Vec2::ZERO, size);
            }
        }

        // USB mouse.
        let mut m = MouseState::default();
        if unsafe { hidGetMouseStates(&mut m, 1) } > 0 && m.sampling_number != 0 {
            if (m.delta_x, m.delta_y) != (0, 0) && (m.x, m.y) != hid.last_mouse {
                motion.write(MouseMotion { delta: Vec2::new(m.delta_x as f32, m.delta_y as f32) });
                hid.cursor = (hid.cursor + Vec2::new(m.delta_x as f32, m.delta_y as f32)).clamp(Vec2::ZERO, size);
            }
            hid.last_mouse = (m.x, m.y);
            for (bit, button) in [(1, MouseButton::Left), (2, MouseButton::Right), (4, MouseButton::Middle)] {
                if (m.buttons ^ hid.mouse_buttons) & bit != 0 {
                    let state = if m.buttons & bit != 0 { ButtonState::Pressed } else { ButtonState::Released };
                    hid.tapped |= bit == 1 && state == ButtonState::Pressed;
                    buttons.write(MouseButtonInput { button, state, window });
                }
            }
            hid.mouse_buttons = m.buttons;
            if m.wheel_delta_y != 0 {
                wheel.write(MouseWheel {
                    unit: MouseScrollUnit::Line,
                    x: 0.0,
                    y: m.wheel_delta_y as f32 / 120.0,
                    window,
                });
            }
        }

        // Touch: the first finger is the cursor and the left button. The panel is 1280×720.
        let mut t = TouchScreenState::default();
        if unsafe { hidGetTouchScreenStates(&mut t, 1) } > 0 {
            let touching = t.count > 0;
            if touching {
                let p = Vec2::new(t.touches[0].x as f32, t.touches[0].y as f32) * size / Vec2::new(1280.0, 720.0);
                hid.cursor = p.clamp(Vec2::ZERO, size);
            }
            if touching != hid.touching {
                // The press lands where the finger is: move first, then press.
                if touching {
                    win.set_cursor_position(Some(hid.cursor));
                    moved.write(CursorMoved { window, position: hid.cursor, delta: Some(hid.cursor - start) });
                }
                let state = if touching { ButtonState::Pressed } else { ButtonState::Released };
                buttons.write(MouseButtonInput { button: MouseButton::Left, state, window });
                hid.touching = touching;
                hid.tapped |= touching;
            }
        }

        if hid.cursor != start || win.cursor_position().is_none() {
            win.set_cursor_position(Some(hid.cursor));
            moved.write(CursorMoved { window, position: hid.cursor, delta: Some(hid.cursor - start) });
        }
    }

    unsafe extern "C" {
        /// benilla-horizon's keyboard.c.
        fn wownx_keyboard(initial: *const core::ffi::c_char, password: i32, out: *mut u8, size: usize) -> i32;
    }

    /// The system keyboard for the focused text box: on a tap on the box, after the UI pass has given it the focus,
    /// or on the right stick pressed in. What the player typed replaces the box's text, typed in as a paste is, so
    /// the box's own rules and events apply.
    pub(super) fn system_keyboard(
        mut hid: NonSendMut<Hid>,
        script: Option<NonSendMut<benilla_ui::script::UiScript>>,
        hover: Res<crate::ui_script::PlayerUiHover>,
    ) {
        let (tapped, asked) = (core::mem::take(&mut hid.tapped), core::mem::take(&mut hid.keyboard_asked));
        let Some(mut script) = script else { return };
        if !tapped && !asked {
            return;
        }
        // A tap opens it over the box only; the stick, over whichever box has the focus.
        let target = if tapped && !asked { hover.0 } else { script.focused_editbox_id() };
        let Some((text, password)) = target.and_then(|id| script.focused_editbox_text(id)) else { return };
        let initial = std::ffi::CString::new(text).unwrap_or_default();
        let mut out = [0u8; 512];
        // The applet takes the screen, and the display refuses a frame the render thread queues meanwhile ("failed
        // to enqueue buffer", which deko3d raises as a break): the render thread waits at its next acquire or present
        // while the screen is held.
        let screen = wgpu_hal::gles::hold_screen();
        let confirmed = unsafe { wownx_keyboard(initial.as_ptr(), password as i32, out.as_mut_ptr(), out.len()) } != 0;
        drop(screen);
        if !confirmed {
            return;
        }
        let end = out.iter().position(|&b| b == 0).unwrap_or(out.len());
        let typed = String::from_utf8_lossy(&out[..end]).into_owned();
        use benilla_ui::script::{EditAction, EditUnit};
        script.editbox_action(EditAction::SelectAll);
        if typed.is_empty() {
            script.editbox_action(EditAction::Delete { unit: EditUnit::Char, back: true });
        } else {
            script.paste(&typed);
        }
    }

    /// The fps overlay, top left: frames a second, ms a frame, and the GPU's ms a frame (the deko3d backend's pass
    /// timestamps). Drawn on the player-UI quad pass as the dev build's pill is, laid out again four times a second.
    pub(super) fn fps_overlay(
        hid: NonSend<Hid>,
        time: Res<Time<bevy::time::Real>>,
        atlas: Option<Res<crate::ui_text::UiFontAtlas>>,
        mut quads: ResMut<crate::ui_pass::UiQuads>,
        mut cache: Local<(f32, Vec<crate::ui_pass::UiQuad>)>,
    ) {
        if !hid.overlay {
            return;
        }
        let Some(atlas) = atlas else { return };
        let now = time.elapsed_secs();
        if cache.1.is_empty() || now - cache.0 >= 0.25 {
            let (fps, gpu) = wgpu_hal::gles::frame_rate();
            let ms = if fps > 0.0 { 1000.0 / fps } else { 0.0 };
            let text = format!("{fps:.0} fps  {ms:.1} ms  GPU {gpu:.1} ms");
            let mut e = atlas.lock();
            let glyphs = crate::ui_text::layout_text_quads(
                &mut e,
                &text,
                Rect::from_corners(Vec2::new(10.0, 6.0), Vec2::new(400.0, 26.0)),
                [0.92, 0.92, 0.92, 1.0],
                crate::ui_text::Justify {
                    h: benilla_ui::script::JustifyH::Left,
                    v: benilla_ui::script::JustifyV::Middle,
                },
                u128::MAX,
                crate::ui_text::FontSpec {
                    path: None,
                    height: Some(14.0),
                    outline: benilla_ui::script::Outline::None,
                    monochrome: false,
                    spacing: 0.0,
                    shadow_x: 0.0,
                    alpha_gradient: None,
                    non_space_wrap: false,
                    max_lines: 0,
                },
                crate::ui_text::TextSeat::Exact,
            );
            drop(e);
            let bounds = glyphs.iter().map(|q| q.rect).reduce(|a, b| a.union(b)).unwrap_or(Rect::default());
            let mut out = vec![crate::ui_pass::UiQuad {
                rect: Rect::new(bounds.min.x - 6.0, bounds.min.y - 3.0, bounds.max.x + 6.0, bounds.max.y + 3.0),
                z_key: u128::MAX - 1,
                color: [0.0, 0.0, 0.0, 224.0 / 255.0],
                ..Default::default()
            }];
            out.extend(glyphs);
            *cache = (now, out);
        }
        quads.overlays.extend(cache.1.iter().cloned());
    }

    /// The key a code means on a US layout, and its text.
    fn logical(code: KeyCode, shift: bool) -> (Key, Option<String>) {
        use KeyCode as K;
        let named = match code {
            K::Enter | K::NumpadEnter => Some(Key::Enter),
            K::Escape => Some(Key::Escape),
            K::Backspace => Some(Key::Backspace),
            K::Tab => Some(Key::Tab),
            K::Delete => Some(Key::Delete),
            K::ArrowUp => Some(Key::ArrowUp),
            K::ArrowDown => Some(Key::ArrowDown),
            K::ArrowLeft => Some(Key::ArrowLeft),
            K::ArrowRight => Some(Key::ArrowRight),
            K::Home => Some(Key::Home),
            K::End => Some(Key::End),
            K::PageUp => Some(Key::PageUp),
            K::PageDown => Some(Key::PageDown),
            K::ShiftLeft | K::ShiftRight => Some(Key::Shift),
            K::ControlLeft | K::ControlRight => Some(Key::Control),
            K::AltLeft | K::AltRight => Some(Key::Alt),
            K::F1 => Some(Key::F1),
            K::F2 => Some(Key::F2),
            K::F3 => Some(Key::F3),
            K::F4 => Some(Key::F4),
            K::F5 => Some(Key::F5),
            K::F6 => Some(Key::F6),
            K::F7 => Some(Key::F7),
            K::F8 => Some(Key::F8),
            K::F9 => Some(Key::F9),
            K::F10 => Some(Key::F10),
            K::F11 => Some(Key::F11),
            K::F12 => Some(Key::F12),
            _ => None,
        };
        if let Some(k) = named {
            let text = match k {
                Key::Enter => Some("\r".to_string()),
                Key::Tab => Some("\t".to_string()),
                _ => None,
            };
            return (k, text);
        }
        let c = match code {
            K::Space => Some((' ', ' ')),
            K::KeyA => Some(('a', 'A')),
            K::KeyB => Some(('b', 'B')),
            K::KeyC => Some(('c', 'C')),
            K::KeyD => Some(('d', 'D')),
            K::KeyE => Some(('e', 'E')),
            K::KeyF => Some(('f', 'F')),
            K::KeyG => Some(('g', 'G')),
            K::KeyH => Some(('h', 'H')),
            K::KeyI => Some(('i', 'I')),
            K::KeyJ => Some(('j', 'J')),
            K::KeyK => Some(('k', 'K')),
            K::KeyL => Some(('l', 'L')),
            K::KeyM => Some(('m', 'M')),
            K::KeyN => Some(('n', 'N')),
            K::KeyO => Some(('o', 'O')),
            K::KeyP => Some(('p', 'P')),
            K::KeyQ => Some(('q', 'Q')),
            K::KeyR => Some(('r', 'R')),
            K::KeyS => Some(('s', 'S')),
            K::KeyT => Some(('t', 'T')),
            K::KeyU => Some(('u', 'U')),
            K::KeyV => Some(('v', 'V')),
            K::KeyW => Some(('w', 'W')),
            K::KeyX => Some(('x', 'X')),
            K::KeyY => Some(('y', 'Y')),
            K::KeyZ => Some(('z', 'Z')),
            K::Digit1 => Some(('1', '!')),
            K::Digit2 => Some(('2', '@')),
            K::Digit3 => Some(('3', '#')),
            K::Digit4 => Some(('4', '$')),
            K::Digit5 => Some(('5', '%')),
            K::Digit6 => Some(('6', '^')),
            K::Digit7 => Some(('7', '&')),
            K::Digit8 => Some(('8', '*')),
            K::Digit9 => Some(('9', '(')),
            K::Digit0 => Some(('0', ')')),
            K::Minus => Some(('-', '_')),
            K::Equal => Some(('=', '+')),
            K::BracketLeft => Some(('[', '{')),
            K::BracketRight => Some((']', '}')),
            K::Backslash => Some(('\\', '|')),
            K::Semicolon => Some((';', ':')),
            K::Quote => Some(('\'', '"')),
            K::Backquote => Some(('`', '~')),
            K::Comma => Some((',', '<')),
            K::Period => Some(('.', '>')),
            K::Slash => Some(('/', '?')),
            _ => None,
        };
        match c {
            Some((lo, hi)) => {
                let s = if shift { hi } else { lo }.to_string();
                if lo == ' ' {
                    (Key::Space, Some(s))
                } else {
                    (Key::Character(s.as_str().into()), Some(s))
                }
            }
            None => (Key::Unidentified(NativeKey::Unidentified), None),
        }
    }

    /// A USB HID usage as the key it names.
    fn from_usage(u: u8) -> Option<KeyCode> {
        use KeyCode as K;
        const LETTERS: [KeyCode; 26] = [
            K::KeyA, K::KeyB, K::KeyC, K::KeyD, K::KeyE, K::KeyF, K::KeyG, K::KeyH, K::KeyI, K::KeyJ, K::KeyK, K::KeyL,
            K::KeyM, K::KeyN, K::KeyO, K::KeyP, K::KeyQ, K::KeyR, K::KeyS, K::KeyT, K::KeyU, K::KeyV, K::KeyW, K::KeyX,
            K::KeyY, K::KeyZ,
        ];
        const DIGITS: [KeyCode; 10] = [
            K::Digit1, K::Digit2, K::Digit3, K::Digit4, K::Digit5, K::Digit6, K::Digit7, K::Digit8, K::Digit9, K::Digit0,
        ];
        const F: [KeyCode; 12] = [K::F1, K::F2, K::F3, K::F4, K::F5, K::F6, K::F7, K::F8, K::F9, K::F10, K::F11, K::F12];
        const PAD: [KeyCode; 10] = [
            K::Numpad1, K::Numpad2, K::Numpad3, K::Numpad4, K::Numpad5, K::Numpad6, K::Numpad7, K::Numpad8, K::Numpad9,
            K::Numpad0,
        ];
        Some(match u {
            0x04..=0x1d => LETTERS[(u - 0x04) as usize],
            0x1e..=0x27 => DIGITS[(u - 0x1e) as usize],
            0x28 => K::Enter,
            0x29 => K::Escape,
            0x2a => K::Backspace,
            0x2b => K::Tab,
            0x2c => K::Space,
            0x2d => K::Minus,
            0x2e => K::Equal,
            0x2f => K::BracketLeft,
            0x30 => K::BracketRight,
            0x31 => K::Backslash,
            0x33 => K::Semicolon,
            0x34 => K::Quote,
            0x35 => K::Backquote,
            0x36 => K::Comma,
            0x37 => K::Period,
            0x38 => K::Slash,
            0x39 => K::CapsLock,
            0x3a..=0x45 => F[(u - 0x3a) as usize],
            0x46 => K::PrintScreen,
            0x47 => K::ScrollLock,
            0x48 => K::Pause,
            0x49 => K::Insert,
            0x4a => K::Home,
            0x4b => K::PageUp,
            0x4c => K::Delete,
            0x4d => K::End,
            0x4e => K::PageDown,
            0x4f => K::ArrowRight,
            0x50 => K::ArrowLeft,
            0x51 => K::ArrowDown,
            0x52 => K::ArrowUp,
            0x53 => K::NumLock,
            0x54 => K::NumpadDivide,
            0x55 => K::NumpadMultiply,
            0x56 => K::NumpadSubtract,
            0x57 => K::NumpadAdd,
            0x58 => K::NumpadEnter,
            0x59..=0x62 => PAD[(u - 0x59) as usize],
            0x63 => K::NumpadDecimal,
            0xe0 => K::ControlLeft,
            0xe1 => K::ShiftLeft,
            0xe2 => K::AltLeft,
            0xe3 => K::SuperLeft,
            0xe4 => K::ControlRight,
            0xe5 => K::ShiftRight,
            0xe6 => K::AltRight,
            0xe7 => K::SuperRight,
            _ => return None,
        })
    }
}
