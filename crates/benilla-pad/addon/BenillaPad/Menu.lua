-- The controller menu, on Select: pages of rows driven by the pad (BENILLAPAD_NAV while it is up,
-- the Rust side's "menu" mode) or the mouse. D-pad up/down picks a row, left/right changes its
-- value, A opens or toggles, B goes back a page or closes.

local P = BenillaPad
local Menu = {}
P.Menu = Menu

local WIDTH = 440
local ROW = 28
local BACKDROP = {
    bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background",
    edgeFile = "Interface\\DialogFrame\\UI-DialogBox-Border",
    tile = true, tileSize = 32, edgeSize = 32,
    insets = { left = 11, right = 12, top = 12, bottom = 11 },
}

-- ── Row kinds ──
-- link:   { label, desc, open = page } or { label, desc, run = function }
-- toggle: { label, desc, key }
-- choice: { label, desc, key, values = { {value, text}, ... } }
-- number: { label, desc, key, min, max, step, fmt }

local function percent(v) return math.floor(v * 100 + 0.5) .. "%" end
local function plain(v) return string.format("%.1f", v) end
local function whole(v) return tostring(math.floor(v + 0.5)) end

Menu.pages = {
    main = {
        title = "Controller",
        rows = {
            { label = "Keybinds", desc = "Put spells, macros, items and game actions on every button and layer.",
                run = function() Menu.Close() P.Binds.Toggle() end },
            { label = "Quick Chat", desc = "Phrases for every channel, the on-screen keyboard, and your bots: log them in, command them, build the bot wheel (LT+RT+Y).",
                run = function() Menu.Close() P.Chat.Open() end },
            { label = "Camera & sticks", desc = "Look speed, camera direction, stick dead zone, zoom.", open = "camera" },
            { label = "Buttons", desc = "When spells cast, and what A / B / X / Y do with no trigger held.", open = "buttons" },
            { label = "Look", desc = "Size and place of the gamepad bar, button symbols, the stock action bar.", open = "look" },
            { label = "Reset all settings", desc = "Every setting on this menu back to its default. Your binds are kept.",
                run = function()
                    if Menu.confirm then
                        Menu.confirm = nil
                        P.ResetSettings()
                        P.Print("settings reset")
                    else
                        Menu.confirm = true
                    end
                end },
            { label = "Close", desc = "Close the menu (B or Select also close it).", run = function() Menu.Close() end },
        },
    },
    camera = {
        title = "Camera & sticks",
        rows = {
            { label = "Look speed", desc = "How fast the right stick turns the camera at full tilt.",
                key = "lookSpeed", min = 1, max = 8, step = 0.5, fmt = plain },
            { label = "Invert camera up / down", desc = "Push the right stick up to look down.", key = "invertY" },
            { label = "Stick dead zone", desc = "How far a stick moves before it counts.",
                key = "deadzone", min = 0.1, max = 0.5, step = 0.05, fmt = percent },
            { label = "Zoom with LB / RB", desc = "Hold LB or RB and push the right stick up or down to zoom.",
                key = "shoulderZoom" },
            { label = "Pad cursor in windows", desc = "Open game windows take the right stick as a mouse cursor: A clicks, X right-clicks, B closes, D-pad jumps between buttons.",
                key = "padCursor" },
            { label = "Cursor speed", desc = "How fast the right stick moves the cursor at full tilt.",
                key = "cursorSpeed", min = 400, max = 2400, step = 100, fmt = whole },
        },
    },
    buttons = {
        title = "Buttons",
        rows = {
            { label = "Cast spells on press", desc = "On: a spell casts when the button goes down. Off: when it comes up, as 1.12's keys do.",
                key = "castOnPress" },
            { label = "Interact loots everything", desc = "X on a corpse takes all its loot at once, as a Shift-click does.",
                key = "interactLootAll" },
            { label = "A / B / X / Y with no trigger",
                desc = "Game actions: Jump, Interact, Back, Inspect. Spells: their own action slots, like the D-pad.",
                key = "faceSpells", values = { { false, "Game actions" }, { true, "Spells" } } },
        },
    },
    look = {
        title = "Look",
        rows = {
            { label = "Bar size", desc = "The gamepad bar's scale.", key = "scale", min = 0.6, max = 1.6, step = 0.1, fmt = percent },
            { label = "Bar height", desc = "How far above the bottom of the screen the bar sits.",
                key = "offsetY", min = 40, max = 500, step = 10, fmt = whole },
            { label = "Button symbols", desc = "Auto follows the connected controller.", key = "symbols",
                values = { { "auto", "Auto" }, { "xbox", "Xbox" }, { "playstation", "PlayStation" } } },
            { label = "Always show the bar", desc = "Show the gamepad bar even with no controller connected.", key = "alwaysShow" },
            { label = "Hide the stock bars", desc = "While the gamepad bar shows, the stock action bars, bags, XP bar and micro menu go (the window wheel opens everything). The pet bar stays.",
                key = "hideStockBars" },
            { label = "Experience bar", desc = "A thin experience bar along the bottom of the screen, with the rested bonus shown ahead of it.",
                key = "xpBar" },
            { label = "Experience numbers", desc = "Level, experience and rested bonus as text over the bar. Off: shown only under the mouse pointer.",
                key = "xpText" },
            { label = "Cooldowns from other layers", desc = "A row above the bar with what is cooling down on the layers you are not holding.",
                key = "layerCooldowns" },
        },
    },
}

Menu.page = "main"
Menu.row = 1
Menu.history = {}

-- ── The frame ──

local win = CreateFrame("Frame", "BenillaPadMenu", UIParent)
win:SetWidth(WIDTH)
win:SetFrameStrata("DIALOG")
win:SetBackdrop(BACKDROP)
win:SetPoint("LEFT", UIParent, "LEFT", 40, 60)
win:EnableMouse(true)
win:Hide()
tinsert(UISpecialFrames, "BenillaPadMenu")
Menu.frame = win

local title = win:CreateFontString(nil, "ARTWORK", "GameFontNormalLarge")
title:SetPoint("TOP", win, "TOP", 0, -20)

local desc = win:CreateFontString(nil, "ARTWORK", "GameFontHighlightSmall")
desc:SetWidth(WIDTH - 50)
desc:SetJustifyH("LEFT")

local hint = win:CreateFontString(nil, "ARTWORK", "GameFontDisableSmall")
hint:SetPoint("BOTTOM", win, "BOTTOM", 0, 20)
hint:SetText("D-pad: move    Left / Right: change    A: choose    B: back")

local rows = {}
for i = 1, 10 do
    local r = CreateFrame("Button", nil, win)
    r:SetWidth(WIDTH - 40)
    r:SetHeight(ROW - 2)
    r:SetPoint("TOPLEFT", win, "TOPLEFT", 20, -50 - (i - 1) * ROW)
    local hi = r:CreateTexture(nil, "BACKGROUND")
    hi:SetAllPoints(r)
    hi:SetTexture(1, 0.75, 0.1, 0.22)
    r.hi = hi
    local bar = r:CreateTexture(nil, "ARTWORK")
    bar:SetWidth(3)
    bar:SetPoint("TOPLEFT", r, "TOPLEFT", 0, 0)
    bar:SetPoint("BOTTOMLEFT", r, "BOTTOMLEFT", 0, 0)
    bar:SetTexture(1, 0.82, 0)
    r.bar = bar
    local label = r:CreateFontString(nil, "ARTWORK", "GameFontHighlight")
    label:SetPoint("LEFT", r, "LEFT", 12, 0)
    r.label = label
    local value = r:CreateFontString(nil, "ARTWORK", "GameFontNormal")
    value:SetPoint("RIGHT", r, "RIGHT", -8, 0)
    r.value = value
    r.index = i
    r:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    r:SetScript("OnClick", function()
        Menu.row = this.index
        if arg1 == "RightButton" then
            Menu.Adjust(-1)
        else
            Menu.Activate()
        end
        Menu.Refresh()
    end)
    r:SetScript("OnEnter", function()
        Menu.row = this.index
        Menu.Refresh()
    end)
    rows[i] = r
end

local function valueText(row)
    if row.open then
        return "|cffffd100>|r"
    end
    if row.run then
        if row.label == "Reset all settings" and Menu.confirm then
            return "|cffff4040Press A again|r"
        end
        return ""
    end
    local v = P.Setting(row.key)
    if row.values then
        for i = 1, table.getn(row.values) do
            if row.values[i][1] == v then
                return "|cffffd100< " .. row.values[i][2] .. " >|r"
            end
        end
        return "?"
    end
    if row.min then
        return "|cffffd100< " .. row.fmt(v) .. " >|r"
    end
    if v then
        return "|cff40ff40On|r"
    end
    return "|cffff4040Off|r"
end

function Menu.Refresh()
    if not win:IsVisible() then
        return
    end
    local page = Menu.pages[Menu.page]
    title:SetText(page.title)
    local n = table.getn(page.rows)
    if Menu.row > n then Menu.row = n end
    if Menu.row < 1 then Menu.row = 1 end
    for i = 1, table.getn(rows) do
        local r = rows[i]
        local row = page.rows[i]
        if row then
            r.label:SetText(row.label)
            r.value:SetText(valueText(row))
            if i == Menu.row then
                r.hi:Show()
                r.bar:Show()
            else
                r.hi:Hide()
                r.bar:Hide()
            end
            r:Show()
        else
            r:Hide()
        end
    end
    local selected = page.rows[Menu.row]
    desc:ClearAllPoints()
    desc:SetPoint("TOPLEFT", win, "TOPLEFT", 26, -58 - n * ROW)
    desc:SetText(selected and selected.desc or "")
    win:SetHeight(120 + n * ROW)
end

-- Change the selected row's value by `dir` steps.
function Menu.Adjust(dir)
    local row = Menu.pages[Menu.page].rows[Menu.row]
    if not row or not row.key then
        return
    end
    local v = P.Setting(row.key)
    if row.values then
        local at = 1
        for i = 1, table.getn(row.values) do
            if row.values[i][1] == v then at = i end
        end
        at = at + dir
        local n = table.getn(row.values)
        if at < 1 then at = n end
        if at > n then at = 1 end
        P.SetSetting(row.key, row.values[at][1])
    elseif row.min then
        local nv = v + dir * row.step
        if nv < row.min then nv = row.min end
        if nv > row.max then nv = row.max end
        -- Round to the step, so repeated steps do not drift.
        nv = math.floor(nv / row.step + 0.5) * row.step
        P.SetSetting(row.key, nv)
    else
        P.SetSetting(row.key, not v)
    end
end

-- A on the selected row.
function Menu.Activate()
    local row = Menu.pages[Menu.page].rows[Menu.row]
    if not row then
        return
    end
    if row.label ~= "Reset all settings" then
        Menu.confirm = nil
    end
    if row.open then
        table.insert(Menu.history, { page = Menu.page, row = Menu.row })
        Menu.page = row.open
        Menu.row = 1
    elseif row.run then
        row.run()
    elseif row.min then
        Menu.Adjust(1)
    else
        Menu.Adjust(1)
    end
end

function Menu.Back()
    Menu.confirm = nil
    local n = table.getn(Menu.history)
    if n == 0 then
        Menu.Close()
        return
    end
    local last = Menu.history[n]
    table.remove(Menu.history, n)
    Menu.page = last.page
    Menu.row = last.row
end

function Menu.Nav(btn)
    if btn == "DUP" then
        Menu.row = Menu.row - 1
        Menu.confirm = nil
    elseif btn == "DDOWN" then
        Menu.row = Menu.row + 1
        Menu.confirm = nil
    elseif btn == "DLEFT" then
        Menu.Adjust(-1)
    elseif btn == "DRIGHT" then
        Menu.Adjust(1)
    elseif btn == "A" then
        Menu.Activate()
    elseif btn == "B" then
        Menu.Back()
    elseif btn == "SELECT" or btn == "START" then
        Menu.Close()
    end
    Menu.Refresh()
end

function Menu.Open()
    if P.Binds and P.Binds.frame:IsVisible() then
        P.Binds.frame:Hide()
    end
    Menu.page = "main"
    Menu.row = 1
    Menu.history = {}
    Menu.confirm = nil
    win:Show()
end

function Menu.Close()
    win:Hide()
end

function Menu.Toggle()
    if win:IsVisible() then
        Menu.Close()
    else
        Menu.Open()
    end
end

win:SetScript("OnShow", function()
    P.mode = "menu"
    Menu.Refresh()
end)
win:SetScript("OnHide", function()
    Menu.confirm = nil
    -- The keybind window may have taken over the pad on its way out.
    if not (P.Binds and P.Binds.frame:IsVisible()) then
        P.mode = "world"
    end
end)

P.Listen(function(event, arg1, arg2)
    if not win:IsVisible() then
        return
    end
    if event == "BENILLAPAD_NAV" then
        if arg2 then
            Menu.Nav(arg1)
        end
        return true
    end
end)

-- A setting changed elsewhere (slash command, reset) shows here too.
local oldChanged = P.Changed
P.Changed = function()
    oldChanged()
    Menu.Refresh()
end
