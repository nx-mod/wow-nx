-- Quick Chat: a Bots tab (the roster and the command catalog, see Bots.lua) and phrase tabs for
-- Say, Party, Raid, Guild, Yell and Reply, driven by the pad (BENILLAPAD_NAV) or the mouse.
--   Every tab: LB / RB change tab, B or Select close.
--   Bots: the category list on the left, tiles on the right. A runs a tile, X adds it to the bot
--   wheel (on a character: invites it), Y changes who the commands go to, Start types a command.
--   Phrases: A sends, X edits the tile, Y types a message.

local P = BenillaPad
local Bots = P.Bots
local Chat = {}
P.Chat = Chat

local WIDTH, HEIGHT = 820, 470
local TILE_W, TILE_H, TILE_GAP = 142, 40, 6
local GRID_X, GRID_Y = 214, -110
local COLS = 4
local BACKDROP = {
    bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background",
    edgeFile = "Interface\\DialogFrame\\UI-DialogBox-Border",
    tile = true, tileSize = 32, edgeSize = 32,
    insets = { left = 11, right = 12, top = 12, bottom = 11 },
}

local TABS = {
    { title = "Bots" },
    { title = "Say", channel = "SAY" },
    { title = "Party", channel = "PARTY" },
    { title = "Raid", channel = "RAID" },
    { title = "Guild", channel = "GUILD" },
    { title = "Yell", channel = "YELL" },
    { title = "Reply", channel = "REPLY" },
}

local PHRASE_ROWS = { "Social", "Status", "Combat", "Moving", "Yours" }
local DEFAULT_PHRASES = {
    "Hi!", "Thanks!", "gg", "lol", "Bye!",
    "brb", "afk a sec", "Ready to go", "One more?", "Need a break",
    "Incoming!", "Need heals!", "Need mana", "Pull now", "Wait for me",
    "Let's go!", "Follow me", "Wait here", "On my way!", "Back off",
    "", "", "", "", "",
}
local ROW_COLOR = {
    { 0.35, 0.75, 1 }, { 0.75, 0.75, 0.75 }, { 1, 0.3, 0.25 }, { 0.4, 0.9, 0.4 }, { 1, 0.82, 0 },
}

Chat.tab = 1
Chat.focus = "list"
Chat.category = 1
Chat.tile = 1

-- The Bots tab's left list: the roster, the catalog's categories, the custom commands.
local function botCategories()
    local list = { { title = "Roster", roster = true } }
    for i = 1, table.getn(Bots.CATEGORIES) do
        table.insert(list, Bots.CATEGORIES[i])
    end
    table.insert(list, { title = "Custom", custom = true })
    return list
end

-- ── The phrases ──

local lastTell

local function phrase(i)
    local saved = BenillaPadDB and BenillaPadDB.phrases and BenillaPadDB.phrases[i]
    if saved then
        return saved
    end
    return DEFAULT_PHRASES[i]
end

local function setPhrase(i, text)
    BenillaPadDB = BenillaPadDB or {}
    BenillaPadDB.phrases = BenillaPadDB.phrases or {}
    BenillaPadDB.phrases[i] = text
end

-- Send `text` on a tab's channel; Raid falls back to Party, then Say; Reply whispers the last
-- whisper's sender.
function Chat.Send(channel, text)
    if not text or text == "" then
        return
    end
    if channel == "REPLY" then
        if not lastTell then
            UIErrorsFrame:AddMessage("Nobody has whispered you yet", 1, 0.1, 0.1, 1, 5)
            return
        end
        SendChatMessage(text, "WHISPER", nil, lastTell)
        return
    end
    if channel == "RAID" and GetNumRaidMembers() == 0 then
        channel = "PARTY"
    end
    if channel == "PARTY" and GetNumPartyMembers() == 0 then
        channel = "SAY"
    end
    SendChatMessage(text, channel)
end

-- ── The frame ──

local win = CreateFrame("Frame", "BenillaPadChat", UIParent)
win:SetWidth(WIDTH)
win:SetHeight(HEIGHT)
win:SetPoint("CENTER", UIParent, "CENTER", 0, 60)
win:SetFrameStrata("DIALOG")
win:SetBackdrop(BACKDROP)
win:EnableMouse(true)
win:Hide()
tinsert(UISpecialFrames, "BenillaPadChat")
Chat.frame = win

local title = win:CreateFontString(nil, "ARTWORK", "GameFontNormalLarge")
title:SetPoint("TOP", win, "TOP", 0, -18)
title:SetText("Quick Chat")

local close = CreateFrame("Button", nil, win, "UIPanelCloseButton")
close:SetPoint("TOPRIGHT", win, "TOPRIGHT", -6, -6)

local sendTo = win:CreateFontString(nil, "ARTWORK", "GameFontHighlight")
sendTo:SetPoint("TOPLEFT", win, "TOPLEFT", GRID_X, -84)

local hint = win:CreateFontString(nil, "ARTWORK", "GameFontDisableSmall")
hint:SetPoint("BOTTOM", win, "BOTTOM", 0, 18)
hint:SetWidth(WIDTH - 40)

local tabs = {}
for i = 1, table.getn(TABS) do
    local t = CreateFrame("Button", nil, win)
    t:SetWidth(100)
    t:SetHeight(22)
    t:SetPoint("TOPLEFT", win, "TOPLEFT", 24 + (i - 1) * 108, -46)
    local bg = t:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(t)
    t.bg = bg
    local text = t:CreateFontString(nil, "ARTWORK", "GameFontNormal")
    text:SetPoint("CENTER", t, "CENTER", 0, 0)
    text:SetText(TABS[i].title)
    t.text = text
    t.index = i
    t:SetScript("OnClick", function()
        Chat.SetTab(this.index)
    end)
    tabs[i] = t
end

local listRows = {}
for i = 1, 10 do
    local r = CreateFrame("Button", nil, win)
    r:SetWidth(170)
    r:SetHeight(26)
    r:SetPoint("TOPLEFT", win, "TOPLEFT", 24, -84 - (i - 1) * 30)
    local bg = r:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(r)
    r.bg = bg
    local text = r:CreateFontString(nil, "ARTWORK", "GameFontHighlight")
    text:SetPoint("LEFT", r, "LEFT", 10, 0)
    r.text = text
    r.index = i
    r:SetScript("OnClick", function()
        Chat.category = this.index
        Chat.focus = "grid"
        Chat.tile = 1
        Chat.Refresh()
    end)
    listRows[i] = r
end

local tiles = {}
local function tileFrame(i)
    if tiles[i] then
        return tiles[i]
    end
    local t = CreateFrame("Button", nil, win)
    t:SetWidth(TILE_W)
    t:SetHeight(TILE_H)
    local bg = t:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(t)
    t.bg = bg
    local stripe = t:CreateTexture(nil, "ARTWORK")
    stripe:SetWidth(3)
    stripe:SetPoint("TOPLEFT", t, "TOPLEFT", 0, 0)
    stripe:SetPoint("BOTTOMLEFT", t, "BOTTOMLEFT", 0, 0)
    t.stripe = stripe
    local text = t:CreateFontString(nil, "ARTWORK", "GameFontHighlight")
    text:SetPoint("CENTER", t, "CENTER", 0, 5)
    text:SetWidth(TILE_W - 10)
    t.text = text
    local sub = t:CreateFontString(nil, "ARTWORK", "GameFontDisableSmall")
    sub:SetPoint("CENTER", t, "CENTER", 0, -9)
    t.sub = sub
    local star = t:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    star:SetPoint("TOPRIGHT", t, "TOPRIGHT", -4, -3)
    t.star = star
    t.index = i
    t:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    t:SetScript("OnClick", function()
        Chat.focus = "grid"
        Chat.tile = this.index
        if arg1 == "RightButton" then
            Chat.Nav("X")
        else
            Chat.Nav("A")
        end
    end)
    tiles[i] = t
    return t
end

-- ── What the current view shows ──

-- The tiles of the current view: { label, sub, color, kind, ... }.
function Chat.Tiles()
    local out = {}
    local tab = TABS[Chat.tab]
    if tab.channel then
        for i = 1, 25 do
            local text = phrase(i)
            local row = math.floor((i - 1) / 5) + 1
            table.insert(out, {
                label = (text ~= "" and text) or "+ Add your own",
                sub = PHRASE_ROWS[row], color = ROW_COLOR[row],
                kind = "phrase", index = i, empty = (text == ""),
            })
        end
        return out
    end
    local cat = botCategories()[Chat.category]
    if cat.roster then
        table.insert(out, { label = "Refresh", sub = ".bot list", kind = "do", run = Bots.Refresh })
        table.insert(out, { label = "Add all", sub = "log every bot in", kind = "do", run = Bots.AddAll })
        table.insert(out, { label = "Remove all", sub = "log every bot out", kind = "do", run = Bots.RemoveAll })
        table.insert(out, { label = "Add by name...", sub = "type a name", kind = "do",
            run = function()
                P.Keyboard.Open("Character to log in", "", function(text)
                    if text ~= "" then Bots.Add(text) end
                end)
            end })
        local roster = Bots.Roster()
        for i = 1, table.getn(roster) do
            local bot = roster[i]
            local color = { 0.5, 0.5, 0.5 }
            local state = "offline"
            if bot.online then
                color = { 0.3, 1, 0.3 }
                state = "online"
            end
            table.insert(out, { label = bot.name, sub = (bot.class or "") .. "  " .. state,
                color = color, kind = "bot", bot = bot })
        end
        return out
    end
    local commands = cat.commands
    if cat.custom then
        commands = Bots.Custom()
    end
    for i = 1, table.getn(commands) do
        local c = commands[i]
        local sub = c.cmd
        if c.ask then sub = c.cmd .. " ..." end
        table.insert(out, { label = c.label, sub = sub, kind = "command", command = c,
            favorite = Bots.IsFavorite(c.id) })
    end
    if cat.custom then
        table.insert(out, { label = "+ New command", sub = "type it once, keep it", kind = "do",
            run = function()
                P.Keyboard.Open("New bot command", "", function(text)
                    if text ~= "" then
                        Bots.AddCustom(text)
                        Chat.Refresh()
                    end
                end)
            end })
    end
    return out
end

function Chat.Refresh()
    if not win:IsVisible() then
        return
    end
    for i = 1, table.getn(tabs) do
        if i == Chat.tab then
            tabs[i].bg:SetTexture(1, 0.75, 0.1, 0.9)
            tabs[i].text:SetTextColor(0, 0, 0)
        else
            tabs[i].bg:SetTexture(0.1, 0.1, 0.15, 0.9)
            tabs[i].text:SetTextColor(1, 0.82, 0)
        end
    end
    local tab = TABS[Chat.tab]
    local bots = not tab.channel
    -- The left list, on the Bots tab only.
    local cats = botCategories()
    for i = 1, table.getn(listRows) do
        local r = listRows[i]
        if bots and cats[i] then
            r.text:SetText(cats[i].title)
            if i == Chat.category then
                if Chat.focus == "list" then
                    r.bg:SetTexture(1, 0.75, 0.1, 0.9)
                    r.text:SetTextColor(0, 0, 0)
                else
                    r.bg:SetTexture(1, 0.75, 0.1, 0.35)
                    r.text:SetTextColor(1, 1, 1)
                end
            else
                r.bg:SetTexture(0.1, 0.1, 0.15, 0.6)
                r.text:SetTextColor(1, 1, 1)
            end
            r:Show()
        else
            r:Hide()
        end
    end
    local list = Chat.Tiles()
    Chat.tiles = list
    local cols, x0 = COLS, GRID_X
    if not bots then
        cols, x0 = 5, 24
    end
    Chat.cols = cols
    if Chat.tile > table.getn(list) then Chat.tile = table.getn(list) end
    if Chat.tile < 1 then Chat.tile = 1 end
    for i = 1, table.getn(list) do
        local e = list[i]
        local t = tileFrame(i)
        local col = math.mod(i - 1, cols)
        local row = math.floor((i - 1) / cols)
        t:ClearAllPoints()
        t:SetPoint("TOPLEFT", win, "TOPLEFT", x0 + col * (TILE_W + TILE_GAP), GRID_Y - row * (TILE_H + TILE_GAP))
        t.text:SetText(e.label)
        t.sub:SetText(e.sub or "")
        local selected = i == Chat.tile and (Chat.focus == "grid" or not bots)
        if selected then
            t.bg:SetTexture(1, 0.75, 0.1, 0.35)
        else
            t.bg:SetTexture(0.08, 0.08, 0.12, 0.85)
        end
        local c = e.color or { 1, 0.82, 0 }
        t.stripe:SetTexture(c[1], c[2], c[3], 1)
        if e.empty then t.text:SetTextColor(0.6, 0.6, 0.6) else t.text:SetTextColor(1, 1, 1) end
        if e.favorite then t.star:SetText("*") else t.star:SetText("") end
        t:Show()
    end
    for i = table.getn(list) + 1, table.getn(tiles) do
        tiles[i]:Hide()
    end
    if bots then
        sendTo:SetText("Commands go to: |cffffd100" .. Bots.TargetLabel() .. "|r   (Y: change)")
        sendTo:Show()
        if Chat.focus == "list" then
            hint:SetText("D-pad: choose a category   A / Right: open it   LB / RB: tab   B: close")
        elseif botCategories()[Chat.category].roster then
            hint:SetText("A: log in / out (or run)   X: invite   Y: send to   LB / RB: tab   B: back")
        else
            hint:SetText("A: send   X: add to / remove from the bot wheel (LT+RT+Y)   Y: send to   Start: type a command   B: back")
        end
    else
        sendTo:Hide()
        hint:SetText("A: send   X: edit tile   Y: type a message   LB / RB: tab   B: close")
    end
end

-- ── Acting ──

local function activate(e)
    if not e then
        return
    end
    local tab = TABS[Chat.tab]
    if e.kind == "phrase" then
        if e.empty then
            Chat.EditPhrase(e.index)
        else
            Chat.Send(tab.channel, phrase(e.index))
        end
    elseif e.kind == "do" then
        e.run()
    elseif e.kind == "bot" then
        if e.bot.online then Bots.Remove(e.bot.name) else Bots.Add(e.bot.name) end
    elseif e.kind == "command" then
        Bots.Run(e.command)
    end
end

function Chat.EditPhrase(i)
    P.Keyboard.Open("Edit tile", phrase(i), function(text)
        setPhrase(i, text)
        Chat.Refresh()
    end)
end

function Chat.SetTab(i)
    Chat.tab = i
    Chat.tile = 1
    if TABS[i].channel then Chat.focus = "grid" else Chat.focus = "list" end
    Chat.Refresh()
end

function Chat.Nav(btn)
    local tab = TABS[Chat.tab]
    local bots = not tab.channel
    local n = table.getn(Chat.tiles or {})
    if btn == "LB" or btn == "RB" then
        local i = Chat.tab + ((btn == "LB") and -1 or 1)
        if i < 1 then i = table.getn(TABS) end
        if i > table.getn(TABS) then i = 1 end
        Chat.SetTab(i)
        return
    elseif btn == "SELECT" then
        win:Hide()
        return
    end
    if bots and Chat.focus == "list" then
        local cats = table.getn(botCategories())
        if btn == "DUP" then
            Chat.category = Chat.category - 1
            if Chat.category < 1 then Chat.category = cats end
        elseif btn == "DDOWN" then
            Chat.category = Chat.category + 1
            if Chat.category > cats then Chat.category = 1 end
        elseif btn == "DRIGHT" or btn == "A" then
            Chat.focus = "grid"
            Chat.tile = 1
        elseif btn == "Y" then
            Bots.CycleTarget(1)
        elseif btn == "B" then
            win:Hide()
            return
        end
        Chat.Refresh()
        return
    end
    local cols = Chat.cols or COLS
    local e = Chat.tiles and Chat.tiles[Chat.tile]
    if btn == "DUP" then
        if Chat.tile > cols then Chat.tile = Chat.tile - cols end
    elseif btn == "DDOWN" then
        if Chat.tile + cols <= n then Chat.tile = Chat.tile + cols end
    elseif btn == "DLEFT" then
        if math.mod(Chat.tile - 1, cols) == 0 then
            if bots then Chat.focus = "list" end
        else
            Chat.tile = Chat.tile - 1
        end
    elseif btn == "DRIGHT" then
        if math.mod(Chat.tile - 1, cols) < cols - 1 and Chat.tile < n then
            Chat.tile = Chat.tile + 1
        end
    elseif btn == "A" then
        activate(e)
    elseif btn == "X" then
        if e and e.kind == "phrase" then
            Chat.EditPhrase(e.index)
        elseif e and e.kind == "bot" then
            Bots.Invite(e.bot.name)
        elseif e and e.kind == "command" then
            Bots.ToggleFavorite(e.command.id)
        end
    elseif btn == "Y" then
        if bots then
            Bots.CycleTarget(1)
        else
            P.Keyboard.Open("Message (" .. tab.title .. ")", "", function(text)
                Chat.Send(tab.channel, text)
            end)
        end
    elseif btn == "START" then
        if bots then
            P.Keyboard.Open("Bot command (to " .. Bots.TargetLabel() .. ")", "", function(text)
                Bots.Say(text)
            end)
        end
    elseif btn == "B" then
        if bots then
            Chat.focus = "list"
        else
            win:Hide()
            return
        end
    end
    Chat.Refresh()
end

function Chat.Open()
    if P.Menu and P.Menu.frame:IsVisible() then P.Menu.Close() end
    if P.Binds and P.Binds.frame:IsVisible() then P.Binds.frame:Hide() end
    if P.Wheel and P.Wheel.frame:IsVisible() then P.Wheel.Close() end
    win:Show()
end

function Chat.Toggle()
    if win:IsVisible() then
        win:Hide()
    else
        Chat.Open()
    end
end

win:SetScript("OnShow", function()
    P.mode = "menu"
    Chat.Refresh()
end)
win:SetScript("OnHide", function()
    if P.mode == "menu" then
        P.mode = "world"
    end
end)

P.Listen(function(event, a1, a2)
    if not win:IsVisible() then
        return
    end
    if event == "BENILLAPAD_NAV" then
        if a2 then
            Chat.Nav(a1)
        end
        return true
    end
end)

local watch = CreateFrame("Frame", nil, win)
watch:RegisterEvent("PLAYER_TARGET_CHANGED")
watch:SetScript("OnEvent", function() Chat.Refresh() end)

-- The last whisper's sender, for the Reply tab.
local tells = CreateFrame("Frame")
tells:RegisterEvent("CHAT_MSG_WHISPER")
tells:SetScript("OnEvent", function() lastTell = arg2 end)
