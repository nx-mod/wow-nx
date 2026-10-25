-- The gamepad action bar: a D-pad cluster of square slots, a face cluster of round ones, LB and RB
-- above them and L3 and R3 below, showing what each button does on the layer the triggers hold.
-- Round icons are built from horizontal strips of the icon texture, each cropped to the circle's
-- width at its height: 1.12 has no texture masks, and the ring hides the steps.

local P = BenillaPad
local Bar = {}
P.Bar = Bar

local ART = P.ART
local SQUARE = 40
local ROUND = 46
-- The icon art's own border, trimmed.
local TRIM = 0.07
-- The ring art (tools/gen_art.py) is drawn at 1.3x the button: its band covers the round icon's
-- edge.
local RING_SCALE = 1.3

local BADGE_COLOR = {
    A = { 0.37, 0.75, 0.22 }, B = { 0.86, 0.20, 0.18 },
    X = { 0.20, 0.45, 0.90 }, Y = { 0.95, 0.75, 0.10 },
}
local PS_GLYPH = { A = "X", B = "O", X = "[]", Y = "/\\" }

-- Where each button sits, from the bar's centre.
local DPAD_X, FACE_X, STEP = -150, 150, 46
local PLACE = {
    DUP = { DPAD_X, STEP }, DDOWN = { DPAD_X, -STEP },
    DLEFT = { DPAD_X - STEP, 0 }, DRIGHT = { DPAD_X + STEP, 0 },
    Y = { FACE_X, STEP + 2 }, A = { FACE_X, -STEP - 2 },
    X = { FACE_X - STEP - 2, 0 }, B = { FACE_X + STEP + 2, 0 },
    LB = { DPAD_X - 100, 92 }, RB = { FACE_X + 100, 92 },
    L3 = { -52, -88 }, R3 = { 52, -88 },
}
local ROUND_KEYS = { Y = true, B = true, A = true, X = true, LB = true, RB = true, L3 = true, R3 = true }

local frame = CreateFrame("Frame", "BenillaPadBar", UIParent)
frame:SetWidth(560)
frame:SetHeight(260)
frame:SetFrameStrata("MEDIUM")
frame:Hide()
Bar.frame = frame
Bar.buttons = {}

local layerText = frame:CreateFontString(nil, "OVERLAY")
layerText:SetFont("Fonts\\FRIZQT__.TTF", 22, "OUTLINE")
layerText:SetPoint("CENTER", frame, "CENTER", 0, 4)
layerText:SetTextColor(1, 0.82, 0)

-- ── Building a button ──

local function newIconSquare(b, size)
    local icon = b:CreateTexture(nil, "ARTWORK")
    icon:SetWidth(size - 4)
    icon:SetHeight(size - 4)
    icon:SetPoint("CENTER", b, "CENTER", 0, 0)
    icon:SetTexCoord(TRIM, 1 - TRIM, TRIM, 1 - TRIM)
    b.strips = { icon }
end

local function newIconRound(b, size)
    b.strips = P.NewRoundIcon(b, size * 0.9)
end

local function newButton(key)
    local round = ROUND_KEYS[key]
    local size = round and ROUND or SQUARE
    if key == "LB" or key == "RB" or key == "L3" or key == "R3" then
        size = 40
    end
    local b = CreateFrame("Button", "BenillaPadButton" .. key, frame)
    b:SetWidth(size)
    b:SetHeight(size)
    b.key = key
    b.round = round
    b.size = size

    if round then
        local shadow = b:CreateTexture(nil, "BACKGROUND")
        shadow:SetTexture(ART .. "Disc")
        shadow:SetVertexColor(0, 0, 0, 0.65)
        shadow:SetWidth(size * 0.95)
        shadow:SetHeight(size * 0.95)
        shadow:SetPoint("CENTER", b, "CENTER", 0, 0)
        newIconRound(b, size)
        local ring = b:CreateTexture(nil, "OVERLAY")
        ring:SetTexture(ART .. "Ring")
        ring:SetVertexColor(0.85, 0.72, 0.5)
        ring:SetWidth(size * RING_SCALE)
        ring:SetHeight(size * RING_SCALE)
        ring:SetPoint("CENTER", b, "CENTER", 0, 0)
        b.ring = ring
        local hi = b:CreateTexture(nil, "OVERLAY")
        hi:SetTexture(ART .. "RingSelect")
        hi:SetWidth(size * RING_SCALE)
        hi:SetHeight(size * RING_SCALE)
        hi:SetPoint("CENTER", b, "CENTER", 0, 0)
        hi:Hide()
        b.pressed = hi
    else
        local slot = b:CreateTexture(nil, "BACKGROUND")
        slot:SetTexture("Interface\\Buttons\\UI-Quickslot")
        slot:SetWidth(size * 64 / 36)
        slot:SetHeight(size * 64 / 36)
        slot:SetPoint("CENTER", b, "CENTER", 0, 0)
        newIconSquare(b, size)
        local border = b:CreateTexture(nil, "OVERLAY")
        border:SetTexture("Interface\\Buttons\\UI-Quickslot2")
        border:SetWidth(size * 66 / 36)
        border:SetHeight(size * 66 / 36)
        border:SetPoint("CENTER", b, "CENTER", 0, -1)
        local hi = b:CreateTexture(nil, "OVERLAY")
        hi:SetTexture("Interface\\Buttons\\CheckButtonHilight")
        hi:SetBlendMode("ADD")
        hi:SetAllPoints(b)
        hi:Hide()
        b.pressed = hi
    end

    local cd = CreateFrame("Model", "BenillaPadButton" .. key .. "Cooldown", b, "CooldownFrameTemplate")
    cd:SetWidth(size * 0.9)
    cd:SetHeight(size * 0.9)
    cd:SetPoint("CENTER", b, "CENTER", 0, 0)
    b.cooldown = cd

    -- Text sits on a child above the cooldown model.
    local over = CreateFrame("Frame", nil, b)
    over:SetAllPoints(b)
    over:SetFrameLevel(cd:GetFrameLevel() + 2)

    local timer = over:CreateFontString(nil, "OVERLAY")
    timer:SetFont("Fonts\\FRIZQT__.TTF", size * 0.42, "OUTLINE")
    timer:SetPoint("CENTER", b, "CENTER", 0, 0)
    timer:SetTextColor(1, 1, 1)
    b.timer = timer

    local count = over:CreateFontString(nil, "OVERLAY", "NumberFontNormal")
    count:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", -2, 2)
    b.count = count

    local badge = over:CreateTexture(nil, "OVERLAY")
    badge:SetTexture(ART .. "Disc")
    badge:SetWidth(20)
    badge:SetHeight(20)
    badge:SetVertexColor(0.05, 0.05, 0.05, 0.9)
    badge:SetPoint("CENTER", b, "TOPRIGHT", -3, -3)
    b.badge = badge
    local glyph = over:CreateFontString(nil, "OVERLAY")
    glyph:SetFont("Fonts\\FRIZQT__.TTF", 11, "OUTLINE")
    glyph:SetPoint("CENTER", badge, "CENTER", 0, 0)
    b.glyph = glyph
    if PLACE[key][1] < 0 and (key == "LB" or key == "L3") then
        badge:ClearAllPoints()
        badge:SetPoint("CENTER", b, "TOPLEFT", 3, -3)
    end
    -- The D-pad reads by its diamond; no badge.
    if string.sub(key, 1, 1) == "D" then
        badge:Hide()
        glyph:Hide()
    end

    -- The mouse: drop a spell or item on a button, drag one off, read its tooltip.
    b:RegisterForDrag("LeftButton")
    b:SetScript("OnReceiveDrag", function()
        P.Binds.Drop(this.key, P.layer)
    end)
    b:SetScript("OnClick", function()
        if CursorHasSpell() or CursorHasItem() then
            P.Binds.Drop(this.key, P.layer)
        end
    end)
    b:SetScript("OnDragStart", function()
        if this.slot and HasAction(this.slot) then
            PickupAction(this.slot)
            P.SetOverride(this.key, P.layer, { slot = true })
        end
    end)
    b:SetScript("OnEnter", function()
        if this.slot and HasAction(this.slot) then
            GameTooltip:SetOwner(this, "ANCHOR_TOP")
            GameTooltip:SetAction(this.slot)
        end
    end)
    b:SetScript("OnLeave", function() GameTooltip:Hide() end)

    b:SetPoint("CENTER", frame, "CENTER", PLACE[key][1], PLACE[key][2])
    Bar.buttons[key] = b
    return b
end

for i = 1, table.getn(P.BUTTONS) do
    newButton(P.BUTTONS[i])
end

-- ── Updating ──

local function setIcon(b, tex)
    P.SetIcon(b.strips, tex)
end

local function tintIcon(b, r, g, bl)
    P.TintIcon(b.strips, r, g, bl)
end

local function setGlyph(b)
    local key = b.key
    local color = BADGE_COLOR[key]
    if color then
        local text = key
        if P.Symbols() == "playstation" then
            text = PS_GLYPH[key]
        end
        b.glyph:SetText(text)
        b.glyph:SetTextColor(color[1], color[2], color[3])
    else
        b.glyph:SetText(key)
        b.glyph:SetTextColor(0.9, 0.9, 0.9)
    end
end

-- What a button shows: its binding on the held layer.
function Bar.UpdateButton(b)
    local kind, value = P.Binding(b.key, P.layer)
    b.slot = nil
    b.spellIndex = nil
    local tex
    if kind == "form" then
        tex = value.icon
        b.spellIndex = value.spell and P.SpellIndex(value.spell, value.rank)
    elseif kind == "slot" then
        b.slot = P.PagedSlot(value)
        tex = GetActionTexture(b.slot)
    elseif kind == "action" then
        tex = P.ActionIcon(value)
    end
    setIcon(b, tex)
    setGlyph(b)
    if b.slot and HasAction(b.slot) then
        local start, duration, enable = GetActionCooldown(b.slot)
        CooldownFrame_SetTimer(b.cooldown, start, duration, enable)
        if IsConsumableAction(b.slot) then
            b.count:SetText(GetActionCount(b.slot))
        else
            b.count:SetText("")
        end
    elseif b.spellIndex then
        local start, duration, enable = GetSpellCooldown(b.spellIndex, "spell")
        CooldownFrame_SetTimer(b.cooldown, start, duration, enable)
        b.count:SetText("")
    else
        b.cooldown:Hide()
        b.count:SetText("")
    end
    Bar.UpdateState(b)
end

-- The fast-changing look: range and power tint, the countdown, the press highlight.
function Bar.UpdateState(b)
    if P.held[b.key] then
        b.pressed:Show()
    else
        b.pressed:Hide()
    end
    local slot = b.slot
    if b.spellIndex then
        tintIcon(b, 1, 1, 1)
        local start, duration, enable = GetSpellCooldown(b.spellIndex, "spell")
        local left = 0
        if enable == 1 and duration and duration > 1.5 then
            left = start + duration - GetTime()
        end
        if left > 0 then b.timer:SetText(math.ceil(left)) else b.timer:SetText("") end
        return
    end
    if not slot or not HasAction(slot) then
        tintIcon(b, 1, 1, 1)
        b.timer:SetText("")
        return
    end
    local usable, noMana = IsUsableAction(slot)
    if IsActionInRange(slot) == 0 then
        tintIcon(b, 0.85, 0.15, 0.15)
    elseif usable then
        tintIcon(b, 1, 1, 1)
    elseif noMana then
        tintIcon(b, 0.4, 0.45, 1)
    else
        tintIcon(b, 0.4, 0.4, 0.4)
    end
    local start, duration, enable = GetActionCooldown(slot)
    local left = 0
    if enable == 1 and duration and duration > 1.5 then
        left = start + duration - GetTime()
    end
    if left > 0 then
        if left >= 60 then
            b.timer:SetText(math.ceil(left / 60) .. "m")
        else
            b.timer:SetText(math.ceil(left))
        end
    else
        b.timer:SetText("")
    end
end

-- The stock bars, gone while the gamepad bar shows: each is hidden and moved under a hidden frame,
-- and its OnShow hides it again, so nothing in the stock code brings it back; afterwards it returns
-- to its own parent and shown state. The pet bar stays.
local STOCK = { "MainMenuBar", "MultiBarBottomLeft", "MultiBarBottomRight", "MultiBarLeft", "MultiBarRight" }
local hider = CreateFrame("Frame")
hider:Hide()
local stockState = {}
local stockHidden = false

local function guardOnShow(f, name)
    if f.benillaPadGuard then
        return
    end
    f.benillaPadGuard = true
    local old = f:GetScript("OnShow")
    f:SetScript("OnShow", function()
        if stockHidden then
            this:Hide()
            return
        end
        if old then
            old()
        end
    end)
end

function Bar.StockBar()
    local hide = (P.Setting("hideStockBars") and frame:IsShown()) and true or false
    if hide == stockHidden then
        return
    end
    stockHidden = hide
    for i = 1, table.getn(STOCK) do
        local name = STOCK[i]
        local f = getglobal(name)
        if f then
            if hide then
                stockState[name] = { parent = f:GetParent(), shown = f:IsShown() }
                guardOnShow(f, name)
                f:Hide()
                f:SetParent(hider)
            elseif stockState[name] then
                local was = stockState[name]
                f:SetParent(was.parent)
                -- benilla keeps a frame moved back from a hidden parent invisible; a Hide and
                -- Show recomputes it, as 1.12's SetParent does by itself.
                f:Hide()
                if was.shown then
                    f:Show()
                end
            end
        end
    end
end

function Bar.Layout()
    frame:SetScale(P.Setting("scale"))
    frame:ClearAllPoints()
    frame:SetPoint("BOTTOM", UIParent, "BOTTOM", 0, P.Setting("offsetY"))
end

function Bar.Refresh()
    if P.connected or P.Setting("alwaysShow") then
        frame:Show()
    else
        frame:Hide()
    end
    Bar.StockBar()
    if P.Xp then
        P.Xp.Refresh()
    end
    if not frame:IsShown() then
        return
    end
    layerText:SetText(P.LAYER_LABEL[P.layer] or "")
    for key, b in pairs(Bar.buttons) do
        Bar.UpdateButton(b)
    end
end

-- ── Events ──

P.Listen(function(event, arg1, arg2)
    if event == "BENILLAPAD_BUTTON" then
        local b = Bar.buttons[arg1]
        if b and frame:IsVisible() then
            Bar.UpdateState(b)
        end
    else
        Bar.Refresh()
    end
end)

local watcher = CreateFrame("Frame", nil, frame)
watcher:RegisterEvent("PLAYER_ENTERING_WORLD")
watcher:RegisterEvent("ACTIONBAR_SLOT_CHANGED")
watcher:RegisterEvent("ACTIONBAR_UPDATE_COOLDOWN")
watcher:RegisterEvent("ACTIONBAR_UPDATE_USABLE")
watcher:RegisterEvent("ACTIONBAR_UPDATE_STATE")
watcher:RegisterEvent("UPDATE_BONUS_ACTIONBAR")
watcher:RegisterEvent("PLAYER_TARGET_CHANGED")
watcher:RegisterEvent("BAG_UPDATE")
watcher:RegisterEvent("SPELL_UPDATE_COOLDOWN")
watcher:RegisterEvent("UPDATE_SHAPESHIFT_FORMS")
watcher:SetScript("OnEvent", function()
    if event == "PLAYER_ENTERING_WORLD" then
        Bar.Layout()
    end
    Bar.Refresh()
end)

-- Range, power and countdowns move without events: ten times a second.
local elapsedSince = 0
frame:SetScript("OnUpdate", function()
    elapsedSince = elapsedSince + arg1
    if elapsedSince < 0.1 then
        return
    end
    elapsedSince = 0
    for key, b in pairs(Bar.buttons) do
        Bar.UpdateState(b)
    end
end)

-- ── Cooldowns from the other layers ──
-- A row above the bar: what is cooling down (past the global cooldown) on the layers the
-- triggers do not hold, soonest first, so a trigger need not be held to check.

local ROW_MAX, ROW_ICON = 10, 26
local row = CreateFrame("Frame", nil, frame)
row:SetWidth(ROW_MAX * (ROW_ICON + 4))
row:SetHeight(ROW_ICON)
row:SetPoint("BOTTOM", frame, "TOP", 0, 2)
local rowIcons = {}
for i = 1, ROW_MAX do
    local f = CreateFrame("Frame", nil, row)
    f:SetWidth(ROW_ICON)
    f:SetHeight(ROW_ICON)
    local icon = f:CreateTexture(nil, "ARTWORK")
    icon:SetAllPoints(f)
    icon:SetTexCoord(0.07, 0.93, 0.07, 0.93)
    icon:SetVertexColor(0.6, 0.6, 0.6)
    f.icon = icon
    local text = f:CreateFontString(nil, "OVERLAY")
    text:SetFont("Fonts\\FRIZQT__.TTF", 12, "OUTLINE")
    text:SetPoint("CENTER", f, "CENTER", 0, 0)
    f.text = text
    local layer = f:CreateFontString(nil, "OVERLAY")
    layer:SetFont("Fonts\\FRIZQT__.TTF", 8, "OUTLINE")
    layer:SetPoint("BOTTOM", f, "TOP", 0, 1)
    layer:SetTextColor(1, 0.82, 0)
    f.layer = layer
    f:Hide()
    rowIcons[i] = f
end

local function cooldownLeft(start, duration, enable)
    if enable == 1 and duration and duration > 1.6 then
        return start + duration - GetTime()
    end
    return 0
end

function Bar.UpdateCooldownRow()
    local list = {}
    if P.Setting("layerCooldowns") then
        for l = 1, table.getn(P.LAYERS) do
            local layer = P.LAYERS[l]
            if layer ~= P.layer then
                for i = 1, table.getn(P.BUTTONS) do
                    local kind, value = P.Binding(P.BUTTONS[i], layer)
                    local tex, left = nil, 0
                    if kind == "slot" then
                        local slot = P.PagedSlot(value)
                        if HasAction(slot) then
                            tex = GetActionTexture(slot)
                            left = cooldownLeft(GetActionCooldown(slot))
                        end
                    elseif kind == "form" and value.spell then
                        local index = P.SpellIndex(value.spell, value.rank)
                        if index then
                            tex = value.icon
                            left = cooldownLeft(GetSpellCooldown(index, "spell"))
                        end
                    end
                    if tex and left > 0 then
                        table.insert(list, { tex = tex, left = left, layer = P.LAYER_LABEL[layer] })
                    end
                end
            end
        end
    end
    table.sort(list, function(a, b) return a.left < b.left end)
    local n = math.min(table.getn(list), ROW_MAX)
    for i = 1, ROW_MAX do
        local f = rowIcons[i]
        local e = list[i]
        if e and i <= n then
            f:ClearAllPoints()
            f:SetPoint("LEFT", row, "LEFT", (ROW_MAX - n) * (ROW_ICON + 4) / 2 + (i - 1) * (ROW_ICON + 4), 0)
            f.icon:SetTexture(e.tex)
            if e.left >= 60 then
                f.text:SetText(math.ceil(e.left / 60) .. "m")
            else
                f.text:SetText(math.ceil(e.left))
            end
            if e.layer == "" then f.layer:SetText("-") else f.layer:SetText(e.layer) end
            f:Show()
        else
            f:Hide()
        end
    end
end

local rowElapsed = 0
row:SetScript("OnUpdate", function()
    rowElapsed = rowElapsed + arg1
    if rowElapsed > 0.25 then
        rowElapsed = 0
        Bar.UpdateCooldownRow()
    end
end)

Bar.Layout()
