-- Playerbots (cmangos) control: the roster of the account's characters, the command catalog by
-- category, and where a command goes. `.bot` commands are GM commands, sent as a "SAY" line the
-- server takes before anyone hears it; bot commands go to party chat (every bot) or a whisper
-- (one bot).
--   `.bot list` answers "Bot roster: +Name Class, -Name Class, ..." (+ online), every character
--   on the account but the one playing (PlayerbotMgr.cpp ListBots).
--   `.bot add A,B` logs bots in and adds them to the group; `.bot remove A` logs one out.

local P = BenillaPad
local Bots = {}
P.Bots = Bots

-- ── The catalog ──
-- A command: { id, label, cmd } sends cmd; `ask` opens the keyboard with cmd and a space typed.

local CATEGORIES = {
    { title = "Movement", commands = {
        { id = "follow", label = "Follow", cmd = "follow" },
        { id = "stay", label = "Stay", cmd = "stay" },
        { id = "guard", label = "Guard", cmd = "guard" },
        { id = "flee", label = "Flee", cmd = "flee" },
        { id = "summon", label = "Summon", cmd = "summon" },
        { id = "wander", label = "Wander", cmd = "wander" },
        { id = "free", label = "Free", cmd = "free" },
        { id = "where", label = "Where are you", cmd = "where" },
        { id = "home", label = "Go home", cmd = "home" },
        { id = "taxi", label = "Take my taxi", cmd = "taxi" },
    } },
    { title = "Combat", commands = {
        { id = "attack", label = "Attack", cmd = "attack" },
        { id = "tankattack", label = "Tank attack", cmd = "tank attack" },
        { id = "pull", label = "Pull", cmd = "pull" },
        { id = "maxdps", label = "Max DPS", cmd = "max dps" },
        { id = "savemana", label = "Save mana", cmd = "save mana" },
        { id = "grind", label = "Grind", cmd = "grind" },
        { id = "attackers", label = "Attackers", cmd = "attackers" },
        { id = "cast", label = "Cast...", cmd = "cast", ask = true },
        { id = "rti", label = "Attack marked", cmd = "attack rti" },
    } },
    { title = "Quests", commands = {
        { id = "acceptall", label = "Accept all", cmd = "accept *" },
        { id = "quests", label = "Quests", cmd = "quests" },
        { id = "talk", label = "Talk", cmd = "talk" },
        { id = "reward", label = "Quest reward", cmd = "quest reward" },
        { id = "share", label = "Share", cmd = "share" },
        { id = "autoaccept", label = "Auto-accept on", cmd = "nc +accept all quests" },
        { id = "drop", label = "Drop quest...", cmd = "drop", ask = true },
    } },
    { title = "Loot & items", commands = {
        { id = "loot", label = "Loot", cmd = "loot" },
        { id = "addallloot", label = "Loot everything", cmd = "add all loot" },
        { id = "inventory", label = "Inventory", cmd = "items" },
        { id = "repair", label = "Repair", cmd = "repair" },
        { id = "sell", label = "Sell...", cmd = "s", ask = true },
        { id = "trainer", label = "Trainer", cmd = "trainer" },
        { id = "equip", label = "Equip...", cmd = "e", ask = true },
        { id = "use", label = "Use...", cmd = "u", ask = true },
        { id = "stats", label = "Stats", cmd = "stats" },
    } },
    { title = "Death", commands = {
        { id = "release", label = "Release", cmd = "release" },
        { id = "revive", label = "Revive", cmd = "revive" },
        { id = "selfres", label = "Self res", cmd = "self res" },
    } },
    { title = "Group", commands = {
        { id = "ready", label = "Ready check", cmd = "ready" },
        { id = "leave", label = "Leave group", cmd = "leave" },
        { id = "whoami", label = "Who are you", cmd = "who" },
        { id = "help", label = "Help", cmd = "help" },
    } },
    { title = "Strategies", commands = {
        { id = "resetstrats", label = "Reset strategies", cmd = "reset strats" },
        { id = "resetai", label = "Reset AI", cmd = "reset ai" },
        { id = "co", label = "Combat strategy...", cmd = "co", ask = true },
        { id = "nc", label = "Out-of-combat strategy...", cmd = "nc", ask = true },
    } },
}
Bots.CATEGORIES = CATEGORIES

local BY_ID = {}
for i = 1, table.getn(CATEGORIES) do
    local list = CATEGORIES[i].commands
    for j = 1, table.getn(list) do
        BY_ID[list[j].id] = list[j]
    end
end

-- The bot wheel's default eight.
local DEFAULT_FAVORITES = { "follow", "stay", "attack", "flee", "pull", "summon", "acceptall", "loot" }

-- The saved state: the roster (account-wide), favourites and custom commands.
local function db()
    BenillaPadDB = BenillaPadDB or {}
    local d = BenillaPadDB
    d.roster = d.roster or {}
    if not d.botFavorites then
        d.botFavorites = {}
        for i = 1, table.getn(DEFAULT_FAVORITES) do
            d.botFavorites[i] = DEFAULT_FAVORITES[i]
        end
    end
    d.botCustom = d.botCustom or {}
    return d
end

function Bots.Command(id)
    if BY_ID[id] then
        return BY_ID[id]
    end
    local custom = db().botCustom
    for i = 1, table.getn(custom) do
        if custom[i].id == id then
            return custom[i]
        end
    end
    return nil
end

function Bots.Custom()
    return db().botCustom
end

function Bots.AddCustom(text)
    local custom = db().botCustom
    table.insert(custom, { id = "custom" .. (table.getn(custom) + 1) .. "-" .. text, label = text, cmd = text })
end

function Bots.RemoveCustom(id)
    local custom = db().botCustom
    for i = table.getn(custom), 1, -1 do
        if custom[i].id == id then
            table.remove(custom, i)
        end
    end
end

function Bots.Favorites()
    return db().botFavorites
end

function Bots.IsFavorite(id)
    local f = db().botFavorites
    for i = 1, table.getn(f) do
        if f[i] == id then
            return i
        end
    end
    return nil
end

-- Add or drop a command on the bot wheel (12 at most).
function Bots.ToggleFavorite(id)
    local f = db().botFavorites
    local at = Bots.IsFavorite(id)
    if at then
        table.remove(f, at)
        return false
    end
    if table.getn(f) >= 12 then
        P.Print("the bot wheel is full (12); remove one first")
        return false
    end
    table.insert(f, id)
    return true
end

-- ── The roster ──

function Bots.Roster()
    return db().roster
end

function Bots.Find(name)
    local roster = db().roster
    for i = 1, table.getn(roster) do
        if roster[i].name == name then
            return roster[i]
        end
    end
    return nil
end

-- Read a "Bot roster: +Name Class, -Name Class" line into the saved roster.
function Bots.ParseRoster(msg)
    local _, _, body = string.find(msg, "^Bot roster: (.*)$")
    if not body then
        return false
    end
    local roster = {}
    for entry in string.gfind(body, "[^,]+") do
        local _, _, mark, name, class = string.find(entry, "^%s*([%+%-])(%S+)%s*(%S*)")
        if name then
            table.insert(roster, { name = name, class = class, online = (mark == "+") })
        end
    end
    db().roster = roster
    if P.Chat and P.Chat.Refresh then
        P.Chat.Refresh()
    end
    return true
end

-- A server line about one bot changes its mark until the next refresh.
local function mark(name, online)
    local bot = Bots.Find(name)
    if bot then
        bot.online = online
    end
end

-- ── Sending ──

-- Who a bot command goes to: "auto" (the targeted bot, else the party), "all", or a name.
Bots.target = "auto"

local function grouped()
    return GetNumRaidMembers() > 0 or GetNumPartyMembers() > 0
end

-- The bot names in the current target choice, or nil for the group channel.
function Bots.Recipients()
    local t = Bots.target
    if t == "auto" then
        if UnitExists("target") and UnitIsPlayer("target") then
            local name = UnitName("target")
            if Bots.Find(name) then
                return { name }
            end
        end
        t = "all"
    end
    if t ~= "all" then
        return { t }
    end
    if grouped() then
        return nil
    end
    -- Not grouped: whisper every online bot.
    local list = {}
    local roster = db().roster
    for i = 1, table.getn(roster) do
        if roster[i].online then
            table.insert(list, roster[i].name)
        end
    end
    return list
end

function Bots.TargetLabel()
    if Bots.target == "auto" then
        if UnitExists("target") and UnitIsPlayer("target") and Bots.Find(UnitName("target")) then
            return "Auto: " .. UnitName("target")
        end
        return "Auto: everyone"
    elseif Bots.target == "all" then
        return "Everyone"
    end
    return Bots.target
end

-- Step the target choice: auto, everyone, then each online bot.
function Bots.CycleTarget(dir)
    local choices = { "auto", "all" }
    local roster = db().roster
    for i = 1, table.getn(roster) do
        if roster[i].online then
            table.insert(choices, roster[i].name)
        end
    end
    local at = 1
    for i = 1, table.getn(choices) do
        if choices[i] == Bots.target then at = i end
    end
    at = at + (dir or 1)
    if at < 1 then at = table.getn(choices) end
    if at > table.getn(choices) then at = 1 end
    Bots.target = choices[at]
end

-- Send a bot chat command to the current choice.
function Bots.Say(text)
    if not text or text == "" then
        return
    end
    local to = Bots.Recipients()
    if not to then
        local channel = "PARTY"
        if GetNumRaidMembers() > 0 then channel = "RAID" end
        SendChatMessage(text, channel)
        return
    end
    if table.getn(to) == 0 then
        UIErrorsFrame:AddMessage("No bot to command: add one first", 1, 0.1, 0.1, 1, 5)
        return
    end
    for i = 1, table.getn(to) do
        SendChatMessage(text, "WHISPER", nil, to[i])
    end
end

-- Run a catalog command; one that takes a word opens the keyboard first.
function Bots.Run(command)
    if command.ask then
        P.Keyboard.Open(command.label .. "  (to " .. Bots.TargetLabel() .. ")", command.cmd .. " ",
            function(text) Bots.Say(text) end)
    else
        Bots.Say(command.cmd)
    end
end

-- A GM command, which the server reads off a say line.
function Bots.Dot(text)
    SendChatMessage(text, "SAY")
end

function Bots.Refresh()
    Bots.Dot(".bot list")
end

function Bots.Add(name)
    Bots.Dot(".bot add " .. name)
end

function Bots.Remove(name)
    Bots.Dot(".bot remove " .. name)
end

-- Every offline character on the roster, in one `.bot add`.
function Bots.AddAll()
    local names = {}
    local roster = db().roster
    for i = 1, table.getn(roster) do
        if not roster[i].online then
            table.insert(names, roster[i].name)
        end
    end
    if table.getn(names) == 0 then
        P.Print("no offline character on the roster; Refresh first")
        return
    end
    Bots.Add(table.concat(names, ","))
end

function Bots.RemoveAll()
    local roster = db().roster
    for i = 1, table.getn(roster) do
        if roster[i].online then
            Bots.Remove(roster[i].name)
        end
    end
end

function Bots.Invite(name)
    InviteByName(name)
end

-- The server's answers: the roster line, and a refresh after logins and logouts.
local listen = CreateFrame("Frame")
listen:RegisterEvent("CHAT_MSG_SYSTEM")
listen:RegisterEvent("PARTY_MEMBERS_CHANGED")
listen:SetScript("OnEvent", function()
    if event == "CHAT_MSG_SYSTEM" then
        if Bots.ParseRoster(arg1 or "") then
            return
        end
    end
    if event == "PARTY_MEMBERS_CHANGED" then
        for i = 1, GetNumPartyMembers() do
            local name = UnitName("party" .. i)
            if name then mark(name, true) end
        end
        if P.Chat and P.Chat.Refresh then
            P.Chat.Refresh()
        end
    end
end)
