-- BenillaPad: the face of benilla-pad's controller support. The Rust side reads the gamepad and
-- asks this file what each press does (BenillaPad_Resolve), then runs the answer as a binding
-- command, the path a key takes. Lua 5.0 grammar and the 1.12 API only: no `#`, `%`, `...`,
-- string methods or `self` in handlers.

BenillaPad = {}
local P = BenillaPad

P.ART = "Interface\\AddOns\\BenillaPad\\Art\\"

-- Button names as the Rust side sends them, in the HUD's slot order.
P.BUTTONS = { "DUP", "DRIGHT", "DDOWN", "DLEFT", "Y", "B", "A", "X", "LB", "RB", "L3", "R3" }
P.LAYERS = { "bare", "lt", "rt", "ltrt" }
P.LAYER_LABEL = { bare = "", lt = "LT", rt = "RT", ltrt = "LT + RT" }

-- The highest action slot a pad button may hold: the Bindings.xml BENILLAPAD_ACTION rows.
P.MAX_SLOT = 48

-- Each layer owns twelve action slots, one per button, in P.BUTTONS order. Slots 1-12 are paged
-- by stance and form (1.12's bonus bars), so the bare layer follows druid forms, warrior stances
-- and stealth by itself.
local SLOT_BASE = { bare = 0, lt = 12, rt = 24, ltrt = 36 }
local SLOT_INDEX = {}
for i = 1, table.getn(P.BUTTONS) do
    SLOT_INDEX[P.BUTTONS[i]] = i
end

-- Buttons that hold a game action until the player binds something else there.
local DEFAULT_ACTIONS = {
    bare = {
        Y = "inspect", B = "back", A = "jump", X = "interact",
        LB = "targetfriend", RB = "targetenemy", L3 = "autorun", R3 = "targetself",
    },
    ltrt = { A = "wheel", X = "consumables", B = "questitem", Y = "botwheel" },
}

-- Start and Select on every layer.
local SYSTEM_ACTIONS = { START = "wheel", SELECT = "menu" }

P.DEFAULTS = {
    castOnPress = true,
    deadzone = 0.25,
    lookSpeed = 3.0,
    invertY = false,
    shoulderZoom = true,
    -- A/B/X/Y with no trigger: their game actions, or their own action slots like the D-pad.
    faceSpells = false,
    scale = 1.0,
    offsetY = 170,
    -- "auto" follows the pad's vendor; else "xbox" or "playstation".
    symbols = "auto",
    alwaysShow = false,
    -- The stock action bars, bags and micro menu go while the gamepad bar shows.
    -- (A new key: an older build saved "hideStockBar" off by default.)
    hideStockBars = true,
    -- Stock windows take the pad as a mouse cursor; and how fast it moves, pixels a second.
    padCursor = true,
    cursorSpeed = 1100,
    -- The experience bar along the bottom of the screen, and its numbers.
    xpBar = true,
    xpText = true,
    -- Interact loots a whole corpse at once.
    interactLootAll = true,
    -- A row above the bar with the cooldowns of the layers not held.
    layerCooldowns = true,
}

local FACES = { A = true, B = true, X = true, Y = true }

P.layer = "bare"
P.held = {}
P.connected = false
P.style = "xbox"
P.mode = "world"
P.settingsGen = 1

function P.Print(msg)
    if DEFAULT_CHAT_FRAME then
        DEFAULT_CHAT_FRAME:AddMessage("|cff66bbffBenillaPad|r: " .. msg)
    end
end

function P.Setting(key)
    local db = BenillaPadDB
    if db and db.settings and db.settings[key] ~= nil then
        return db.settings[key]
    end
    return P.DEFAULTS[key]
end

function P.SetSetting(key, value)
    BenillaPadDB = BenillaPadDB or {}
    BenillaPadDB.settings = BenillaPadDB.settings or {}
    BenillaPadDB.settings[key] = value
    P.settingsGen = P.settingsGen + 1
    if P.Changed then
        P.Changed()
    end
end

-- The action slot a button owns on a layer; every button has one, whatever it is set to.
function P.OwnSlot(btn, layer)
    local index = SLOT_INDEX[btn]
    if not index or not SLOT_BASE[layer] then
        return nil
    end
    return SLOT_BASE[layer] + index
end

-- Whether a button can be set (Start and Select keep their jobs).
function P.Bindable(btn)
    return SLOT_INDEX[btn] ~= nil
end

-- The player's setting for a button on a layer: nil (the default), { action = id } or
-- { slot = true } (its own action slot, even where the default is a game action).
function P.Override(btn, layer)
    local char = BenillaPadCharDB
    return char and char.binds and char.binds[layer] and char.binds[layer][btn]
end

function P.SetOverride(btn, layer, value)
    BenillaPadCharDB = BenillaPadCharDB or {}
    BenillaPadCharDB.binds = BenillaPadCharDB.binds or {}
    BenillaPadCharDB.binds[layer] = BenillaPadCharDB.binds[layer] or {}
    BenillaPadCharDB.binds[layer][btn] = value
    P.Changed()
end

-- ── Per-form trigger layers ──
-- 1.12 pages only slots 1-12 by stance and form, and 120 slots leave no room for a second set of
-- trigger layers, so a form's LT / RT / LT+RT buttons are overrides kept here: a spell (cast by
-- name and rank) or a game action, keyed by the bonus bar offset. Unset buttons fall back to the
-- shared slots.

-- The current form's key (GetBonusBarOffset: druid forms, warrior stances, stealth), 0 for none.
function P.FormKey()
    return GetBonusBarOffset() or 0
end

-- The name of the active shapeshift form, for labels.
function P.FormName()
    for i = 1, GetNumShapeshiftForms() do
        local _, name, active = GetShapeshiftFormInfo(i)
        if active then
            return name
        end
    end
    return nil
end

function P.FormOverride(btn, layer)
    local key = P.FormKey()
    if key == 0 or layer == "bare" then
        return nil
    end
    local char = BenillaPadCharDB
    local forms = char and char.formBinds and char.formBinds[key]
    return forms and forms[layer] and forms[layer][btn]
end

function P.SetFormOverride(btn, layer, value)
    local key = P.FormKey()
    if key == 0 or layer == "bare" then
        return false
    end
    BenillaPadCharDB = BenillaPadCharDB or {}
    local char = BenillaPadCharDB
    char.formBinds = char.formBinds or {}
    char.formBinds[key] = char.formBinds[key] or {}
    char.formBinds[key][layer] = char.formBinds[key][layer] or {}
    char.formBinds[key][layer][btn] = value
    P.Changed()
    return true
end

-- The spellbook index of a spell by name and rank, cached until the book changes.
local bookCache = {}
function P.SpellIndex(name, rank)
    local key = name .. "|" .. (rank or "")
    if bookCache[key] ~= nil then
        return bookCache[key] or nil
    end
    local found = false
    local i = 1
    while true do
        local n, r = GetSpellName(i, "spell")
        if not n then
            break
        end
        if n == name and (not rank or rank == "" or r == rank) then
            found = i
        end
        i = i + 1
    end
    bookCache[key] = found
    return found or nil
end

local book = CreateFrame("Frame")
book:RegisterEvent("SPELLS_CHANGED")
book:RegisterEvent("LEARNED_SPELL_IN_TAB")
book:SetScript("OnEvent", function() bookCache = {} end)

-- Cast a form override's spell.
function P.CastOverride(over)
    if over.spell then
        if over.rank and over.rank ~= "" then
            CastSpellByName(over.spell .. "(" .. over.rank .. ")")
        else
            CastSpellByName(over.spell)
        end
    end
end

-- The layer and button a pad slot belongs to (its own slot, 1-48).
function P.SlotOwner(slot)
    local layer = P.LAYERS[math.floor((slot - 1) / 12) + 1]
    local btn = P.BUTTONS[slot - 12 * math.floor((slot - 1) / 12)]
    return layer, btn
end

-- What a button does on a layer: ("slot", n), ("action", id), ("form", override) or nil.
function P.Binding(btn, layer)
    if SYSTEM_ACTIONS[btn] then
        return "action", SYSTEM_ACTIONS[btn]
    end
    local form = P.FormOverride(btn, layer)
    if form then
        if form.action then
            return "action", form.action
        end
        return "form", form
    end
    local over = P.Override(btn, layer)
    if over and over.action then
        return "action", over.action
    end
    if not over and not (FACES[btn] and P.Setting("faceSpells")) then
        local defaults = DEFAULT_ACTIONS[layer]
        if defaults and defaults[btn] then
            return "action", defaults[btn]
        end
    end
    local slot = P.OwnSlot(btn, layer)
    if not slot then
        return nil
    end
    return "slot", slot
end

-- Tell every view that a binding or a slot changed.
function P.Changed()
    if P.Bar then
        P.Bar.Layout()
        P.Bar.Refresh()
    end
    if P.Binds and P.Binds.Refresh then
        P.Binds.Refresh()
    end
    if P.Xp then
        P.Xp.Refresh()
    end
end

-- The slot a pad slot casts from, paged as the main bar is: slots 1-12 move to the bonus bar of
-- the current stance or form (ActionButton.lua's ActionButton_GetPagedID).
function P.PagedSlot(slot)
    if slot <= 12 then
        local offset = GetBonusBarOffset()
        if offset and offset > 0 then
            local pages = NUM_ACTIONBAR_PAGES or 6
            local buttons = NUM_ACTIONBAR_BUTTONS or 12
            return slot + (pages + offset - 1) * buttons
        end
    end
    return slot
end

-- ── Round icons ──

-- A round icon of diameter `d` centred on `parent`: horizontal strips of the icon texture, each
-- cropped to the circle's width at its height (1.12 has no texture masks; a ring over the edge
-- hides the steps). Returns the strips, for P.SetIcon and P.TintIcon.
function P.NewRoundIcon(parent, d, count)
    count = count or 14
    local trim = 0.07
    local strips = {}
    local size = parent:GetWidth()
    local top = (size - d) / 2
    for i = 1, count do
        local y0 = (i - 1) / count
        local y1 = i / count
        -- The chord at the strip's edge nearest the centre, so the ring covers the overshoot.
        local near = y1 - 0.5
        if y0 > 0.5 then
            near = y0 - 0.5
        elseif y1 > 0.5 then
            near = 0
        end
        local half = math.sqrt(math.max(0, 0.25 - near * near))
        local l, r = 0.5 - half, 0.5 + half
        local t = parent:CreateTexture(nil, "ARTWORK")
        t:SetWidth(d * (r - l))
        t:SetHeight(d / count)
        t:SetPoint("TOPLEFT", parent, "TOPLEFT", top + d * l, -(top + d * y0))
        local span = 1 - 2 * trim
        t:SetTexCoord(trim + span * l, trim + span * r, trim + span * y0, trim + span * y1)
        strips[i] = t
    end
    return strips
end

function P.SetIcon(strips, tex)
    for i = 1, table.getn(strips) do
        if tex then
            strips[i]:SetTexture(tex)
            strips[i]:Show()
        else
            strips[i]:Hide()
        end
    end
end

function P.TintIcon(strips, r, g, b)
    for i = 1, table.getn(strips) do
        strips[i]:SetVertexColor(r, g, b)
    end
end

-- A game action's icon; some change with the bags (the quest item).
function P.ActionIcon(id)
    local action = P.Actions and P.Actions[id]
    if not action then
        return nil
    end
    if action.iconFn then
        return action.iconFn() or action.icon
    end
    return action.icon
end

-- ── The Rust side's calls ──

-- The binding command a press runs, or nil for nothing.
function BenillaPad_Resolve(btn, layer)
    local kind, value = P.Binding(btn, layer)
    if kind == "form" then
        -- The button's own slot command; its body casts the override.
        local own = P.OwnSlot(btn, layer)
        return own and ("BENILLAPAD_ACTION" .. own)
    end
    if kind == "slot" then
        if value >= 1 and value <= P.MAX_SLOT then
            return "BENILLAPAD_ACTION" .. value
        end
        return nil
    end
    if kind == "action" then
        local action = P.Actions and P.Actions[value]
        if action then
            return action.command
        end
    end
    return nil
end

-- The pad's mode ("world", "wheel", "menu", "keyboard", "cursor") and the settings generation.
-- This addon's own windows set P.mode; otherwise an open stock window asks for the cursor.
function BenillaPad_Mode()
    if P.mode == "world" and P.Cursor and P.Cursor.Wanted() then
        return "cursor", P.settingsGen
    end
    return P.mode, P.settingsGen
end

-- The settings the Rust side applies: deadzone, look speed, invert Y, shoulder zoom, cursor speed.
function BenillaPad_Settings()
    return P.Setting("deadzone"), P.Setting("lookSpeed"), P.Setting("invertY") and true or false,
        P.Setting("shoulderZoom") and true or false, P.Setting("cursorSpeed"),
        P.Setting("interactLootAll") and true or false
end

-- The face symbols to draw: the setting, or the pad's own.
function P.Symbols()
    local chosen = P.Setting("symbols")
    if chosen == "auto" then
        return P.style
    end
    return chosen
end

-- Put every setting back to its default.
function P.ResetSettings()
    BenillaPadDB = BenillaPadDB or {}
    BenillaPadDB.settings = {}
    P.settingsGen = P.settingsGen + 1
    P.Changed()
end

-- A pad slot's binding body (Bindings.xml): cast on the press or on the release.
function BenillaPad_ActionKey(slot)
    local onPress = P.Setting("castOnPress")
    local layer, btn = P.SlotOwner(slot)
    local form = P.FormOverride(btn, layer)
    if form then
        if (keystate == "down") == (onPress and true or false) then
            P.CastOverride(form)
        end
        return
    end
    local paged = P.PagedSlot(slot)
    if keystate == "down" then
        if onPress then
            UseAction(paged, 0)
        end
    elseif not onPress then
        UseAction(paged, 0)
    end
end

-- ── Events ──

local listeners = {}

-- Call fn(event, arg1, arg2, arg3, arg4) for every Core event below; true takes the event.
function P.Listen(fn)
    table.insert(listeners, fn)
end

local core = CreateFrame("Frame", "BenillaPadCore")
core:RegisterEvent("ADDON_LOADED")
core:RegisterEvent("BENILLAPAD_CONNECTED")
core:RegisterEvent("BENILLAPAD_DISCONNECTED")
core:RegisterEvent("BENILLAPAD_LAYER")
core:RegisterEvent("BENILLAPAD_BUTTON")
core:RegisterEvent("BENILLAPAD_NAV")
core:RegisterEvent("BENILLAPAD_STICK")
core:SetScript("OnEvent", function()
    if event == "ADDON_LOADED" then
        if arg1 ~= "BenillaPad" then
            return
        end
        BenillaPadDB = BenillaPadDB or {}
        BenillaPadDB.settings = BenillaPadDB.settings or {}
        BenillaPadCharDB = BenillaPadCharDB or {}
        BenillaPadCharDB.binds = BenillaPadCharDB.binds or {}
        P.settingsGen = P.settingsGen + 1
    elseif event == "BENILLAPAD_CONNECTED" then
        P.connected = true
        P.style = arg1 or "xbox"
    elseif event == "BENILLAPAD_DISCONNECTED" then
        P.connected = false
        P.held = {}
        P.layer = "bare"
    elseif event == "BENILLAPAD_LAYER" then
        P.layer = arg1 or "bare"
    elseif event == "BENILLAPAD_BUTTON" then
        P.held[arg1] = arg2 and true or nil
    end
    -- A listener that answers true has taken the event: the windows on top (the keyboard)
    -- register first, so the one underneath does not also act on the same press.
    for i = 1, table.getn(listeners) do
        if listeners[i](event, arg1, arg2, arg3, arg4) then
            break
        end
    end
end)

-- The Key Bindings window's names for this addon's rows.
BINDING_HEADER_BENILLAPAD = "BenillaPad (controller)"
for i = 1, P.MAX_SLOT do
    setglobal("BINDING_NAME_BENILLAPAD_ACTION" .. i, "Pad action slot " .. i)
end
BINDING_NAME_BENILLAPAD_BACK = "Back / stop casting / clear target"
BINDING_NAME_BENILLAPAD_INSPECT = "Inspect target"
BINDING_NAME_BENILLAPAD_TARGETENEMY = "Target nearest enemy"
BINDING_NAME_BENILLAPAD_TARGETFRIEND = "Target nearest friend"
BINDING_NAME_BENILLAPAD_TARGETSELF = "Target yourself"
BINDING_NAME_BENILLAPAD_ATTACK = "Attack target"
BINDING_NAME_BENILLAPAD_MENU = "Controller menu"
BINDING_NAME_BENILLAPAD_WHEEL = "Window wheel"
BINDING_NAME_BENILLAPAD_CONSUMABLES = "Consumables wheel"
BINDING_NAME_BENILLAPAD_QUESTITEM = "Use quest item"
BINDING_NAME_BENILLAPAD_BOTWHEEL = "Bot wheel"
BINDING_NAME_BENILLAPAD_CHAT = "Quick Chat"

SLASH_BENILLAPAD1 = "/pad"
SlashCmdList["BENILLAPAD"] = function(msg)
    local _, _, key, value = string.find(msg or "", "^(%S+)%s*(.*)$")
    if key == "press" then
        P.SetSetting("castOnPress", value ~= "off")
        P.Print("cast on press: " .. (P.Setting("castOnPress") and "on" or "off"))
    elseif key == "scale" and tonumber(value) then
        P.SetSetting("scale", tonumber(value))
        if P.Bar then P.Bar.Layout() end
    elseif key == "y" and tonumber(value) then
        P.SetSetting("offsetY", tonumber(value))
        if P.Bar then P.Bar.Layout() end
    elseif key == "look" and tonumber(value) then
        P.SetSetting("lookSpeed", tonumber(value))
    elseif key == "invert" then
        P.SetSetting("invertY", value == "on")
    elseif key == "chat" then
        P.Chat.Toggle()
    elseif key == "binds" then
        P.Binds.Toggle()
    elseif key == "menu" or key == "" or key == nil then
        P.Menu.Toggle()
    elseif key == "show" then
        P.SetSetting("alwaysShow", value ~= "off")
    else
        P.Print("/pad (menu), binds, press on|off, scale <n>, y <offset>, look <rad/s>, invert on|off, show on|off")
        P.Print("connected: " .. (P.connected and P.style or "no") .. ", layer: " .. P.layer)
    end
end
