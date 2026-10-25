-- Game windows with the pad. While a stock window that wants the pointer is open (bags, vendor,
-- loot, quest, gossip, trainer, mail, popups, ...), BenillaPad_Mode() answers "cursor": the Rust
-- side moves the mouse cursor with the right stick, clicks with A (left) and X (right), closes
-- with B (the Escape ladder), scrolls with LB / RB, and snaps to the nearest button with the
-- D-pad through BenillaPad_Snap. The left stick still walks.

local P = BenillaPad
local Cursor = {}
P.Cursor = Cursor

-- The stock windows that take the pointer, by global name.
local WINDOWS = {
    "MerchantFrame", "LootFrame", "QuestFrame", "GossipFrame", "ClassTrainerFrame", "TaxiFrame",
    "BankFrame", "MailFrame", "OpenMailFrame", "AuctionFrame", "TradeFrame", "CharacterFrame",
    "SpellBookFrame", "TalentFrame", "QuestLogFrame", "WorldMapFrame", "FriendsFrame",
    "GameMenuFrame", "OptionsFrame", "SoundOptionsFrame", "UIOptionsFrame", "KeyBindingFrame",
    "MacroFrame", "PetStableFrame", "TradeSkillFrame", "CraftFrame", "InspectFrame",
    "ItemTextFrame", "GuildRegistrarFrame", "PetitionFrame", "TabardFrame", "HelpFrame",
    "DressUpFrame", "BattlefieldFrame", "PetPaperDollFrame", "AddonList",
    "StaticPopup1", "StaticPopup2", "StaticPopup3", "StaticPopup4",
    "GroupLootFrame1", "GroupLootFrame2", "GroupLootFrame3", "GroupLootFrame4",
}
for i = 1, 12 do
    table.insert(WINDOWS, "ContainerFrame" .. i)
end

-- The open windows, as a set of frames.
function Cursor.Open()
    local open, any = {}, false
    for i = 1, table.getn(WINDOWS) do
        local f = getglobal(WINDOWS[i])
        if f and f:IsVisible() then
            open[f] = true
            any = true
        end
    end
    return open, any
end

function Cursor.Wanted()
    if not P.Setting("padCursor") then
        return false
    end
    local _, any = Cursor.Open()
    return any
end

-- Add the visible buttons under `f` to `list`. Only shown frames are entered, so a window's
-- hidden tabs and pages cost nothing.
local function collect(f, list, depth)
    local children = { f:GetChildren() }
    for i = 1, table.getn(children) do
        local c = children[i]
        if c:IsVisible() then
            local kind = c:GetObjectType()
            if kind == "Button" or kind == "CheckButton" then
                local left, bottom = c:GetLeft(), c:GetBottom()
                local w, h = c:GetWidth(), c:GetHeight()
                if left and bottom and w > 4 and h > 4 then
                    local s = c:GetEffectiveScale()
                    table.insert(list, { x = (left + w / 2) * s, y = (bottom + h / 2) * s, frame = c })
                end
            end
            if depth < 8 then
                collect(c, list, depth + 1)
            end
        end
    end
end

-- The clickable buttons of the open windows, as { x, y } centres in absolute units (UI units
-- times the effective scale; 768 is the screen's height). It walks each open window's own
-- children: a walk over every frame in the game (EnumerateFrames) costs tens of milliseconds a
-- press once addons have made thousands of them.
function Cursor.Targets()
    local open = Cursor.Open()
    local list = {}
    for f in pairs(open) do
        collect(f, list, 1)
    end
    return list
end

-- The button to jump to from (x, y) toward `dir` (DUP / DDOWN / DLEFT / DRIGHT), or, for "enter",
-- an open popup's first button. Answers absolute units, or nothing.
function BenillaPad_Snap(dir, x, y)
    if dir == "enter" then
        for i = 1, 4 do
            local popup = getglobal("StaticPopup" .. i)
            local button = getglobal("StaticPopup" .. i .. "Button1")
            if popup and popup:IsVisible() and button and button:IsVisible() then
                local s = button:GetEffectiveScale()
                return (button:GetLeft() + button:GetWidth() / 2) * s,
                    (button:GetBottom() + button:GetHeight() / 2) * s
            end
        end
        return nil
    end
    local best, bestScore
    local targets = Cursor.Targets()
    for i = 1, table.getn(targets) do
        local t = targets[i]
        local dx, dy = t.x - x, t.y - y
        local along, across
        if dir == "DUP" then along, across = dy, dx
        elseif dir == "DDOWN" then along, across = -dy, dx
        elseif dir == "DLEFT" then along, across = -dx, dy
        else along, across = dx, dy end
        if along > 4 then
            local score = along + 2 * math.abs(across)
            if not bestScore or score < bestScore then
                best, bestScore = t, score
            end
        end
    end
    if best then
        return best.x, best.y
    end
    return nil
end
