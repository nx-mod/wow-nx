-- The experience bar, along the bottom of the screen while the gamepad bar shows (the stock one
-- goes with the stock bars): experience in purple, or blue while rested, the rested bonus as a
-- lighter stretch ahead of it, and the numbers as text. Hidden at the level cap.

local P = BenillaPad
local Xp = {}
P.Xp = Xp

local WIDTH, HEIGHT = 620, 9
local TEXTURE = "Interface\\TargetingFrame\\UI-StatusBar"

local frame = CreateFrame("Frame", "BenillaPadXpBar", UIParent)
frame:SetWidth(WIDTH + 4)
frame:SetHeight(HEIGHT + 4)
frame:SetPoint("BOTTOM", UIParent, "BOTTOM", 0, 6)
frame:SetFrameStrata("LOW")
frame:EnableMouse(true)
frame:Hide()
Xp.frame = frame

local back = frame:CreateTexture(nil, "BACKGROUND")
back:SetAllPoints(frame)
back:SetTexture(0, 0, 0, 0.6)

-- The rested bonus, under the experience bar: it shows past the experience's end.
local rested = CreateFrame("StatusBar", nil, frame)
rested:SetWidth(WIDTH)
rested:SetHeight(HEIGHT)
rested:SetPoint("CENTER", frame, "CENTER", 0, 0)
rested:SetStatusBarTexture(TEXTURE)
rested:SetStatusBarColor(0.25, 0.45, 0.85, 0.55)

local bar = CreateFrame("StatusBar", nil, frame)
bar:SetWidth(WIDTH)
bar:SetHeight(HEIGHT)
bar:SetPoint("CENTER", frame, "CENTER", 0, 0)
bar:SetStatusBarTexture(TEXTURE)
bar:SetFrameLevel(rested:GetFrameLevel() + 1)

-- Twenty segments, as the stock bar is divided.
local over = CreateFrame("Frame", nil, frame)
over:SetAllPoints(bar)
over:SetFrameLevel(bar:GetFrameLevel() + 1)
for i = 1, 19 do
    local tick = over:CreateTexture(nil, "OVERLAY")
    tick:SetTexture(0, 0, 0, 0.55)
    tick:SetWidth(1)
    tick:SetHeight(HEIGHT)
    tick:SetPoint("LEFT", over, "LEFT", i * WIDTH / 20, 0)
end

local text = over:CreateFontString(nil, "OVERLAY")
text:SetFont("Fonts\\FRIZQT__.TTF", 10, "OUTLINE")
text:SetPoint("BOTTOM", frame, "TOP", 0, 1)
text:SetTextColor(1, 1, 1)

function Xp.Update()
    local xp, max = UnitXP("player"), UnitXPMax("player")
    if not max or max <= 0 then
        return
    end
    local bonus = GetXPExhaustion() or 0
    bar:SetMinMaxValues(0, max)
    bar:SetValue(xp)
    rested:SetMinMaxValues(0, max)
    rested:SetValue(math.min(max, xp + bonus))
    if bonus > 0 then
        bar:SetStatusBarColor(0.2, 0.5, 1.0)
    else
        bar:SetStatusBarColor(0.6, 0.2, 0.85)
    end
    local line = "Level " .. UnitLevel("player") .. "   " .. xp .. " / " .. max
        .. "  (" .. math.floor(xp * 100 / max) .. "%)"
    if bonus > 0 then
        line = line .. "   |cff66aaffRested +" .. bonus .. "|r"
    end
    text:SetText(line)
end

-- Shown with the gamepad bar, below the level cap, while the setting is on.
function Xp.Refresh()
    local cap = MAX_PLAYER_LEVEL or 60
    local show = P.Setting("xpBar") and P.Bar and P.Bar.frame:IsShown()
        and (UnitLevel("player") or 0) < cap
    if show then
        frame:Show()
        if P.Setting("xpText") then text:Show() else text:Hide() end
        Xp.Update()
    else
        frame:Hide()
    end
end

-- With the numbers off, they show while the pointer is over the bar.
frame:SetScript("OnEnter", function() text:Show() end)
frame:SetScript("OnLeave", function()
    if not P.Setting("xpText") then text:Hide() end
end)

local watch = CreateFrame("Frame")
watch:RegisterEvent("PLAYER_ENTERING_WORLD")
watch:RegisterEvent("PLAYER_XP_UPDATE")
watch:RegisterEvent("PLAYER_LEVEL_UP")
watch:RegisterEvent("UPDATE_EXHAUSTION")
watch:RegisterEvent("PLAYER_UPDATE_RESTING")
watch:SetScript("OnEvent", function() Xp.Refresh() end)
