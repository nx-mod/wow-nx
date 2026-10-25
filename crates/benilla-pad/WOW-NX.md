# benilla-pad in wow-nx

From [lucasvallejodev/benilla-gamepad](https://github.com/lucasvallejodev/benilla-gamepad) (MIT/Apache-2.0), at its
`main` 0e1db694, with these changes for the Switch and for today's upstream:

- **Calls with the call host.** Upstream now refuses an engine call made with no call host lent, so every press,
  target and mouselook failed. The pad queues its calls, and `run_calls` makes them through `benilla_app::ScriptVm`
  right after `drive_pad`, in the same frame. Reads (the addon's mode, a resolve, a snap) stay in place.
- **Events through `BenillaPad_Event`.** 1.12's `RegisterEvent` takes only the names in its event table, and benilla
  follows it, so the addon's `BENILLAPAD_*` events registered nothing (no gamepad bar, menus deaf to the pad). The pad
  calls `BenillaPad_Event(name, ...)`, which runs the addon's handler with the globals an `OnEvent` reads.
- **No gilrs on the Switch.** `PadPlugin` adds `GilrsPlugin` elsewhere only; on the Switch the Joy-Con come from
  `benilla-app/src/switch.rs` as Bevy's gamepad, by place (B, the bottom, is South), with Nintendo's vendor id.
- **Nintendo pads.** Button symbols gain "Nintendo" (Auto picks it): the bar and keybind screen print each button's
  own letter, and L/R, ZL/ZR, LS/RS and +/− (`P.Name`, `P.Hint`). The windows, the cursor and the login screens take
  the printed letters (`map::letter`), so A confirms and B backs out there. In the world the bare face buttons A, Y
  and X play action slots 1–3 and the D-pad 4–7; B jumps; the right stick pressed interacts.
- **Tracing.** The first 40 calls are logged with whether a host was lent (to be removed once the pad is settled on
  the Switch).

Still in Xbox names: the controller menu's descriptions (`Menu.lua`).
