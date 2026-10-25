-- The on-screen keyboard: types into an edit box with the pad. D-pad moves, A types the key,
-- X deletes, Y types a space, LB shifts the next letter, RB swaps letters and symbols, Start
-- sends, B cancels. The edit box also takes a real keyboard once clicked.
--   P.Keyboard.Open(title, text, onDone)  -- onDone(text) on Start / Enter

local P = BenillaPad
local Keyboard = {}
P.Keyboard = Keyboard

local KEY = 38
local GAP = 4
local PAGES = {
    { "qwertyuiop", "asdfghjkl'", "zxcvbnm,.?" },
    { "1234567890", "!@#$%&*()-", "+=/:;\"_<>~" },
}
-- The last row's wide keys, after the character rows.
local SPECIALS = { "Shift", "123", "Space", "Del", "Enter" }
local SPECIAL_WIDTH = { Shift = 2, ["123"] = 2, Space = 4, Del = 2, Enter = 2 }

local BACKDROP = {
    bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background",
    edgeFile = "Interface\\DialogFrame\\UI-DialogBox-Border",
    tile = true, tileSize = 32, edgeSize = 32,
    insets = { left = 11, right = 12, top = 12, bottom = 11 },
}

local win = CreateFrame("Frame", "BenillaPadKeyboard", UIParent)
win:SetWidth(10 * (KEY + GAP) + 40)
win:SetHeight(4 * (KEY + GAP) + 130)
win:SetPoint("BOTTOM", UIParent, "BOTTOM", 0, 120)
win:SetFrameStrata("FULLSCREEN_DIALOG")
win:SetBackdrop(BACKDROP)
win:EnableMouse(true)
win:Hide()
Keyboard.frame = win

local title = win:CreateFontString(nil, "ARTWORK", "GameFontNormal")
title:SetPoint("TOP", win, "TOP", 0, -18)

local edit = CreateFrame("EditBox", "BenillaPadKeyboardEdit", win, "InputBoxTemplate")
edit:SetWidth(10 * (KEY + GAP) - 10)
edit:SetHeight(24)
edit:SetPoint("TOP", win, "TOP", 4, -38)
edit:SetAutoFocus(false)
edit:SetMaxLetters(255)
edit:SetScript("OnEnterPressed", function() Keyboard.Done() end)
edit:SetScript("OnEscapePressed", function() Keyboard.Cancel() end)

local hint = win:CreateFontString(nil, "ARTWORK", "GameFontDisableSmall")
hint:SetPoint("BOTTOM", win, "BOTTOM", 0, 18)
hint:SetText("A: type   X: delete   Y: space   LB: shift   RB: 123   Start: send   B: cancel")

Keyboard.page = 1
Keyboard.shift = false
Keyboard.row = 1
Keyboard.col = 1
local keys = {}

-- Rows of { label, x, width } for the current page; the specials row is last.
local function layout()
    local rows = {}
    local chars = PAGES[Keyboard.page]
    for r = 1, table.getn(chars) do
        local row = {}
        for c = 1, string.len(chars[r]) do
            table.insert(row, { char = string.sub(chars[r], c, c), units = 1 })
        end
        table.insert(rows, row)
    end
    local special = {}
    for i = 1, table.getn(SPECIALS) do
        table.insert(special, { special = SPECIALS[i], units = SPECIAL_WIDTH[SPECIALS[i]] })
    end
    table.insert(rows, special)
    return rows
end

local function keyFrame(i)
    if keys[i] then
        return keys[i]
    end
    local b = CreateFrame("Button", nil, win)
    b:SetHeight(KEY)
    local bg = b:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(b)
    b.bg = bg
    local text = b:CreateFontString(nil, "ARTWORK", "GameFontHighlight")
    text:SetPoint("CENTER", b, "CENTER", 0, 0)
    b.text = text
    b:SetScript("OnClick", function()
        Keyboard.row, Keyboard.col = this.row, this.col
        Keyboard.Press()
    end)
    keys[i] = b
    return b
end

function Keyboard.Refresh()
    local rows = layout()
    Keyboard.rows = rows
    local n = 0
    for r = 1, table.getn(rows) do
        local x = 0
        for c = 1, table.getn(rows[r]) do
            local k = rows[r][c]
            n = n + 1
            local b = keyFrame(n)
            b.row, b.col = r, c
            b:SetWidth(k.units * KEY + (k.units - 1) * GAP)
            b:ClearAllPoints()
            b:SetPoint("TOPLEFT", win, "TOPLEFT", 20 + x, -72 - (r - 1) * (KEY + GAP))
            x = x + k.units * (KEY + GAP)
            local label = k.char or k.special
            if k.char and Keyboard.shift then
                label = string.upper(label)
            end
            if k.special == "123" and Keyboard.page == 2 then
                label = "abc"
            end
            b.text:SetText(label)
            if r == Keyboard.row and c == Keyboard.col then
                b.bg:SetTexture(1, 0.75, 0.1, 0.9)
                b.text:SetTextColor(0, 0, 0)
            elseif k.special == "Shift" and Keyboard.shift then
                b.bg:SetTexture(0.3, 0.5, 0.9, 0.9)
                b.text:SetTextColor(1, 1, 1)
            else
                b.bg:SetTexture(0.15, 0.17, 0.25, 0.95)
                b.text:SetTextColor(1, 1, 1)
            end
            b:Show()
        end
    end
    for i = n + 1, table.getn(keys) do
        keys[i]:Hide()
    end
end

local function insert(text)
    edit:Insert(text)
end

local function backspace()
    local t = edit:GetText() or ""
    edit:SetText(string.sub(t, 1, string.len(t) - 1))
end

-- A on the selected key.
function Keyboard.Press()
    local k = Keyboard.rows[Keyboard.row][Keyboard.col]
    if k.char then
        local c = k.char
        if Keyboard.shift then
            c = string.upper(c)
            Keyboard.shift = false
        end
        insert(c)
    elseif k.special == "Shift" then
        Keyboard.shift = not Keyboard.shift
    elseif k.special == "123" then
        Keyboard.page = 3 - Keyboard.page
    elseif k.special == "Space" then
        insert(" ")
    elseif k.special == "Del" then
        backspace()
    elseif k.special == "Enter" then
        Keyboard.Done()
        return
    end
    Keyboard.Refresh()
end

-- Up and down keep the column's place along the row.
local function moveRow(dir)
    local rows = Keyboard.rows
    local from = rows[Keyboard.row]
    local x = 0
    for c = 1, Keyboard.col - 1 do x = x + from[c].units end
    x = x + from[Keyboard.col].units / 2
    local r = Keyboard.row + dir
    if r < 1 then r = table.getn(rows) end
    if r > table.getn(rows) then r = 1 end
    local acc, pick = 0, table.getn(rows[r])
    for c = 1, table.getn(rows[r]) do
        acc = acc + rows[r][c].units
        if acc >= x then
            pick = c
            break
        end
    end
    Keyboard.row, Keyboard.col = r, pick
end

function Keyboard.Nav(btn)
    local rows = Keyboard.rows
    if btn == "DUP" then
        moveRow(-1)
    elseif btn == "DDOWN" then
        moveRow(1)
    elseif btn == "DLEFT" then
        Keyboard.col = Keyboard.col - 1
        if Keyboard.col < 1 then Keyboard.col = table.getn(rows[Keyboard.row]) end
    elseif btn == "DRIGHT" then
        Keyboard.col = Keyboard.col + 1
        if Keyboard.col > table.getn(rows[Keyboard.row]) then Keyboard.col = 1 end
    elseif btn == "A" then
        Keyboard.Press()
        return
    elseif btn == "X" then
        backspace()
    elseif btn == "Y" then
        insert(" ")
    elseif btn == "LB" then
        Keyboard.shift = not Keyboard.shift
    elseif btn == "RB" then
        Keyboard.page = 3 - Keyboard.page
        if Keyboard.col > table.getn(layout()[Keyboard.row]) then Keyboard.col = 1 end
    elseif btn == "START" then
        Keyboard.Done()
        return
    elseif btn == "B" or btn == "SELECT" then
        Keyboard.Cancel()
        return
    end
    Keyboard.Refresh()
end

function Keyboard.Open(titleText, text, onDone)
    Keyboard.onDone = onDone
    Keyboard.page, Keyboard.shift, Keyboard.row, Keyboard.col = 1, false, 1, 1
    title:SetText(titleText or "Type")
    edit:SetText(text or "")
    win:Show()
    Keyboard.Refresh()
end

function Keyboard.IsOpen()
    return win:IsVisible()
end

function Keyboard.Done()
    local text = edit:GetText() or ""
    local done = Keyboard.onDone
    edit:ClearFocus()
    win:Hide()
    if done then
        done(text)
    end
end

function Keyboard.Cancel()
    edit:ClearFocus()
    win:Hide()
end

local before
win:SetScript("OnShow", function()
    before = P.mode
    P.mode = "keyboard"
end)
win:SetScript("OnHide", function()
    if P.mode == "keyboard" then
        P.mode = before or "world"
    end
end)

-- First in line for the pad's presses while it is up (Keyboard.lua loads before the windows).
P.Listen(function(event, a1, a2)
    if win:IsVisible() and event == "BENILLAPAD_NAV" then
        if a2 then
            Keyboard.Nav(a1)
        end
        return true
    end
end)
