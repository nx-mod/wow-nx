//! The pure halves of the pad: button names, layers, the left stick's eight directions, and the
//! fallback map used when the BenillaPad addon is not loaded.

use bevy::prelude::*;

/// The buttons the pad routes, with the names the addon knows them by. The triggers are not here:
/// they only pick the layer.
pub const BUTTONS: [(GamepadButton, &str); 14] = [
    (GamepadButton::DPadUp, "DUP"),
    (GamepadButton::DPadRight, "DRIGHT"),
    (GamepadButton::DPadDown, "DDOWN"),
    (GamepadButton::DPadLeft, "DLEFT"),
    (GamepadButton::North, "Y"),
    (GamepadButton::East, "B"),
    (GamepadButton::South, "A"),
    (GamepadButton::West, "X"),
    (GamepadButton::LeftTrigger, "LB"),
    (GamepadButton::RightTrigger, "RB"),
    (GamepadButton::LeftThumb, "L3"),
    (GamepadButton::RightThumb, "R3"),
    (GamepadButton::Start, "START"),
    (GamepadButton::Select, "SELECT"),
];

/// The layer the triggers hold, by the addon's name.
pub fn layer(lt: bool, rt: bool) -> &'static str {
    match (lt, rt) {
        (false, false) => "bare",
        (true, false) => "lt",
        (false, true) => "rt",
        (true, true) => "ltrt",
    }
}

/// The face style a pad's USB vendor implies.
pub fn style(vendor: Option<u16>) -> &'static str {
    match vendor {
        Some(0x054c) => "playstation",
        Some(0x057e) => "nintendo",
        _ => "xbox",
    }
}

/// The four movement commands, in [`move_commands`]'s order.
pub const MOVES: [&str; 4] = ["MOVEFORWARD", "MOVEBACKWARD", "STRAFELEFT", "STRAFERIGHT"];

/// The left stick as [`MOVES`] held: 8 sectors of 45°, nothing inside the deadzone. 1.12's
/// movement is digital direction bits at one speed, so the stick's magnitude only gates.
pub fn move_commands(stick: Vec2, deadzone: f32) -> [bool; 4] {
    if stick.length() < deadzone.max(f32::EPSILON) {
        return [false; 4];
    }
    let d = stick.normalize();
    // sin 22.5°: a component past it is inside that direction's three sectors.
    const EDGE: f32 = 0.382_683_43;
    [d.y > EDGE, d.y < -EDGE, d.x < -EDGE, d.x > EDGE]
}

/// What a press runs without the addon: the stock commands, so the pad still plays.
pub fn fallback_command(button: &str, layer: &str) -> Option<&'static str> {
    match button {
        "LB" => return Some("TARGETNEARESTFRIEND"),
        "RB" => return Some("TARGETNEARESTENEMY"),
        "START" => return Some("TOGGLEGAMEMENU"),
        "L3" => return Some("TOGGLEAUTORUN"),
        _ => {}
    }
    let slot = match button {
        "DUP" => 0,
        "DRIGHT" => 1,
        "DDOWN" => 2,
        "DLEFT" => 3,
        "Y" => 4,
        "B" => 5,
        "A" => 6,
        "X" => 7,
        _ => return None,
    };
    const BARE: [Option<&str>; 8] = [
        Some("ACTIONBUTTON1"),
        Some("ACTIONBUTTON2"),
        Some("ACTIONBUTTON3"),
        Some("ACTIONBUTTON4"),
        None,
        None,
        Some("JUMP"),
        Some("ATTACKTARGET"),
    ];
    const LT: [&str; 8] = [
        "ACTIONBUTTON5",
        "ACTIONBUTTON6",
        "ACTIONBUTTON7",
        "ACTIONBUTTON8",
        "ACTIONBUTTON9",
        "ACTIONBUTTON10",
        "ACTIONBUTTON11",
        "ACTIONBUTTON12",
    ];
    const RT: [&str; 8] = [
        "MULTIACTIONBAR1BUTTON1",
        "MULTIACTIONBAR1BUTTON2",
        "MULTIACTIONBAR1BUTTON3",
        "MULTIACTIONBAR1BUTTON4",
        "MULTIACTIONBAR1BUTTON5",
        "MULTIACTIONBAR1BUTTON6",
        "MULTIACTIONBAR1BUTTON7",
        "MULTIACTIONBAR1BUTTON8",
    ];
    const BOTH: [&str; 8] = [
        "MULTIACTIONBAR2BUTTON1",
        "MULTIACTIONBAR2BUTTON2",
        "MULTIACTIONBAR2BUTTON3",
        "MULTIACTIONBAR2BUTTON4",
        "MULTIACTIONBAR2BUTTON5",
        "MULTIACTIONBAR2BUTTON6",
        "MULTIACTIONBAR2BUTTON7",
        "MULTIACTIONBAR2BUTTON8",
    ];
    match layer {
        "lt" => Some(LT[slot]),
        "rt" => Some(RT[slot]),
        "ltrt" => Some(BOTH[slot]),
        _ => BARE[slot],
    }
}

/// A Lua string literal for `s`, for the resolver's arguments (names are ASCII words).
pub fn lua_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stick_reads_as_eight_directions() {
        assert_eq!(
            move_commands(Vec2::new(0.0, 1.0), 0.25),
            [true, false, false, false]
        );
        assert_eq!(
            move_commands(Vec2::new(0.7, 0.7), 0.25),
            [true, false, false, true]
        );
        assert_eq!(
            move_commands(Vec2::new(-1.0, 0.1), 0.25),
            [false, false, true, false]
        );
        assert_eq!(move_commands(Vec2::new(0.1, 0.1), 0.25), [false; 4]);
    }

    #[test]
    fn the_triggers_pick_the_layer() {
        assert_eq!(layer(false, false), "bare");
        assert_eq!(layer(true, false), "lt");
        assert_eq!(layer(false, true), "rt");
        assert_eq!(layer(true, true), "ltrt");
        assert_eq!(fallback_command("DUP", "lt"), Some("ACTIONBUTTON5"));
        assert_eq!(fallback_command("A", "bare"), Some("JUMP"));
        assert_eq!(fallback_command("RB", "ltrt"), Some("TARGETNEARESTENEMY"));
        assert_eq!(fallback_command("Y", "bare"), None);
    }

    #[test]
    fn a_lua_literal_keeps_only_word_characters() {
        assert_eq!(lua_str("DUP"), "\"DUP\"");
        assert_eq!(lua_str("a\") os.exit(\""), "\"aosexit\"");
    }
}
