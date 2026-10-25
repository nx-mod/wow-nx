-- Binding the pad. The All Keybinds window shows every bindable button on one layer, laid out as
-- on the bar; the picker lists what can go on a button (spellbook, macros, pet, game actions).
-- Driven by the pad (BENILLAPAD_NAV while the window is up, the Rust side's "menu" mode) or the
-- mouse. A spell, macro or item lives in the button's own action slot, so the server keeps it;
-- a game action is the button's setting in BenillaPadCharDB.

local P = BenillaPad
local Binds = {}
P.Binds = Binds

local CELL = 44
local ROWS = 12
local LAYER_TITLE = { bare = "No trigger", lt = "LT", rt = "RT", ltrt = "LT + RT" }
local BUTTON_TITLE = {
    DUP = "D-pad Up", DDOWN = "D-pad Down", DLEFT = "D-pad Left", DRIGHT = "D-pad Right",
    A = "A", B = "B", X = "X", Y = "Y", LB = "LB", RB = "RB", L3 = "L3", R3 = "R3",
}
local PLACE = {
    DUP = { -170, 52 }, DDOWN = { -170, -60 }, DLEFT = { -226, -4 }, DRIGHT = { -114, -4 },
    Y = { 170, 52 }, A = { 170, -60 }, X = { 114, -4 }, B = { 226, -4 },
    LB = { -170, 118 }, RB = { 170, 118 }, L3 = { -60, -124 }, R3 = { 60, -124 },
}
-- The short names over the cells; the line under the grid spells them out.
local CELL_LABEL = {
    DUP = "Up", DDOWN = "Down", DLEFT = "Left", DRIGHT = "Right",
    A = "A", B = "B", X = "X", Y = "Y", LB = "LB", RB = "RB", L3 = "L3", R3 = "R3",
}
local BACKDROP = {
    bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background",
    edgeFile = "Interface\\DialogFrame\\UI-DialogBox-Border",
    tile = true, tileSize = 32, edgeSize = 32,
    insets = { left = 11, right = 12, top = 12, bottom = 11 },
}

Binds.tab = "bare"
Binds.selected = "DUP"
Binds.picked = nil
Binds.cells = {}

-- ── Reading a button ──

local scan = CreateFrame("GameTooltip", "BenillaPadScanTip", nil, "GameTooltipTemplate")

-- The name of what sits in an action slot, read off its tooltip (1.12 has no GetActionInfo).
local function slotName(slot)
    if not HasAction(slot) then
        return nil
    end
    scan:SetOwner(UIParent, "ANCHOR_NONE")
    scan:SetAction(slot)
    local line = getglobal("BenillaPadScanTipTextLeft1")
    local text = line and line:GetText()
    scan:Hide()
    return text
end

-- The icon and the name of a button on a layer.
function Binds.Describe(btn, layer)
    local kind, value = P.Binding(btn, layer)
    if kind == "form" then
        local name = value.spell or "?"
        if value.rank and value.rank ~= "" then
            name = name .. " (" .. value.rank .. ")"
        end
        return value.icon, name
    end
    if kind == "action" then
        local action = P.Actions[value]
        if action then
            return P.ActionIcon(value), action.label
        end
    elseif kind == "slot" then
        local slot = P.PagedSlot(value)
        return GetActionTexture(slot), slotName(slot) or "Empty"
    end
    return nil, "Empty"
end

-- The layer the window edits: the one the triggers hold, else the chosen tab.
function Binds.Layer()
    if P.layer and P.layer ~= "bare" then
        return P.layer
    end
    return Binds.tab
end

-- ── Changing a button ──

-- Swap two buttons' settings and their action slots' contents, through the cursor as the
-- stock bars move actions: pick up a, drop on b (b's old action comes back on the cursor), drop
-- on a.
function Binds.Swap(btnA, layerA, btnB, layerB)
    if btnA == btnB and layerA == layerB then
        return
    end
    if P.FormOverride(btnA, layerA) or P.FormOverride(btnB, layerB) then
        P.Print("in a form, set the trigger layers from the spell list (Y); X clears")
        return
    end
    local slotA = P.PagedSlot(P.OwnSlot(btnA, layerA))
    local slotB = P.PagedSlot(P.OwnSlot(btnB, layerB))
    local overA = P.Override(btnA, layerA)
    local overB = P.Override(btnB, layerB)
    -- A default game action travels as an explicit one.
    local kindA, valueA = P.Binding(btnA, layerA)
    local kindB, valueB = P.Binding(btnB, layerB)
    if kindA == "action" then overA = { action = valueA } else overA = { slot = true } end
    if kindB == "action" then overB = { action = valueB } else overB = { slot = true } end
    ClearCursor()
    PickupAction(slotA)
    PlaceAction(slotB)
    PlaceAction(slotA)
    ClearCursor()
    P.SetOverride(btnA, layerA, overB)
    P.SetOverride(btnB, layerB, overA)
end

-- Empty a button: no game action, nothing in its slot.
function Binds.Clear(btn, layer)
    if P.FormOverride(btn, layer) then
        P.SetFormOverride(btn, layer, nil)
        return
    end
    local slot = P.PagedSlot(P.OwnSlot(btn, layer))
    ClearCursor()
    PickupAction(slot)
    ClearCursor()
    P.SetOverride(btn, layer, { slot = true })
end

-- Put a picker entry on a button.
function Binds.Assign(entry, btn, layer)
    if not P.Bindable(btn) then
        return
    end
    if P.FormKey() > 0 and layer ~= "bare" then
        if entry.action then
            P.SetFormOverride(btn, layer, { action = entry.action })
            return
        end
        if entry.book == "spell" then
            local name, rank = GetSpellName(entry.spell, "spell")
            P.SetFormOverride(btn, layer, { spell = name, rank = rank, icon = entry.icon })
            return
        end
        P.Print("macros and pet spells go on the shared trigger layers, not a form's own")
    end
    if entry.action then
        P.SetOverride(btn, layer, { action = entry.action })
        return
    end
    local slot = P.PagedSlot(P.OwnSlot(btn, layer))
    ClearCursor()
    if entry.macro then
        PickupMacro(entry.macro)
    else
        PickupSpell(entry.spell, entry.book)
    end
    PlaceAction(slot)
    ClearCursor()
    P.SetOverride(btn, layer, { slot = true })
end

-- What the cursor holds, dropped on a button (the mouse's drag and drop). 1.12 cannot ask the
-- cursor for a macro, so the drop is kept when the slot holds something afterwards.
function Binds.Drop(btn, layer)
    if not P.Bindable(btn) then
        return
    end
    local slot = P.PagedSlot(P.OwnSlot(btn, layer))
    PlaceAction(slot)
    if HasAction(slot) then
        P.SetOverride(btn, layer, { slot = true })
    end
end

-- ── The window ──

local win = CreateFrame("Frame", "BenillaPadBinds", UIParent)
win:SetWidth(620)
win:SetHeight(500)
win:SetPoint("CENTER", UIParent, "CENTER", -150, 40)
win:SetBackdrop(BACKDROP)
win:SetFrameStrata("DIALOG")
win:EnableMouse(true)
win:SetMovable(true)
win:RegisterForDrag("LeftButton")
win:SetScript("OnDragStart", function() this:StartMoving() end)
win:SetScript("OnDragStop", function() this:StopMovingOrSizing() end)
win:Hide()
tinsert(UISpecialFrames, "BenillaPadBinds")
Binds.frame = win

local title = win:CreateFontString(nil, "ARTWORK", "GameFontNormalLarge")
title:SetPoint("TOP", win, "TOP", 0, -18)
title:SetText("Controller keybinds")

local close = CreateFrame("Button", nil, win, "UIPanelCloseButton")
close:SetPoint("TOPRIGHT", win, "TOPRIGHT", -6, -6)

local layerText = win:CreateFontString(nil, "ARTWORK", "GameFontNormalLarge")
layerText:SetPoint("CENTER", win, "CENTER", 0, -4)

local described = win:CreateFontString(nil, "ARTWORK", "GameFontHighlight")
described:SetPoint("BOTTOM", win, "BOTTOM", 0, 52)

local hints = win:CreateFontString(nil, "ARTWORK", "GameFontDisableSmall")
hints:SetPoint("BOTTOM", win, "BOTTOM", 0, 22)
hints:SetWidth(580)

-- The layer tabs; LB / RB step them, holding a trigger shows that layer.
local tabs = {}
for i = 1, table.getn(P.LAYERS) do
    local layer = P.LAYERS[i]
    local t = CreateFrame("Button", nil, win)
    t:SetWidth(110)
    t:SetHeight(22)
    t:SetPoint("TOP", win, "TOP", (i - 2.5) * 118, -48)
    local bg = t:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(t)
    t.bg = bg
    local text = t:CreateFontString(nil, "ARTWORK", "GameFontNormal")
    text:SetPoint("CENTER", t, "CENTER", 0, 0)
    text:SetText(LAYER_TITLE[layer])
    t.text = text
    t.layer = layer
    t:SetScript("OnClick", function()
        Binds.tab = this.layer
        Binds.Refresh()
    end)
    tabs[i] = t
end

local function newCell(btn)
    local c = CreateFrame("Button", "BenillaPadBindCell" .. btn, win)
    c:SetWidth(CELL)
    c:SetHeight(CELL)
    c:SetPoint("CENTER", win, "CENTER", PLACE[btn][1], PLACE[btn][2])
    c.btn = btn
    local slot = c:CreateTexture(nil, "BACKGROUND")
    slot:SetTexture("Interface\\Buttons\\UI-Quickslot")
    slot:SetWidth(CELL * 64 / 36)
    slot:SetHeight(CELL * 64 / 36)
    slot:SetPoint("CENTER", c, "CENTER", 0, 0)
    local icon = c:CreateTexture(nil, "ARTWORK")
    icon:SetWidth(CELL - 4)
    icon:SetHeight(CELL - 4)
    icon:SetPoint("CENTER", c, "CENTER", 0, 0)
    icon:SetTexCoord(0.07, 0.93, 0.07, 0.93)
    c.icon = icon
    local sel = c:CreateTexture(nil, "OVERLAY")
    sel:SetTexture("Interface\\Buttons\\CheckButtonHilight")
    sel:SetBlendMode("ADD")
    sel:SetWidth(CELL + 14)
    sel:SetHeight(CELL + 14)
    sel:SetPoint("CENTER", c, "CENTER", 0, 0)
    c.sel = sel
    local glyph = c:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
    glyph:SetPoint("BOTTOM", c, "TOP", 0, 3)
    glyph:SetText(CELL_LABEL[btn])
    c.glyph = glyph
    c:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    c:RegisterForDrag("LeftButton")
    c:SetScript("OnClick", function()
        Binds.selected = this.btn
        if arg1 == "RightButton" then
            Binds.Clear(this.btn, Binds.Layer())
        elseif CursorHasSpell() or CursorHasItem() then
            Binds.Drop(this.btn, Binds.Layer())
        else
            Binds.Nav("A")
        end
    end)
    c:SetScript("OnReceiveDrag", function()
        Binds.Drop(this.btn, Binds.Layer())
    end)
    c:SetScript("OnDragStart", function()
        local kind, value = P.Binding(this.btn, Binds.Layer())
        if kind == "slot" then
            PickupAction(P.PagedSlot(value))
            P.SetOverride(this.btn, Binds.Layer(), { slot = true })
        end
    end)
    c:SetScript("OnEnter", function()
        local kind, value = P.Binding(this.btn, Binds.Layer())
        if kind == "slot" and HasAction(P.PagedSlot(value)) then
            GameTooltip:SetOwner(this, "ANCHOR_RIGHT")
            GameTooltip:SetAction(P.PagedSlot(value))
        end
    end)
    c:SetScript("OnLeave", function() GameTooltip:Hide() end)
    Binds.cells[btn] = c
end

for i = 1, table.getn(P.BUTTONS) do
    newCell(P.BUTTONS[i])
end

function Binds.Refresh()
    if not win:IsVisible() then
        return
    end
    local layer = Binds.Layer()
    for i = 1, table.getn(tabs) do
        local t = tabs[i]
        if t.layer == layer then
            t.bg:SetTexture(1, 0.75, 0.1, 0.9)
            t.text:SetTextColor(0, 0, 0)
        else
            t.bg:SetTexture(0.1, 0.1, 0.15, 0.9)
            t.text:SetTextColor(1, 0.82, 0)
        end
    end
    layerText:SetText(LAYER_TITLE[layer])
    for btn, c in pairs(Binds.cells) do
        local icon = Binds.Describe(btn, layer)
        c.icon:SetTexture(icon)
        if icon then c.icon:Show() else c.icon:Hide() end
        if btn == Binds.selected then c.sel:Show() else c.sel:Hide() end
        if Binds.picked and Binds.picked.btn == btn and Binds.picked.layer == layer then
            c.icon:SetVertexColor(0.4, 0.4, 0.4)
        else
            c.icon:SetVertexColor(1, 1, 1)
        end
    end
    local _, name = Binds.Describe(Binds.selected, layer)
    local prefix = ""
    if layer ~= "bare" then
        prefix = LAYER_TITLE[layer] .. " + "
    end
    local formNote = ""
    if layer ~= "bare" and P.FormKey() > 0 then
        formNote = "   |cff66bbff(" .. (P.FormName() or "this form") .. "'s own set)|r"
    end
    described:SetText(prefix .. BUTTON_TITLE[Binds.selected] .. ":  " .. (name or "Empty") .. formNote)
    if Binds.capture then
        hints:SetText("|cffffd100Press the combo|r to bind " .. Binds.capture.label
            .. " (hold LT / RT for their layers).   Select: cancel")
    elseif Binds.picked then
        hints:SetText("A: put it here (swaps)   B: cancel   D-pad: move")
    else
        hints:SetText("D-pad: move   A: pick up / put down   X: clear   Y: spell list   Start: default"
            .. "   LB / RB: layer   Hold LT / RT: that layer   B: close")
    end
end

-- The nearest cell from `from` in a D-pad direction.
local function step(from, dir)
    local fx, fy = PLACE[from][1], PLACE[from][2]
    local best, bestScore
    for btn, at in pairs(PLACE) do
        local dx, dy = at[1] - fx, at[2] - fy
        local along, across
        if dir == "DUP" then along, across = dy, dx
        elseif dir == "DDOWN" then along, across = -dy, dx
        elseif dir == "DLEFT" then along, across = -dx, dy
        else along, across = dx, dy end
        if along > 1 then
            local score = along + 2 * math.abs(across)
            if not bestScore or score < bestScore then
                best, bestScore = btn, score
            end
        end
    end
    return best or from
end

-- The pad, while the window is up.
function Binds.Nav(btn)
    if Binds.capture then
        if btn == "SELECT" or btn == "START" then
            Binds.capture = nil
        elseif P.Bindable(btn) then
            Binds.Assign(Binds.capture, btn, P.layer or "bare")
            Binds.selected = btn
            Binds.tab = P.layer or "bare"
            Binds.capture = nil
            P.Print("bound " .. Binds.capture_label_done(btn))
        end
        Binds.Refresh()
        return
    end
    if P.Picker and P.Picker.IsOpen() then
        P.Picker.Nav(btn)
        return
    end
    local layer = Binds.Layer()
    if btn == "DUP" or btn == "DDOWN" or btn == "DLEFT" or btn == "DRIGHT" then
        Binds.selected = step(Binds.selected, btn)
    elseif btn == "A" then
        if Binds.picked then
            Binds.Swap(Binds.picked.btn, Binds.picked.layer, Binds.selected, layer)
            Binds.picked = nil
        else
            Binds.picked = { btn = Binds.selected, layer = layer }
        end
    elseif btn == "X" then
        Binds.Clear(Binds.selected, layer)
    elseif btn == "Y" then
        P.Picker.Open()
    elseif btn == "START" then
        P.SetOverride(Binds.selected, layer, nil)
    elseif btn == "B" then
        if Binds.picked then
            Binds.picked = nil
        else
            win:Hide()
        end
    elseif btn == "SELECT" then
        win:Hide()
    elseif btn == "LB" or btn == "RB" then
        local i = 1
        for j = 1, table.getn(P.LAYERS) do
            if P.LAYERS[j] == Binds.tab then i = j end
        end
        if btn == "LB" then i = i - 1 else i = i + 1 end
        if i < 1 then i = table.getn(P.LAYERS) end
        if i > table.getn(P.LAYERS) then i = 1 end
        Binds.tab = P.LAYERS[i]
    end
    Binds.Refresh()
end

function Binds.capture_label_done(btn)
    local layer = P.layer or "bare"
    local prefix = ""
    if layer ~= "bare" then
        prefix = LAYER_TITLE[layer] .. " + "
    end
    return "to " .. prefix .. BUTTON_TITLE[btn]
end

-- Bind `entry` to the next combo pressed.
function Binds.Capture(entry)
    Binds.capture = entry
    Binds.Refresh()
end

function Binds.Toggle()
    if win:IsVisible() then
        win:Hide()
    else
        win:Show()
    end
end

win:SetScript("OnShow", function()
    P.mode = "menu"
    Binds.Refresh()
end)
win:SetScript("OnHide", function()
    Binds.picked = nil
    Binds.capture = nil
    if P.Picker then P.Picker.Close() end
    P.mode = "world"
end)

P.Listen(function(event, arg1, arg2)
    if not win:IsVisible() then
        return
    end
    if event == "BENILLAPAD_NAV" then
        if arg2 then
            Binds.Nav(arg1)
        end
        return true
    elseif event == "BENILLAPAD_LAYER" then
        Binds.Refresh()
    end
end)

local watcher = CreateFrame("Frame", nil, win)
watcher:RegisterEvent("ACTIONBAR_SLOT_CHANGED")
watcher:RegisterEvent("UPDATE_BONUS_ACTIONBAR")
watcher:RegisterEvent("UPDATE_SHAPESHIFT_FORMS")
watcher:SetScript("OnEvent", function() Binds.Refresh() end)

-- ── The picker ──

local Picker = {}
P.Picker = Picker

local pick = CreateFrame("Frame", "BenillaPadPicker", win)
pick:SetWidth(300)
pick:SetHeight(500)
pick:SetPoint("TOPLEFT", win, "TOPRIGHT", -4, 0)
pick:SetBackdrop(BACKDROP)
pick:EnableMouse(true)
pick:Hide()

local pickTitle = pick:CreateFontString(nil, "ARTWORK", "GameFontNormal")
pickTitle:SetPoint("TOP", pick, "TOP", 0, -20)

local pickHint = pick:CreateFontString(nil, "ARTWORK", "GameFontDisableSmall")
pickHint:SetPoint("BOTTOM", pick, "BOTTOM", 0, 20)
pickHint:SetWidth(270)
pickHint:SetText("A: bind to the selected button   X: bind to a combo   D-pad: move / list   B: back")

Picker.lists = {}
Picker.list = 1
Picker.row = 1
Picker.top = 1
local rows = {}

-- Every list: one per spellbook tab, then macros, pet, game actions.
function Picker.Build()
    local lists = {}
    for t = 1, GetNumSpellTabs() do
        local name, _, offset, count = GetSpellTabInfo(t)
        local list = { title = name, entries = {} }
        for i = offset + 1, offset + count do
            local spell, rank = GetSpellName(i, "spell")
            if spell and not IsSpellPassive(i, "spell") then
                table.insert(list.entries, {
                    label = spell, sub = rank, icon = GetSpellTexture(i, "spell"),
                    spell = i, book = "spell",
                })
            end
        end
        table.insert(lists, list)
    end
    local macros = { title = "Macros", entries = {} }
    for i = 1, GetNumMacros() do
        local name, icon = GetMacroInfo(i)
        if name then
            table.insert(macros.entries, { label = name, sub = "macro", icon = icon, macro = i })
        end
    end
    table.insert(lists, macros)
    local petCount = HasPetSpells()
    if petCount and petCount > 0 then
        local pet = { title = "Pet", entries = {} }
        for i = 1, petCount do
            local spell, rank = GetSpellName(i, "pet")
            if spell and not IsSpellPassive(i, "pet") then
                table.insert(pet.entries, {
                    label = spell, sub = rank, icon = GetSpellTexture(i, "pet"),
                    spell = i, book = "pet",
                })
            end
        end
        table.insert(lists, pet)
    end
    local actions = { title = "Game actions", entries = {} }
    local order = { "jump", "interact", "back", "inspect", "targetenemy", "targetfriend",
        "targetself", "attack", "autorun", "sit", "wheel", "consumables", "questitem", "botwheel", "quickchat", "menu" }
    for i = 1, table.getn(order) do
        local a = P.Actions[order[i]]
        table.insert(actions.entries, { label = a.label, sub = "game action", icon = P.ActionIcon(a.id), action = a.id })
    end
    table.insert(lists, actions)
    Picker.lists = lists
    if Picker.list > table.getn(lists) then
        Picker.list = 1
    end
end

for i = 1, ROWS do
    local r = CreateFrame("Button", nil, pick)
    r:SetWidth(264)
    r:SetHeight(28)
    r:SetPoint("TOPLEFT", pick, "TOPLEFT", 18, -44 - (i - 1) * 30)
    local hi = r:CreateTexture(nil, "BACKGROUND")
    hi:SetAllPoints(r)
    hi:SetTexture(1, 0.75, 0.1, 0.25)
    r.hi = hi
    local icon = r:CreateTexture(nil, "ARTWORK")
    icon:SetWidth(24)
    icon:SetHeight(24)
    icon:SetPoint("LEFT", r, "LEFT", 2, 0)
    r.icon = icon
    local text = r:CreateFontString(nil, "ARTWORK", "GameFontHighlight")
    text:SetPoint("LEFT", icon, "RIGHT", 6, 0)
    text:SetJustifyH("LEFT")
    r.text = text
    local sub = r:CreateFontString(nil, "ARTWORK", "GameFontDisableSmall")
    sub:SetPoint("RIGHT", r, "RIGHT", -4, 0)
    r.sub = sub
    r.index = i
    r:SetScript("OnClick", function()
        Picker.row = Picker.top + this.index - 1
        Picker.Nav("A")
    end)
    rows[i] = r
end

function Picker.Refresh()
    local list = Picker.lists[Picker.list]
    if not list then
        return
    end
    pickTitle:SetText("< " .. (list.title or "?") .. " >   " .. Picker.list .. "/" .. table.getn(Picker.lists))
    local n = table.getn(list.entries)
    if Picker.row > n then Picker.row = n end
    if Picker.row < 1 then Picker.row = 1 end
    if Picker.row < Picker.top then Picker.top = Picker.row end
    if Picker.row > Picker.top + ROWS - 1 then Picker.top = Picker.row - ROWS + 1 end
    for i = 1, ROWS do
        local r = rows[i]
        local e = list.entries[Picker.top + i - 1]
        if e then
            r.icon:SetTexture(e.icon)
            r.text:SetText(e.label)
            r.sub:SetText(e.sub or "")
            if Picker.top + i - 1 == Picker.row then r.hi:Show() else r.hi:Hide() end
            r:Show()
        else
            r:Hide()
        end
    end
end

function Picker.Open()
    Picker.Build()
    pick:Show()
    Picker.Refresh()
end

function Picker.Close()
    pick:Hide()
end

function Picker.IsOpen()
    return pick:IsVisible()
end

function Picker.Nav(btn)
    local list = Picker.lists[Picker.list]
    if btn == "DUP" then
        Picker.row = Picker.row - 1
    elseif btn == "DDOWN" then
        Picker.row = Picker.row + 1
    elseif btn == "DLEFT" or btn == "LB" then
        Picker.list = Picker.list - 1
        if Picker.list < 1 then Picker.list = table.getn(Picker.lists) end
        Picker.row, Picker.top = 1, 1
    elseif btn == "DRIGHT" or btn == "RB" then
        Picker.list = Picker.list + 1
        if Picker.list > table.getn(Picker.lists) then Picker.list = 1 end
        Picker.row, Picker.top = 1, 1
    elseif btn == "A" then
        local e = list and list.entries[Picker.row]
        if e then
            Binds.Assign(e, Binds.selected, Binds.Layer())
            Picker.Close()
        end
    elseif btn == "X" then
        local e = list and list.entries[Picker.row]
        if e then
            Picker.Close()
            Binds.Capture(e)
            return
        end
    elseif btn == "B" or btn == "SELECT" then
        Picker.Close()
        Binds.Refresh()
        return
    end
    Picker.Refresh()
end
