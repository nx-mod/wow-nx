//! Gamepad play on top of benilla. 1.12.1 has no gamepad, so nothing here is reference behaviour.
//!
//! The split ("brain in Rust, face in Lua"):
//! - this crate reads the pad, picks the layer the triggers hold (bare / LT / RT / LT+RT), moves
//!   the body with the left stick and the camera with the right, and routes every button press;
//! - the BenillaPad addon ([`addon`], installed into `benilla-config/AddOns`) draws the gamepad
//!   bar and answers what a press does: `BenillaPad_Resolve(button, layer)` returns a binding
//!   command, which runs through the UI VM's `execute_binding`, the path a key takes, so the
//!   hardware-event gate and the press/release bodies behave as the keyboard's do. A command with
//!   a leading `@` is native (`@INTERACT`, the right-click context action on the target or a
//!   soft target, [`interact`]).
//!
//! The addon hears `BENILLAPAD_CONNECTED(style)`, `BENILLAPAD_DISCONNECTED`,
//! `BENILLAPAD_LAYER(layer)`, `BENILLAPAD_BUTTON(button, down)`, and outside the world mode
//! `BENILLAPAD_NAV(button, down)` and `BENILLAPAD_STICK(lx, ly, rx, ry)`. Without the addon the
//! pad still plays on the stock commands ([`map::fallback_command`]).
//!
//! Before the world (login, realm list, character select) the stock UI and the addon are not
//! loaded: the pad presses the keys those screens read and drives the mouse cursor
//! ([`drive_glue`]).

pub mod addon;
pub mod interact;
pub mod map;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input::ButtonState;
use bevy::input::InputSystems;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use benilla_ui::script::{ScriptValue, UiScript};

/// The gamepad plugin: re-enables bevy's gilrs backend, which benilla's boot disables, keeps the
/// addon installed, and runs [`drive_pad`] after bevy's input update so the commands land in this
/// frame's binding pass.
pub struct PadPlugin;

impl Plugin for PadPlugin {
    fn build(&self, app: &mut App) {
        // wow-nx: the Switch has no gilrs; its input layer reports the Joy-Con as bevy's gamepad instead.
        #[cfg(not(target_os = "horizon"))]
        app.add_plugins(bevy::gilrs::GilrsPlugin);
        app.insert_resource(PadSettings::from_env())
            .init_resource::<PadState>()
            .add_systems(Startup, addon::install_addon)
            .add_systems(PreUpdate, (drive_pad, run_calls).chain().after(InputSystems));
    }
}

/// A Lua call a press makes: a binding command down or up, an addon event, a chunk.
enum Call {
    Binding(String, bool),
    Event(String, Vec<ScriptValue>),
    Chunk(String),
}

/// The frame's calls, in order. Upstream runs every script call with its call host lent, so a call into the engine
/// (a target, a cast, mouselook) applies inside the verb; the host holds most of the game's state and cannot share a
/// system with [`drive_pad`]'s window, mouse and keys, so the pad queues its calls and [`run_calls`] makes them
/// right after, in the same frame. Reads (the addon's mode, a resolve, a snap) stay in place.
static CALLS: std::sync::Mutex<Vec<Call>> = std::sync::Mutex::new(Vec::new());

fn queue(call: Call) {
    CALLS.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(call);
}

/// `value` as a Lua literal.
fn lua_value(value: &ScriptValue) -> String {
    match value {
        ScriptValue::Nil => "nil".to_string(),
        ScriptValue::Bool(b) => b.to_string(),
        ScriptValue::Int(i) => i.to_string(),
        ScriptValue::Number(n) if n.is_finite() => format!("{n:?}"),
        ScriptValue::Number(_) => "0".to_string(),
        ScriptValue::Str(s) => map::lua_str(s),
    }
}

/// Make the frame's queued calls on the VM, with the call host lent.
fn run_calls(mut vm: benilla_app::ScriptVm) {
    let calls = std::mem::take(&mut *CALLS.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
    if calls.is_empty() {
        return;
    }
    // wow-nx: the first calls, logged, while the pad's events are traced on the Switch.
    static LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let hosted = vm.has_host();
    vm.run(|script| {
        let Some(script) = script else {
            warn!("pad: {} call(s) dropped: no VM", calls.len());
            return;
        };
        for call in calls {
            if LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 40 {
                let what = match &call {
                    Call::Binding(c, down) => format!("binding {c} {}", if *down { "down" } else { "up" }),
                    Call::Event(e, _) => format!("event {e}"),
                    Call::Chunk(lua) => format!("chunk {lua}"),
                };
                info!("pad: call {what} (host lent: {hosted})");
            }
            match call {
                Call::Binding(command, true) => match script.execute_binding(&command, true) {
                    Ok(true) => {}
                    Ok(false) => warn!("pad: no binding command {command}"),
                    Err(e) => warn!("pad: {command} down failed: {e}"),
                },
                // `Ok(false)` is a command that is not `runOnUp`: nothing to run.
                Call::Binding(command, false) => {
                    if let Err(e) = script.execute_binding(&command, false) {
                        warn!("pad: {command} up failed: {e}");
                    }
                }
                // wow-nx: the addon's own events reach it through BenillaPad_Event (its Core.lua): 1.12's
                // RegisterEvent takes only the names in its event table, so a fired BENILLAPAD_* event reached no
                // frame, and the addon never heard the pad connect or its menus a press.
                Call::Event(event, args) => {
                    let mut chunk = format!("if BenillaPad_Event then BenillaPad_Event({}", map::lua_str(&event));
                    for arg in &args {
                        chunk.push_str(", ");
                        chunk.push_str(&lua_value(arg));
                    }
                    chunk.push_str(") end");
                    if let Err(e) = script.run(&chunk) {
                        warn!("pad: {event} failed: {e}");
                    }
                }
                Call::Chunk(lua) => {
                    if let Err(e) = script.run(&lua) {
                        warn!("pad: {lua} failed: {e}");
                    }
                }
            }
        }
    });
}

/// Tuning. The addon's settings apply once it loads; an environment variable wins over both.
#[derive(Resource, Clone, Copy, Debug)]
pub struct PadSettings {
    /// Stick deflection below which a stick is at rest (`PAD_DEADZONE`, 0..1).
    pub deadzone: f32,
    /// Camera turn at full deflection, radians per second (`PAD_LOOK_SPEED`).
    pub look_speed: f32,
    /// Stick up looks down (`PAD_INVERT_Y=1`).
    pub invert_y: bool,
    /// LB or RB held turns the right stick's up/down into zoom.
    pub shoulder_zoom: bool,
    /// The pad cursor's speed at full tilt, logical pixels a second.
    pub cursor_speed: f32,
    /// Interact's loot window takes everything at once.
    pub loot_all: bool,
    /// Which of the three an environment variable pinned.
    pinned: [bool; 3],
}

impl Default for PadSettings {
    fn default() -> Self {
        Self {
            deadzone: 0.25,
            look_speed: 3.0,
            invert_y: false,
            shoulder_zoom: true,
            cursor_speed: 1100.0,
            loot_all: true,
            pinned: [false; 3],
        }
    }
}

impl PadSettings {
    fn from_env() -> Self {
        let mut s = Self::default();
        let num = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok());
        if let Some(v) = num("PAD_DEADZONE") {
            s.deadzone = v.clamp(0.0, 0.95);
            s.pinned[0] = true;
        }
        if let Some(v) = num("PAD_LOOK_SPEED") {
            s.look_speed = v.max(0.0);
            s.pinned[1] = true;
        }
        if let Ok(v) = std::env::var("PAD_INVERT_Y") {
            s.invert_y = v == "1";
            s.pinned[2] = true;
        }
        s
    }

    /// The addon's values, less what the environment pinned.
    fn apply_addon(
        &mut self,
        deadzone: f32,
        look_speed: f32,
        invert_y: bool,
        zoom: bool,
        cursor_speed: f32,
        loot_all: bool,
    ) {
        self.loot_all = loot_all;
        self.shoulder_zoom = zoom;
        self.cursor_speed = cursor_speed.max(50.0);
        if !self.pinned[0] {
            self.deadzone = deadzone.clamp(0.0, 0.95);
        }
        if !self.pinned[1] {
            self.look_speed = look_speed.max(0.0);
        }
        if !self.pinned[2] {
            self.invert_y = invert_y;
        }
    }
}

/// benilla's mouse-look rate at default sensitivity, radians per mouse unit
/// (`player::camera::LOOK_SENSITIVITY`), to turn a stick's rad/s into motion units.
const LOOK_RAD_PER_UNIT: f32 = 0.003;

/// Camera zoom at full deflection while LB or RB is held, yards per second.
const ZOOM_SPEED: f32 = 20.0;

/// What a held button's press ran, so its release ends exactly that.
#[derive(Clone, Debug)]
enum Latched {
    /// A binding command, released with its up half.
    Command(String),
    /// A native command or nothing: no release.
    Done,
    /// A modal press, released as `BENILLAPAD_NAV(button, false)`.
    Nav,
    /// The pad cursor's click, released with the mouse button.
    Mouse(MouseButton),
}

/// The UI's height in units: FrameXML lays out a 768-high screen, so an absolute position (UI
/// units times the effective scale) is `window height / 768` logical pixels a unit.
const UI_HEIGHT: f32 = 768.0;

/// What is held on our side, so each command gets exactly one down and one up.
#[derive(Resource, Default)]
struct PadState {
    buttons: Vec<(GamepadButton, &'static str, Latched)>,
    /// Which of [`map::MOVES`] is pressed.
    moves: [bool; 4],
    /// We hold mouse-look.
    looking: bool,
    /// The VM session the latches belong to; a new VM drops them, as benilla drops its own.
    session: Option<u64>,
    /// The pad this session's addon was told about.
    announced: Option<Entity>,
    layer: &'static str,
    settings_gen: Option<u32>,
    /// A resolver error, logged once per session.
    resolve_failed: bool,
    /// Last frame was in the cursor mode.
    cursor: bool,
    /// The last target we had, which a soft Interact loots first: 1.12 clears the selection when
    /// the target dies.
    last_target: Option<u64>,
    /// An Interact on a target the object mirror had not streamed yet, retried until this
    /// `Time::elapsed_secs`.
    interact_retry: Option<f32>,
    /// We hold Shift for an Interact's loot-all (1.12's auto-loot is the Shift held as the loot
    /// window opens), until this `Time::elapsed_secs` or the window shows.
    loot_shift: Option<f32>,
}

/// How long an Interact waits for its target to be streamed, and how long its Shift is held for
/// the loot window, in seconds.
const INTERACT_RETRY_SECS: f32 = 0.3;
const LOOT_SHIFT_SECS: f32 = 1.5;

// wow-nx: the three queue their call for [`run_calls`]; `_script` stays so the callers read as before.
fn fire(_script: &mut UiScript, event: &str, args: Vec<ScriptValue>) {
    queue(Call::Event(event.to_owned(), args));
}

fn press(_script: &UiScript, command: &str) {
    queue(Call::Binding(command.to_owned(), true));
}

fn release(_script: &UiScript, command: &str) {
    queue(Call::Binding(command.to_owned(), false));
}

fn set_look(_script: &UiScript, state: &mut PadState, on: bool) {
    if state.looking == on {
        return;
    }
    let lua = if on {
        "MouselookStart()"
    } else {
        "MouselookStop()"
    };
    queue(Call::Chunk(lua.to_owned()));
    state.looking = on;
}

/// The command a press runs: the addon's answer, or the stock fallback without the addon.
fn resolve(
    script: &UiScript,
    state: &mut PadState,
    addon: bool,
    button: &str,
    layer: &str,
) -> Option<String> {
    if !addon {
        return map::fallback_command(button, layer).map(str::to_string);
    }
    let chunk = format!(
        "return BenillaPad_Resolve({}, {})",
        map::lua_str(button),
        map::lua_str(layer)
    );
    match script.eval::<Option<String>>(&chunk) {
        Ok(command) => command,
        Err(e) => {
            if !std::mem::replace(&mut state.resolve_failed, true) {
                warn!("pad: BenillaPad_Resolve failed: {e}");
            }
            None
        }
    }
}

/// The whole gamepad, once a frame. A focused EditBox blocks presses, never releases.
fn drive_pad(
    script: Option<NonSendMut<UiScript>>,
    pads: Query<(Entity, &Gamepad)>,
    mut settings: ResMut<PadSettings>,
    time: Res<Time>,
    mut motion: ResMut<AccumulatedMouseMotion>,
    mut state: ResMut<PadState>,
    mut interacting: interact::Interacting,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut key_events: MessageWriter<KeyboardInput>,
    mut glue: Local<Vec<(GamepadButton, GlueInput)>>,
) {
    // Before the world (login, realm list, character select) the VM is the boot one, with no
    // stock UI loaded (`UIParent` is FrameXML's): nothing of the world's survives, and the pad
    // drives those screens' own keys and the cursor.
    // One Lua call a frame answers it, with the addon's mode and settings generation.
    let (in_world, mode, gen) = script
        .as_ref()
        .and_then(|s| {
            s.eval::<(bool, Option<String>, Option<u32>)>(
                "if BenillaPad_Mode then return UIParent ~= nil, BenillaPad_Mode() end \
                 return UIParent ~= nil",
            )
            .ok()
        })
        .unwrap_or((false, None, None));
    let Some(mut script) = script.filter(|_| in_world) else {
        if state.loot_shift.is_some() {
            keys.release(KeyCode::ShiftLeft);
        }
        *state = PadState::default();
        drive_glue(
            pads.iter().next().map(|(_, g)| g),
            &settings,
            &time,
            windows.single_mut().ok().as_deref_mut(),
            &mut mouse,
            &mut keys,
            &mut key_events,
            &mut glue,
        );
        return;
    };
    // Entering the world with a glue key still held: let go of it.
    for (_, input) in glue.drain(..) {
        release_glue(input, &mut mouse, &mut keys, &mut key_events);
    }
    let session = script.session();
    if state.session != Some(session) {
        if state.loot_shift.is_some() {
            keys.release(KeyCode::ShiftLeft);
        }
        *state = PadState {
            session: Some(session),
            layer: "bare",
            ..default()
        };
    }
    // The first connected pad; none releases everything we hold.
    let pad = pads.iter().next();

    // ── Connection ──
    match pad {
        Some((entity, gamepad)) if state.announced != Some(entity) => {
            let style = map::style(gamepad.vendor_id());
            fire(
                &mut script,
                "BENILLAPAD_CONNECTED",
                vec![ScriptValue::Str(style.to_string())],
            );
            info!("pad: controller connected ({style})");
            state.announced = Some(entity);
        }
        None if state.announced.is_some() => {
            fire(&mut script, "BENILLAPAD_DISCONNECTED", Vec::new());
            info!("pad: controller disconnected");
            state.announced = None;
        }
        _ => {}
    }
    let pad = pad.map(|(_, g)| g);

    // ── The addon: its mode and settings ──
    let addon = mode.is_some();
    let nintendo = pad.is_some_and(|g| map::style(g.vendor_id()) == "nintendo");
    let mode = mode.unwrap_or_else(|| "world".to_string());
    let cursor = mode == "cursor";
    // The addon's own windows (menu, wheel, keybinds) take every button; the cursor mode plays on.
    let world = mode == "world" || cursor;
    if gen.is_some() && gen != state.settings_gen {
        if let Ok((deadzone, look, invert, zoom, speed, loot_all)) =
            script.eval::<(f32, f32, bool, Option<bool>, Option<f32>, Option<bool>)>(
                "return BenillaPad_Settings()",
            )
        {
            settings.apply_addon(
                deadzone,
                look,
                invert,
                zoom.unwrap_or(true),
                speed.unwrap_or(1100.0),
                loot_all.unwrap_or(true),
            );
        }
        state.settings_gen = gen;
    }
    let mut window = windows.single_mut().ok();
    // Entering the cursor mode: onto an open popup's first button, if there is one.
    if cursor && !state.cursor {
        if let Some(w) = window.as_deref_mut() {
            snap(&script, w, "enter");
        }
    }
    state.cursor = cursor;
    let typing = script.has_keyboard_focus();

    // ── Interact's upkeep ──
    let now = time.elapsed_secs();
    if let Some(target) = interacting.target() {
        state.last_target = Some(target);
    }
    if let Some(until) = state.interact_retry {
        if interacting.with_target() || now > until {
            state.interact_retry = None;
        }
    }
    if let Some(until) = state.loot_shift {
        let open = script
            .eval::<bool>("return LootFrame ~= nil and LootFrame:IsVisible() ~= nil")
            .unwrap_or(false);
        if open || now > until {
            keys.release(KeyCode::ShiftLeft);
            state.loot_shift = None;
        }
    }

    // ── Layer ──
    let layer = pad.map_or("bare", |p| {
        map::layer(
            p.pressed(GamepadButton::LeftTrigger2),
            p.pressed(GamepadButton::RightTrigger2),
        )
    });
    if layer != state.layer {
        state.layer = layer;
        fire(
            &mut script,
            "BENILLAPAD_LAYER",
            vec![ScriptValue::Str(layer.to_string())],
        );
    }

    // ── Buttons ── releases first, each to what its press ran.
    let mut i = 0;
    while i < state.buttons.len() {
        let (button, name, _) = state.buttons[i];
        if pad.is_none_or(|p| !p.pressed(button)) {
            let (_, _, latched) = state.buttons.swap_remove(i);
            match latched {
                Latched::Command(command) => release(&script, &command),
                Latched::Nav => fire(
                    &mut script,
                    "BENILLAPAD_NAV",
                    vec![ScriptValue::Str(name.to_string()), ScriptValue::Bool(false)],
                ),
                Latched::Mouse(b) => mouse.release(b),
                Latched::Done => {}
            }
        } else {
            i += 1;
        }
    }
    if let Some(pad) = pad {
        for (button, name) in map::BUTTONS {
            if pad.just_released(button) {
                fire(
                    &mut script,
                    "BENILLAPAD_BUTTON",
                    vec![ScriptValue::Str(name.to_string()), ScriptValue::Bool(false)],
                );
            }
            if !pad.just_pressed(button) {
                continue;
            }
            fire(
                &mut script,
                "BENILLAPAD_BUTTON",
                vec![ScriptValue::Str(name.to_string()), ScriptValue::Bool(true)],
            );
            // wow-nx: the windows and the cursor take a Nintendo pad's printed letters, so A confirms there.
            let letter = if nintendo { map::letter(name) } else { name };
            if !world {
                fire(
                    &mut script,
                    "BENILLAPAD_NAV",
                    vec![ScriptValue::Str(letter.to_string()), ScriptValue::Bool(true)],
                );
                state.buttons.push((button, letter, Latched::Nav));
                continue;
            }
            if cursor {
                if let Some(latched) = cursor_press(
                    &script,
                    letter,
                    window.as_deref_mut(),
                    &mut mouse,
                    &mut scroll,
                ) {
                    state.buttons.push((button, name, latched));
                    continue;
                }
            }
            if typing {
                continue;
            }
            let latched = match resolve(&script, &mut state, addon, name, layer) {
                Some(command) if command == "@INTERACT" => {
                    let now = time.elapsed_secs();
                    // The UI knows at once whether there is a target; the mirror's own record of
                    // it (`UNIT_FIELD_TARGET`) follows the server's echo, so a target not there
                    // yet is retried for a moment.
                    let has_target = script
                        .eval::<bool>("return UnitExists('target') ~= nil")
                        .unwrap_or(false);
                    if has_target {
                        if !interacting.with_target() {
                            state.interact_retry = Some(now + INTERACT_RETRY_SECS);
                        }
                    } else {
                        interacting.with_soft_target(state.last_target);
                    }
                    // Loot everything: hold Shift over the loot window's opening, unless
                    // benilla's own auto-loot is on, where Shift would turn it off.
                    let auto = script
                        .eval::<bool>("return GetCVar('autoLootDefault') == '1'")
                        .unwrap_or(false);
                    if settings.loot_all && !auto && state.loot_shift.is_none() {
                        keys.press(KeyCode::ShiftLeft);
                        state.loot_shift = Some(now + LOOT_SHIFT_SECS);
                    }
                    Latched::Done
                }
                Some(command) if command.starts_with('@') => {
                    warn!("pad: unknown native command {command}");
                    Latched::Done
                }
                Some(command) => {
                    press(&script, &command);
                    Latched::Command(command)
                }
                None => Latched::Done,
            };
            state.buttons.push((button, name, latched));
        }
    }

    let left = pad.map_or(Vec2::ZERO, Gamepad::left_stick);
    let right = pad.map_or(Vec2::ZERO, Gamepad::right_stick);

    // ── Outside the world mode the sticks belong to the addon's window ──
    if !world {
        release_moves(&script, &mut state);
        set_look(&script, &mut state, false);
        fire(
            &mut script,
            "BENILLAPAD_STICK",
            vec![
                ScriptValue::Number(left.x as f64),
                ScriptValue::Number(left.y as f64),
                ScriptValue::Number(right.x as f64),
                ScriptValue::Number(right.y as f64),
            ],
        );
        return;
    }

    // ── Left stick ──
    let want = map::move_commands(left, settings.deadzone);
    for (k, command) in map::MOVES.iter().enumerate() {
        if state.moves[k] && !want[k] {
            release(&script, command);
            state.moves[k] = false;
        } else if !state.moves[k] && want[k] && !typing {
            press(&script, command);
            state.moves[k] = true;
        }
    }

    // ── Right stick ── in the cursor mode, the mouse cursor.
    let right_live = right.length() >= settings.deadzone;
    if cursor {
        set_look(&script, &mut state, false);
        if let (Some(w), true) = (window.as_deref_mut(), right_live) {
            move_cursor(w, right, &settings, &time);
        }
        return;
    }

    // ── Right stick ── with LB or RB held, up/down zooms; otherwise mouse-look, held while
    // either stick is out of rest so the body faces the camera.
    let left_live = left.length() >= settings.deadzone;
    let shoulder = pad.is_some_and(|p| {
        p.pressed(GamepadButton::LeftTrigger) || p.pressed(GamepadButton::RightTrigger)
    });
    let zooming = settings.shoulder_zoom && shoulder && right_live && right.y.abs() > right.x.abs();
    if zooming && !typing {
        let yards = ZOOM_SPEED * time.delta_secs() * deflection(right.length(), settings.deadzone);
        let verb = if right.y > 0.0 {
            "CameraZoomIn"
        } else {
            "CameraZoomOut"
        };
        queue(Call::Chunk(format!("{verb}({yards})")));
    }
    set_look(
        &script,
        &mut state,
        (right_live || left_live) && !typing && !zooming,
    );
    if state.looking && right_live && !zooming {
        let mag = deflection(right.length(), settings.deadzone);
        let dir = right.normalize();
        let units = settings.look_speed * time.delta_secs() / LOOK_RAD_PER_UNIT * mag;
        // Mouse +y is down, which pitches the view down; stick +y is up.
        let y = if settings.invert_y { dir.y } else { -dir.y };
        motion.delta += Vec2::new(dir.x, y) * units;
    }
}

/// What a pad button holds down before the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GlueInput {
    Key(KeyCode),
    Mouse(MouseButton),
}

/// The pad before the world: A is Enter (log in, enter the world, a dialog's Okay), B is Escape
/// (back, Cancel), the D-pad is the arrow keys (the character and realm lists), Y is Tab (the
/// next box), X clicks, and the right stick moves the cursor. These are the keys the screens
/// read (`login`, `realm_select`, `char_select`).
fn glue_input(button: GamepadButton, nintendo: bool) -> Option<GlueInput> {
    // A Nintendo pad confirms with A, its right face button, and backs out with B, its bottom one.
    let (confirm, back) = if nintendo {
        (GamepadButton::East, GamepadButton::South)
    } else {
        (GamepadButton::South, GamepadButton::East)
    };
    Some(match button {
        b if b == confirm => GlueInput::Key(KeyCode::Enter),
        b if b == back => GlueInput::Key(KeyCode::Escape),
        GamepadButton::DPadUp => GlueInput::Key(KeyCode::ArrowUp),
        GamepadButton::DPadDown => GlueInput::Key(KeyCode::ArrowDown),
        GamepadButton::DPadLeft => GlueInput::Key(KeyCode::ArrowLeft),
        GamepadButton::DPadRight => GlueInput::Key(KeyCode::ArrowRight),
        GamepadButton::North => GlueInput::Key(KeyCode::Tab),
        GamepadButton::West => GlueInput::Mouse(MouseButton::Left),
        _ => return None,
    })
}

/// A key edge as the keyboard would send it: the button plane for `just_pressed` readers and the
/// message for the ones that read events (Tab on the login screen).
fn glue_key(
    key: KeyCode,
    down: bool,
    keys: &mut ButtonInput<KeyCode>,
    events: &mut MessageWriter<KeyboardInput>,
) {
    if down {
        keys.press(key);
    } else {
        keys.release(key);
    }
    events.write(KeyboardInput {
        key_code: key,
        logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
        state: if down {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        },
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
}

fn release_glue(
    input: GlueInput,
    mouse: &mut ButtonInput<MouseButton>,
    keys: &mut ButtonInput<KeyCode>,
    events: &mut MessageWriter<KeyboardInput>,
) {
    match input {
        GlueInput::Key(k) => glue_key(k, false, keys, events),
        GlueInput::Mouse(b) => mouse.release(b),
    }
}

/// The pad on the screens before the world ([`glue_input`]).
fn drive_glue(
    pad: Option<&Gamepad>,
    settings: &PadSettings,
    time: &Time,
    window: Option<&mut Window>,
    mouse: &mut ButtonInput<MouseButton>,
    keys: &mut ButtonInput<KeyCode>,
    events: &mut MessageWriter<KeyboardInput>,
    held: &mut Vec<(GamepadButton, GlueInput)>,
) {
    let mut i = 0;
    while i < held.len() {
        let (button, input) = held[i];
        if pad.is_none_or(|p| !p.pressed(button)) {
            release_glue(input, mouse, keys, events);
            held.swap_remove(i);
        } else {
            i += 1;
        }
    }
    let Some(pad) = pad else {
        return;
    };
    let nintendo = map::style(pad.vendor_id()) == "nintendo";
    for (button, _) in map::BUTTONS {
        if !pad.just_pressed(button) {
            continue;
        }
        if let Some(input) = glue_input(button, nintendo) {
            match input {
                GlueInput::Key(k) => glue_key(k, true, keys, events),
                GlueInput::Mouse(b) => mouse.press(b),
            }
            held.push((button, input));
        }
    }
    let right = pad.right_stick();
    if let (Some(w), true) = (window, right.length() >= settings.deadzone) {
        move_cursor(w, right, settings, time);
    }
}

/// The right stick as the mouse cursor: faster the further it is pushed.
fn move_cursor(w: &mut Window, right: Vec2, settings: &PadSettings, time: &Time) {
    let speed = settings.cursor_speed * deflection(right.length(), settings.deadzone).powf(1.5);
    let at = w
        .cursor_position()
        .unwrap_or(Vec2::new(w.width() / 2.0, w.height() / 2.0));
    // Window y runs down; stick y runs up.
    let to = at + Vec2::new(right.x, -right.y).normalize() * speed * time.delta_secs();
    let to = to.clamp(Vec2::ZERO, Vec2::new(w.width() - 1.0, w.height() - 1.0));
    w.set_cursor_position(Some(to));
}

/// A press in the cursor mode: A and X click (left, right), B runs the Escape ladder, the D-pad
/// snaps to the nearest button, LB / RB scroll. `None` for a button the cursor leaves alone
/// (Start, Select), which then resolves as in the world.
fn cursor_press(
    script: &UiScript,
    name: &str,
    window: Option<&mut Window>,
    mouse: &mut ButtonInput<MouseButton>,
    scroll: &mut AccumulatedMouseScroll,
) -> Option<Latched> {
    match name {
        "A" => {
            mouse.press(MouseButton::Left);
            Some(Latched::Mouse(MouseButton::Left))
        }
        "X" => {
            mouse.press(MouseButton::Right);
            Some(Latched::Mouse(MouseButton::Right))
        }
        "B" => {
            press(script, "TOGGLEGAMEMENU");
            Some(Latched::Command("TOGGLEGAMEMENU".to_string()))
        }
        "DUP" | "DDOWN" | "DLEFT" | "DRIGHT" => {
            if let Some(w) = window {
                snap(script, w, name);
            }
            Some(Latched::Done)
        }
        "LB" | "RB" => {
            scroll.unit = MouseScrollUnit::Line;
            scroll.delta.y += if name == "LB" { 1.0 } else { -1.0 };
            Some(Latched::Done)
        }
        "Y" | "L3" | "R3" => Some(Latched::Done),
        _ => None,
    }
}

/// Move the cursor to the button `BenillaPad_Snap(dir, x, y)` names, if it names one.
fn snap(script: &UiScript, window: &mut Window, dir: &str) {
    let h = window.height();
    let px_per_unit = h / UI_HEIGHT;
    let at = window
        .cursor_position()
        .unwrap_or(Vec2::new(window.width() / 2.0, h / 2.0));
    let (ax, ay) = (at.x / px_per_unit, (h - at.y) / px_per_unit);
    let chunk = format!(
        "if BenillaPad_Snap then return BenillaPad_Snap({}, {ax}, {ay}) end",
        map::lua_str(dir)
    );
    match script.eval::<(Option<f32>, Option<f32>)>(&chunk) {
        Ok((Some(x), Some(y))) => {
            window.set_cursor_position(Some(Vec2::new(x * px_per_unit, h - y * px_per_unit)));
        }
        Ok(_) => {}
        Err(e) => warn!("pad: BenillaPad_Snap failed: {e}"),
    }
}

/// A stick's length rescaled past the deadzone, so motion starts from zero at its edge.
fn deflection(length: f32, deadzone: f32) -> f32 {
    ((length - deadzone) / (1.0 - deadzone).max(f32::EPSILON)).clamp(0.0, 1.0)
}

fn release_moves(script: &UiScript, state: &mut PadState) {
    for (k, command) in map::MOVES.iter().enumerate() {
        if std::mem::take(&mut state.moves[k]) {
            release(script, command);
        }
    }
}

#[cfg(test)]
mod tests;
