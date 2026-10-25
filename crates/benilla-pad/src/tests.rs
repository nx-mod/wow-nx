//! The addon's Lua against benilla's own VM: it must parse under the 1.12 grammar, and the
//! resolver must answer the default layout.

use benilla_ui::script::UiScript;
use include_dir::{include_dir, Dir};

static ADDON: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/addon/BenillaPad");

fn source(name: &str) -> &'static str {
    std::str::from_utf8(ADDON.get_file(name).expect("embedded").contents()).expect("utf-8")
}

#[test]
fn every_lua_file_parses_under_the_1_12_grammar() {
    let script = UiScript::new().expect("a bare VM");
    for file in ADDON.files() {
        let name = file.path().to_string_lossy().to_string();
        if !name.ends_with(".lua") {
            continue;
        }
        let text = std::str::from_utf8(file.contents()).expect("utf-8");
        if let Err(e) = script.lua().load(text).set_name(&name).into_function() {
            panic!("{name} does not parse: {e}");
        }
    }
}

/// A bare VM with Core and Actions loaded; the stock FrameXML's `SlashCmdList`, which the core
/// load defines before any addon, stands in as an empty table.
fn loaded() -> UiScript {
    let script = UiScript::new().expect("a bare VM");
    script.run("SlashCmdList = {}").expect("stand-in");
    for name in ["Core.lua", "Actions.lua"] {
        script
            .run_chunk_named(source(name).as_bytes(), name)
            .unwrap_or_else(|e| panic!("{name} failed: {e}"));
    }
    script
}

/// Core and Actions run in a bare VM, and the resolver answers the default layout.
#[test]
fn the_resolver_answers_the_default_layout() {
    let script = loaded();
    let ask = |b: &str, l: &str| {
        script
            .eval::<Option<String>>(&format!("return BenillaPad_Resolve('{b}', '{l}')"))
            .expect("resolve answers")
    };
    assert_eq!(ask("DUP", "bare").as_deref(), Some("BENILLAPAD_ACTION1"));
    assert_eq!(ask("A", "bare").as_deref(), Some("JUMP"));
    assert_eq!(ask("X", "bare").as_deref(), Some("@INTERACT"));
    assert_eq!(ask("RB", "bare").as_deref(), Some("BENILLAPAD_TARGETENEMY"));
    assert_eq!(ask("A", "lt").as_deref(), Some("BENILLAPAD_ACTION19"));
    assert_eq!(ask("DLEFT", "rt").as_deref(), Some("BENILLAPAD_ACTION28"));
    assert_eq!(ask("R3", "ltrt").as_deref(), Some("BENILLAPAD_ACTION48"));
    assert_eq!(ask("START", "lt").as_deref(), Some("BENILLAPAD_WHEEL"));
    assert_eq!(ask("A", "ltrt").as_deref(), Some("BENILLAPAD_WHEEL"));
    assert_eq!(ask("X", "ltrt").as_deref(), Some("BENILLAPAD_CONSUMABLES"));
    assert_eq!(ask("B", "ltrt").as_deref(), Some("BENILLAPAD_QUESTITEM"));
    assert_eq!(ask("Y", "ltrt").as_deref(), Some("BENILLAPAD_BOTWHEEL"));
    assert_eq!(ask("DUP", "ltrt").as_deref(), Some("BENILLAPAD_ACTION37"));
    let (mode, gen) = script
        .eval::<(String, u32)>("return BenillaPad_Mode()")
        .expect("mode answers");
    assert_eq!((mode.as_str(), gen), ("world", 1));
    let (dz, look, invert, zoom) = script
        .eval::<(f32, f32, bool, bool)>("return BenillaPad_Settings()")
        .expect("settings answer");
    assert_eq!((dz, look, invert, zoom), (0.25, 3.0, false, true));
}

/// Every command the resolver can return is a row of the addon's Bindings.xml or a stock one.
#[test]
fn every_addon_command_has_a_binding_row() {
    let bindings = source("Bindings.xml");
    let script = loaded();
    let commands: Vec<String> = script
        .eval(
            "local out = {} for _, a in pairs(BenillaPad.Actions) do \
             table.insert(out, a.command) end return out",
        )
        .expect("actions");
    for command in commands {
        if command.starts_with("BENILLAPAD_") {
            assert!(
                bindings.contains(&format!("name=\"{command}\"")),
                "{command} has no Bindings.xml row"
            );
        }
    }
    for slot in 1..=48 {
        assert!(bindings.contains(&format!("name=\"BENILLAPAD_ACTION{slot}\"")));
    }
}

/// Every event the Rust side fires reaches the addon's listeners through `fire_event`, the path
/// the game takes (a direct call would skip the frame's `RegisterEvent`).
#[test]
fn every_fired_event_reaches_the_listeners() {
    use benilla_ui::script::ScriptValue;
    let mut script = loaded();
    script
        .run("BenillaPadSeen = {} BenillaPad.Listen(function(e) BenillaPadSeen[e] = true end)")
        .expect("listen");
    let s = |v: &str| ScriptValue::Str(v.to_string());
    let fired = [
        ("BENILLAPAD_CONNECTED", vec![s("xbox")]),
        ("BENILLAPAD_LAYER", vec![s("lt")]),
        ("BENILLAPAD_BUTTON", vec![s("A"), ScriptValue::Bool(true)]),
        ("BENILLAPAD_NAV", vec![s("A"), ScriptValue::Bool(true)]),
        ("BENILLAPAD_STICK", vec![ScriptValue::Number(0.5); 4]),
        ("BENILLAPAD_DISCONNECTED", Vec::new()),
    ];
    for (event, args) in fired {
        script.fire_event(event, args);
        assert!(
            script
                .eval::<bool>(&format!("return BenillaPadSeen['{event}'] == true"))
                .expect("seen answers"),
            "{event} never reached the addon"
        );
    }
}

/// Every art file the Lua names (`ART .. "Name"`) is shipped, so no button draws blank.
#[test]
fn every_art_reference_is_shipped() {
    let mut checked = 0;
    for file in ADDON.files() {
        let name = file.path().to_string_lossy().to_string();
        if !name.ends_with(".lua") {
            continue;
        }
        let text = std::str::from_utf8(file.contents()).expect("utf-8");
        for (at, _) in text.match_indices("ART .. \"") {
            let rest = &text[at + "ART .. \"".len()..];
            let art = &rest[..rest.find('"').expect("a closing quote")];
            assert!(
                ADDON.get_file(format!("Art/{art}.tga")).is_some(),
                "{name} names Art/{art}.tga, which is not shipped"
            );
            checked += 1;
        }
    }
    assert!(checked > 30, "only {checked} art references found");
    // The bot wheel's table names its art without the prefix.
    let wheel = source("Wheel.lua");
    let table = &wheel[wheel.find("local BOT_ICONS = {").expect("the table")..];
    let table = &table[..table.find('}').expect("its end")];
    for (at, _) in table.match_indices("= \"") {
        let rest = &table[at + 3..];
        let art = &rest[..rest.find('"').expect("a closing quote")];
        assert!(
            ADDON.get_file(format!("Art/{art}.tga")).is_some(),
            "the bot wheel names Art/{art}.tga, which is not shipped"
        );
    }
}

/// The boot VM of the login and character screens has no stock UI, which is how the pad tells
/// those screens from the world.
#[test]
fn a_bare_vm_reads_as_before_the_world() {
    let script = UiScript::new().expect("a bare VM");
    assert!(!script
        .eval::<bool>("return UIParent ~= nil")
        .expect("the check answers"));
}
