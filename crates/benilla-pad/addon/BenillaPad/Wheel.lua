-- The wheels: a ring of round buttons the sticks point at. The window wheel (Start, LT+RT+A)
-- opens game windows and has an emote page; the consumables wheel (LT+RT+X) uses a potion, food,
-- bandage, scroll or weapon oil from the bags. While a wheel is up the Rust side is in "wheel"
-- mode: the sticks arrive as BENILLAPAD_STICK, the buttons as BENILLAPAD_NAV, and the body stays.
-- The quest item (LT+RT+B) uses the bags' usable quest item straight away.

local P = BenillaPad
local Wheel = {}
P.Wheel = Wheel

local SIZE = 600
local RADIUS = 175
local SLOT = 58
local LABEL_RADIUS = 238
-- Stick lengths: past PICK a direction selects, under RELEASE the stick is at rest.
local PICK = 0.5
local RELEASE = 0.25
local MAX_ENTRIES = 12
local ICON = "Interface\\Icons\\"

-- ── The entries ──

local function runBinding(command)
    return function() RunBinding(command) end
end

local function emote(token)
    return function() DoEmote(token) end
end

-- The first trade skill with a window, cast from the spellbook.
local PROFESSIONS = { "Alchemy", "Blacksmithing", "Enchanting", "Engineering", "Leatherworking",
    "Tailoring", "Cooking", "First Aid", "Smelting", "Poisons" }

local function professionIndex()
    local _, _, offset, count = GetSpellTabInfo(1)
    for i = 1, (offset or 0) + (count or 0) do
        local name = GetSpellName(i, "spell")
        for j = 1, table.getn(PROFESSIONS) do
            if name == PROFESSIONS[j] then
                return i
            end
        end
    end
    return nil
end

local WINDOW_PAGES = {
    {
        title = "Windows",
        entries = {
            { label = "Character", icon = P.ART .. "Character", run = runBinding("TOGGLECHARACTER0") },
            { label = "Bags", icon = P.ART .. "Bags", run = runBinding("OPENALLBAGS"),
                sub = function()
                    local free = 0
                    for bag = 0, 4 do
                        for slot = 1, GetContainerNumSlots(bag) do
                            if not GetContainerItemLink(bag, slot) then free = free + 1 end
                        end
                    end
                    return free .. " free"
                end },
            { label = "Spellbook", icon = P.ART .. "Spellbook", run = runBinding("TOGGLESPELLBOOK") },
            { label = "Talents", icon = P.ART .. "Talents", run = runBinding("TOGGLETALENTS") },
            { label = "Quest Log", icon = P.ART .. "QuestLog", run = runBinding("TOGGLEQUESTLOG") },
            { label = "World Map", icon = P.ART .. "Map", run = runBinding("TOGGLEWORLDMAP") },
            { label = "Social", icon = P.ART .. "Social", run = runBinding("TOGGLESOCIAL") },
            { label = "Professions", icon = P.ART .. "Professions",
                run = function()
                    local i = professionIndex()
                    if i then
                        CastSpell(i, "spell")
                    else
                        UIErrorsFrame:AddMessage("No profession with a window", 1, 0.1, 0.1, 1, 5)
                    end
                end },
            { label = "Controller", icon = P.ART .. "Menu", run = function() P.Menu.Open() end },
            { label = "Quick Chat", icon = P.ART .. "QuickChat", run = function() P.Chat.Open() end },
            { label = "Game Menu", icon = P.ART .. "GameMenu", run = runBinding("TOGGLEGAMEMENU") },
        },
    },
    {
        title = "Emotes",
        entries = {
            { label = "Sit / Stand", icon = P.ART .. "Sit", run = function() SitOrStand() end },
            { label = "Wave", icon = P.ART .. "Emote", run = emote("WAVE") },
            { label = "Dance", icon = P.ART .. "Emote", run = emote("DANCE") },
            { label = "Cheer", icon = P.ART .. "Emote", run = emote("CHEER") },
            { label = "Bow", icon = P.ART .. "Emote", run = emote("BOW") },
            { label = "Thank", icon = P.ART .. "Emote", run = emote("THANK") },
            { label = "Laugh", icon = P.ART .. "Emote", run = emote("LAUGH") },
            { label = "Point", icon = P.ART .. "Emote", run = emote("POINT") },
        },
    },
}

-- Weapon imbues go on the main hand after the use: 1.12 asks for the item with the targeting
-- cursor, which a click on the main-hand slot (inventory slot 16) answers.
local IMBUE_WORDS = { "Sharpening Stone", "Weightstone", "Oil", "Poison" }

local function isImbue(name)
    for i = 1, table.getn(IMBUE_WORDS) do
        if string.find(name, IMBUE_WORDS[i], 1, true) then
            return true
        end
    end
    return false
end

-- The bags' consumables, one entry per item, with the stack counts summed.
local function consumableEntries()
    local _, _, _, consumable = GetAuctionItemClasses()
    consumable = consumable or "Consumable"
    local byId, list = {}, {}
    for bag = 0, 4 do
        for slot = 1, GetContainerNumSlots(bag) do
            local link = GetContainerItemLink(bag, slot)
            local _, _, id = string.find(link or "", "item:(%d+)")
            if id then
                local name, _, _, _, itemType, subType, _, _, texture = GetItemInfo(tonumber(id))
                if name and itemType == consumable then
                    local _, count = GetContainerItemInfo(bag, slot)
                    local e = byId[id]
                    if e then
                        e.count = e.count + (count or 1)
                    else
                        e = { label = name, icon = texture, count = count or 1, bag = bag, slot = slot,
                            kind = subType or "", imbue = isImbue(name) }
                        byId[id] = e
                        table.insert(list, e)
                    end
                end
            end
        end
    end
    table.sort(list, function(a, b)
        if a.kind ~= b.kind then return a.kind < b.kind end
        return a.label < b.label
    end)
    while table.getn(list) > MAX_ENTRIES do
        table.remove(list)
    end
    for i = 1, table.getn(list) do
        local e = list[i]
        local count = e.count
        e.sub = function() return "x" .. count end
        e.cooldown = function() return GetContainerItemCooldown(e.bag, e.slot) end
        e.run = function()
            UseContainerItem(e.bag, e.slot)
            if e.imbue and SpellIsTargeting() then
                PickupInventoryItem(16)
            end
        end
    end
    return list
end

-- The bot wheel's entries: the favourite bot commands (Bots.lua), each with an icon.
local BOT_ICONS = {
    follow = "BotFollow", stay = "BotStay", guard = "BotGuard", flee = "BotFlee",
    summon = "BotSummon", attack = "Attack", tankattack = "BotTank", pull = "BotPull",
    maxdps = "BotMaxDps", savemana = "BotMana", acceptall = "BotAccept", autoaccept = "BotAccept",
    quests = "QuestLog", talk = "QuickChat", loot = "BotLoot", addallloot = "BotLoot",
    inventory = "Bags", release = "BotRelease", revive = "BotRevive", selfres = "BotRevive",
    ready = "BotAccept", cast = "Spellbook", whoami = "Character", stats = "Character",
    where = "Map", home = "Map", trainer = "Talents", repair = "Professions",
}

local function botEntries()
    local list = {}
    local favorites = P.Bots.Favorites()
    for i = 1, table.getn(favorites) do
        local c = P.Bots.Command(favorites[i])
        if c then
            local icon = P.ART .. (BOT_ICONS[c.id] or "BotOther")
            table.insert(list, { label = c.label, icon = icon, run = function() P.Bots.Run(c) end })
        end
    end
    return list
end

-- ── The frame ──

local win = CreateFrame("Frame", "BenillaPadWheel", UIParent)
win:SetWidth(SIZE)
win:SetHeight(SIZE)
win:SetPoint("CENTER", UIParent, "CENTER", 0, 30)
win:SetFrameStrata("DIALOG")
win:EnableMouse(true)
win:Hide()
tinsert(UISpecialFrames, "BenillaPadWheel")
Wheel.frame = win

local disc = win:CreateTexture(nil, "BACKGROUND")
disc:SetTexture(P.ART .. "Disc")
disc:SetVertexColor(0, 0, 0, 0.55)
disc:SetWidth(RADIUS * 2 + SLOT + 30)
disc:SetHeight(RADIUS * 2 + SLOT + 30)
disc:SetPoint("CENTER", win, "CENTER", 0, 0)

local hub = win:CreateTexture(nil, "BORDER")
hub:SetTexture(P.ART .. "Disc")
hub:SetVertexColor(0.08, 0.06, 0.02, 0.85)
hub:SetWidth(RADIUS * 2 - SLOT - 20)
hub:SetHeight(RADIUS * 2 - SLOT - 20)
hub:SetPoint("CENTER", win, "CENTER", 0, 0)

local pageTitle = win:CreateFontString(nil, "OVERLAY")
pageTitle:SetFont("Fonts\\FRIZQT__.TTF", 18, "OUTLINE")
pageTitle:SetPoint("CENTER", win, "CENTER", 0, 54)
pageTitle:SetTextColor(1, 0.82, 0)

local chosen = win:CreateFontString(nil, "OVERLAY")
chosen:SetFont("Fonts\\FRIZQT__.TTF", 16, "OUTLINE")
chosen:SetPoint("CENTER", win, "CENTER", 0, -30)

local hint = win:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
hint:SetPoint("CENTER", win, "CENTER", 0, -56)

-- The pointer: the minimap's player arrow, turned toward the stick with an 8-point texcoord.
local arrow = win:CreateTexture(nil, "OVERLAY")
arrow:SetTexture("Interface\\Minimap\\MinimapArrow")
arrow:SetWidth(44)
arrow:SetHeight(44)
arrow:Hide()

local function pointArrow(degrees)
    local a = math.rad(degrees)
    local c, s = math.cos(a), math.sin(a)
    -- The texel shown at screen offset (x, y) comes from (x, y) turned back by the angle; the
    -- 0.707 keeps every corner inside the texture.
    local function uv(x, y)
        local k = 0.7071
        local sx = (x * c - y * s) * k
        local sy = (x * s + y * c) * k
        return 0.5 + sx, 0.5 - sy
    end
    local ulx, uly = uv(-0.5, 0.5)
    local llx, lly = uv(-0.5, -0.5)
    local urx, ury = uv(0.5, 0.5)
    local lrx, lry = uv(0.5, -0.5)
    arrow:SetTexCoord(ulx, uly, llx, lly, urx, ury, lrx, lry)
    arrow:ClearAllPoints()
    arrow:SetPoint("CENTER", win, "CENTER", math.sin(a) * 70, math.cos(a) * 70)
    arrow:Show()
end

local dots = {}
for i = 1, 4 do
    local d = win:CreateTexture(nil, "OVERLAY")
    d:SetTexture(P.ART .. "Disc")
    d:SetWidth(12)
    d:SetHeight(12)
    d:SetPoint("CENTER", win, "CENTER", (i - 1) * 20 - 10, 84)
    d:Hide()
    dots[i] = d
end

local slots = {}
for i = 1, MAX_ENTRIES do
    local b = CreateFrame("Button", nil, win)
    b:SetWidth(SLOT)
    b:SetHeight(SLOT)
    local shadow = b:CreateTexture(nil, "BACKGROUND")
    shadow:SetTexture(P.ART .. "Disc")
    shadow:SetVertexColor(0, 0, 0, 0.7)
    shadow:SetAllPoints(b)
    b.strips = P.NewRoundIcon(b, SLOT * 0.9)
    local ring = b:CreateTexture(nil, "OVERLAY")
    ring:SetTexture(P.ART .. "Ring")
    ring:SetVertexColor(0.85, 0.72, 0.5)
    ring:SetWidth(SLOT * 1.3)
    ring:SetHeight(SLOT * 1.3)
    ring:SetPoint("CENTER", b, "CENTER", 0, 0)
    local hi = b:CreateTexture(nil, "OVERLAY")
    hi:SetTexture(P.ART .. "RingSelect")
    hi:SetWidth(SLOT * 1.3)
    hi:SetHeight(SLOT * 1.3)
    hi:SetPoint("CENTER", b, "CENTER", 0, 0)
    hi:Hide()
    b.hi = hi
    local timer = b:CreateFontString(nil, "OVERLAY")
    timer:SetFont("Fonts\\FRIZQT__.TTF", 20, "OUTLINE")
    timer:SetPoint("CENTER", b, "CENTER", 0, 0)
    b.timer = timer
    local label = win:CreateFontString(nil, "OVERLAY")
    label:SetFont("Fonts\\FRIZQT__.TTF", 13, "OUTLINE")
    b.label = label
    local sub = win:CreateFontString(nil, "OVERLAY")
    sub:SetFont("Fonts\\FRIZQT__.TTF", 11, "OUTLINE")
    sub:SetTextColor(1, 0.82, 0)
    sub:SetPoint("TOP", label, "BOTTOM", 0, -1)
    b.sub = sub
    b.index = i
    b:SetScript("OnClick", function()
        Wheel.selected = this.index
        Wheel.Activate()
    end)
    b:SetScript("OnEnter", function()
        Wheel.selected = this.index
        Wheel.Refresh()
    end)
    slots[i] = b
end

-- ── State ──

Wheel.set = nil
Wheel.page = 1
Wheel.selected = nil
Wheel.entries = {}

local function entryText(value)
    if type(value) == "function" then
        return value()
    end
    return value
end

-- Lay the current entries round the ring, clockwise from the top.
function Wheel.Layout()
    local n = table.getn(Wheel.entries)
    for i = 1, MAX_ENTRIES do
        local b = slots[i]
        local e = Wheel.entries[i]
        if e then
            local a = math.rad((i - 1) * 360 / n)
            local x, y = math.sin(a), math.cos(a)
            b:ClearAllPoints()
            b:SetPoint("CENTER", win, "CENTER", x * RADIUS, y * RADIUS)
            P.SetIcon(b.strips, e.icon)
            b.label:ClearAllPoints()
            local lx, ly = x * LABEL_RADIUS, y * LABEL_RADIUS
            if x > 0.3 then
                b.label:SetPoint("LEFT", win, "CENTER", lx - 14, ly)
            elseif x < -0.3 then
                b.label:SetPoint("RIGHT", win, "CENTER", lx + 14, ly)
            else
                b.label:SetPoint("CENTER", win, "CENTER", lx, ly)
            end
            b.label:SetText(e.label)
            b:Show()
            b.label:Show()
            b.sub:Show()
        else
            b:Hide()
            b.label:Hide()
            b.sub:Hide()
        end
    end
end

-- What changes as the selection or the clock moves.
function Wheel.Refresh()
    if not win:IsVisible() then
        return
    end
    for i = 1, table.getn(Wheel.entries) do
        local b, e = slots[i], Wheel.entries[i]
        if i == Wheel.selected then
            b.hi:Show()
            b.label:SetTextColor(1, 0.82, 0)
        else
            b.hi:Hide()
            b.label:SetTextColor(1, 1, 1)
        end
        b.sub:SetText(entryText(e.sub) or "")
        local left = 0
        if e.cooldown then
            local start, duration, enable = e.cooldown()
            if enable == 1 and duration and duration > 1.5 then
                left = start + duration - GetTime()
            end
        end
        if left > 0 then
            b.timer:SetText(math.ceil(left))
            P.TintIcon(b.strips, 0.45, 0.45, 0.45)
        else
            b.timer:SetText("")
            P.TintIcon(b.strips, 1, 1, 1)
        end
    end
    local e = Wheel.selected and Wheel.entries[Wheel.selected]
    chosen:SetText(e and e.label or "")
end

function Wheel.Fill()
    if Wheel.set == "windows" then
        local page = WINDOW_PAGES[Wheel.page]
        Wheel.entries = page.entries
        pageTitle:SetText(page.title)
        hint:SetText("Stick: point   A: open   LB / RB: page   B: close")
        for i = 1, table.getn(dots) do
            if i <= table.getn(WINDOW_PAGES) then
                dots[i]:Show()
                if i == Wheel.page then
                    dots[i]:SetVertexColor(1, 0.78, 0.1)
                else
                    dots[i]:SetVertexColor(0.3, 0.3, 0.3)
                end
            else
                dots[i]:Hide()
            end
        end
    elseif Wheel.set == "bots" then
        Wheel.entries = botEntries()
        pageTitle:SetText("Bots")
        hint:SetText("To: " .. P.Bots.TargetLabel() .. "   Y: change   A: send   B: close")
        for i = 1, table.getn(dots) do
            dots[i]:Hide()
        end
    else
        Wheel.entries = consumableEntries()
        pageTitle:SetText("Consumables")
        if table.getn(Wheel.entries) == 0 then
            hint:SetText("Nothing to use in your bags   B: close")
        else
            hint:SetText("Stick: point   A: use   B: close")
        end
        for i = 1, table.getn(dots) do
            dots[i]:Hide()
        end
    end
    Wheel.Layout()
    Wheel.Refresh()
end

function Wheel.Open(set)
    if P.Menu and P.Menu.frame:IsVisible() then P.Menu.Close() end
    if P.Binds and P.Binds.frame:IsVisible() then P.Binds.frame:Hide() end
    Wheel.set = set
    Wheel.page = 1
    Wheel.selected = nil
    arrow:Hide()
    win:Show()
    Wheel.Fill()
end

function Wheel.Close()
    win:Hide()
end

function Wheel.Toggle(set)
    if win:IsVisible() and Wheel.set == set then
        Wheel.Close()
    else
        Wheel.Open(set)
    end
end

-- A on the selection: run it and close (the window it opens takes over).
function Wheel.Activate()
    local e = Wheel.selected and Wheel.entries[Wheel.selected]
    if not e then
        return
    end
    Wheel.Close()
    e.run()
end

-- The selection from a stick: the slot whose angle is nearest the stick's.
function Wheel.Aim(x, y)
    local n = table.getn(Wheel.entries)
    if n == 0 then
        return
    end
    local len = math.sqrt(x * x + y * y)
    if len < PICK then
        if len < RELEASE then
            arrow:Hide()
        end
        return
    end
    -- Clockwise from the top, in degrees.
    local degrees = math.deg(math.atan2(x, y))
    if degrees < 0 then
        degrees = degrees + 360
    end
    local step = 360 / n
    local index = math.floor((degrees + step / 2) / step)
    index = index - n * math.floor(index / n) + 1
    pointArrow(degrees)
    if index ~= Wheel.selected then
        Wheel.selected = index
        Wheel.Refresh()
    end
end

function Wheel.Step(dir)
    local n = table.getn(Wheel.entries)
    if n == 0 then
        return
    end
    local i = (Wheel.selected or 0) + dir
    if i < 1 then i = n end
    if i > n then i = 1 end
    Wheel.selected = i
    arrow:Hide()
    Wheel.Refresh()
end

function Wheel.Nav(btn)
    if btn == "A" then
        Wheel.Activate()
    elseif btn == "B" or btn == "START" or btn == "SELECT" then
        Wheel.Close()
    elseif btn == "Y" and Wheel.set == "bots" then
        P.Bots.CycleTarget(1)
        hint:SetText("To: " .. P.Bots.TargetLabel() .. "   Y: change   A: send   B: close")
    elseif btn == "DRIGHT" or btn == "DDOWN" then
        Wheel.Step(1)
    elseif btn == "DLEFT" or btn == "DUP" then
        Wheel.Step(-1)
    elseif (btn == "LB" or btn == "RB") and Wheel.set == "windows" then
        local n = table.getn(WINDOW_PAGES)
        if btn == "LB" then Wheel.page = Wheel.page - 1 else Wheel.page = Wheel.page + 1 end
        if Wheel.page < 1 then Wheel.page = n end
        if Wheel.page > n then Wheel.page = 1 end
        Wheel.selected = nil
        arrow:Hide()
        Wheel.Fill()
    end
end

win:SetScript("OnShow", function() P.mode = "wheel" end)
win:SetScript("OnHide", function()
    if not ((P.Menu and P.Menu.frame:IsVisible()) or (P.Binds and P.Binds.frame:IsVisible())) then
        P.mode = "world"
    end
end)

P.Listen(function(event, a1, a2, a3, a4)
    if not win:IsVisible() then
        return
    end
    if event == "BENILLAPAD_NAV" then
        if a2 then
            Wheel.Nav(a1)
        end
        return true
    elseif event == "BENILLAPAD_STICK" then
        -- The left stick, then the right; the one pushed further aims.
        local lx, ly, rx, ry = a1 or 0, a2 or 0, a3 or 0, a4 or 0
        if rx * rx + ry * ry >= lx * lx + ly * ly then
            Wheel.Aim(rx, ry)
        else
            Wheel.Aim(lx, ly)
        end
    end
end)

local watcher = CreateFrame("Frame", nil, win)
watcher:RegisterEvent("BAG_UPDATE")
watcher:RegisterEvent("BAG_UPDATE_COOLDOWN")
watcher:SetScript("OnEvent", function()
    if Wheel.set == "consumables" then
        local keep = Wheel.selected
        Wheel.Fill()
        Wheel.selected = keep
        Wheel.Refresh()
    end
end)

local elapsedSince = 0
win:SetScript("OnUpdate", function()
    elapsedSince = elapsedSince + arg1
    if elapsedSince > 0.2 then
        elapsedSince = 0
        Wheel.Refresh()
    end
end)

-- ── The quest item ──

local Quest = {}
P.Quest = Quest

-- The first usable quest item in the bags, read off the tooltips: a "Quest Item" line and a
-- "Use:" line (1.12 has no GetItemSpell). Rescanned when the bags change.
local found, dirty = nil, true
local tip = BenillaPadScanTip

function Quest.Find()
    if not dirty then
        return found
    end
    dirty = false
    found = nil
    for bag = 0, 4 do
        for slot = 1, GetContainerNumSlots(bag) do
            if GetContainerItemLink(bag, slot) then
                tip:SetOwner(UIParent, "ANCHOR_NONE")
                tip:SetBagItem(bag, slot)
                local quest, use = false, false
                for line = 1, tip:NumLines() do
                    local text = getglobal("BenillaPadScanTipTextLeft" .. line):GetText() or ""
                    if text == ITEM_BIND_QUEST then
                        quest = true
                    elseif string.find(text, ITEM_SPELL_TRIGGER_ONUSE, 1, true) == 1 then
                        use = true
                    end
                end
                tip:Hide()
                if quest and use then
                    local texture = GetContainerItemInfo(bag, slot)
                    found = { bag = bag, slot = slot, icon = texture }
                    return found
                end
            end
        end
    end
    return nil
end

function Quest.Use()
    local item = Quest.Find()
    if not item then
        UIErrorsFrame:AddMessage("No usable quest item in your bags", 1, 0.1, 0.1, 1, 5)
        return
    end
    UseContainerItem(item.bag, item.slot)
end

local questWatch = CreateFrame("Frame")
questWatch:RegisterEvent("BAG_UPDATE")
questWatch:SetScript("OnEvent", function()
    dirty = true
    if P.Bar then P.Bar.Refresh() end
end)

P.Actions.questitem.iconFn = function()
    local item = Quest.Find()
    return item and item.icon
end
