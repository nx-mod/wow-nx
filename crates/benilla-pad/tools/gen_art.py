#!/usr/bin/env python3
"""BenillaPad's art, drawn from scratch: no external images, no imaging libraries.

    python3 crates/benilla-pad/tools/gen_art.py [--preview <png>]

Writes 32-bit TGAs (bottom-left origin, the form 1.12 loads) into addon/BenillaPad/Art:

- Ring.tga        the round slot's metal ring, neutral grey (the Lua tints it bronze); drawn at
                  1.3x the button, so the band (radius 0.66-0.84 of the texture) covers the
                  round icon's edge at 0.69
- RingSelect.tga  the gold press / selection ring, same geometry
- Disc.tga        a soft white disc (badges, shadows, the wheel's backdrop)
- one 64x64 icon per game action, window-wheel entry and bot command (`ICONS`): a dark slate
                  tile with a bold glyph, so none reads as a spell

Shapes are signed distance functions sampled 4x4 per pixel.
"""

import math
import os
import struct
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
ART = os.path.join(HERE, "..", "addon", "BenillaPad", "Art")
SS = 4  # supersamples per axis


def clamp(x, a=0.0, b=1.0):
    return a if x < a else b if x > b else x


def smooth(edge0, edge1, x):
    t = clamp((x - edge0) / (edge1 - edge0))
    return t * t * (3 - 2 * t)


def over(dst, src):
    """Source-over of two straight-alpha RGBA tuples (0..1)."""
    sa = src[3]
    da = dst[3]
    a = sa + da * (1 - sa)
    if a <= 0:
        return (0.0, 0.0, 0.0, 0.0)
    return tuple((src[i] * sa + dst[i] * da * (1 - sa)) / a for i in range(3)) + (a,)


def render(size, shade):
    """An image of size x size: `shade(x, y)` gives RGBA at a point of [-1, 1]^2 (y up),
    averaged over SS x SS samples. Rows top to bottom."""
    rows = []
    for py in range(size):
        row = []
        for px in range(size):
            acc = [0.0, 0.0, 0.0, 0.0]
            for sy in range(SS):
                for sx in range(SS):
                    x = ((px + (sx + 0.5) / SS) / size) * 2 - 1
                    y = 1 - ((py + (sy + 0.5) / SS) / size) * 2
                    r, g, b, a = shade(x, y)
                    acc[0] += r * a
                    acc[1] += g * a
                    acc[2] += b * a
                    acc[3] += a
            n = SS * SS
            a = acc[3] / n
            if a > 0:
                row.append((acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3], a))
            else:
                row.append((0.0, 0.0, 0.0, 0.0))
        rows.append(row)
    return rows


def write_tga(path, rows):
    h = len(rows)
    w = len(rows[0])
    header = struct.pack("<BBBHHBHHHHBB", 0, 0, 2, 0, 0, 0, 0, 0, w, h, 32, 8)
    body = bytearray()
    for row in reversed(rows):  # bottom-left origin
        for r, g, b, a in row:
            body += bytes((round(clamp(b) * 255), round(clamp(g) * 255), round(clamp(r) * 255),
                           round(clamp(a) * 255)))
    with open(path, "wb") as f:
        f.write(header + body)


def write_png(path, rows):
    h = len(rows)
    w = len(rows[0])
    raw = bytearray()
    for row in rows:
        raw.append(0)
        for r, g, b, a in row:
            raw += bytes((round(clamp(r) * 255), round(clamp(g) * 255), round(clamp(b) * 255),
                          round(clamp(a) * 255)))

    def chunk(kind, data):
        c = struct.pack(">I", len(data)) + kind + data
        return c + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(png)


# ── Signed distances (negative inside) ──

def sd_circle(x, y, cx, cy, r):
    return math.hypot(x - cx, y - cy) - r


def sd_box(x, y, cx, cy, hw, hh, rad=0.0):
    qx = abs(x - cx) - hw + rad
    qy = abs(y - cy) - hh + rad
    return math.hypot(max(qx, 0), max(qy, 0)) + min(max(qx, qy), 0) - rad


def sd_segment(x, y, ax, ay, bx, by, r):
    px, py = x - ax, y - ay
    dx, dy = bx - ax, by - ay
    t = clamp((px * dx + py * dy) / (dx * dx + dy * dy))
    return math.hypot(px - dx * t, py - dy * t) - r


def fill(d, aa=0.012):
    """Coverage of a signed distance, with a small soft edge."""
    return 1 - smooth(-aa, aa, d)


# ── The rings ──

IN, OUT = 0.66, 0.84


def ring(x, y):
    r = math.hypot(x, y)
    out = (0.0, 0.0, 0.0, 0.0)
    # A soft drop shadow outside the band.
    shadow = (1 - smooth(OUT, 0.98, r)) * smooth(IN, OUT, r) * 0.55
    out = over(out, (0.0, 0.0, 0.0, shadow))
    band = fill(IN - r) * fill(r - OUT)
    if band > 0:
        # Light from the top left: the bevel's profile across the band, lit by the angle.
        t = (r - IN) / (OUT - IN)
        profile = 0.55 + 0.45 * math.sin(t * math.pi)
        angle = math.atan2(y, x)
        light = 0.75 + 0.25 * math.cos(angle - math.radians(135))
        v = clamp(profile * light)
        # A bright inner lip and a dark outer edge.
        lip = 1 - smooth(0.0, 0.12, t)
        edge = smooth(0.85, 1.0, t)
        v = clamp(v + 0.25 * lip - 0.45 * edge)
        out = over(out, (v, v, v, band))
    return out


def ring_select(x, y):
    r = math.hypot(x, y)
    # A gold band with a glow that fades out to the edge.
    glow = (1 - smooth(OUT, 1.0, r)) * smooth(IN - 0.08, IN, r) * 0.6
    out = (1.0, 0.78, 0.18, glow)
    band = fill(IN - 0.02 - r) * fill(r - (OUT - 0.02))
    if band > 0:
        t = (r - (IN - 0.02)) / (OUT - IN)
        v = 0.7 + 0.3 * math.sin(clamp(t) * math.pi)
        out = over(out, (1.0, 0.82 * v + 0.1, 0.25 * v, band))
    return out


def disc(x, y):
    return (1.0, 1.0, 1.0, fill(math.hypot(x, y) - 0.94, 0.04))


# ── The icons ──

def icon_base(x, y):
    """A dark slate tile with a light top and a thin border, behind every icon."""
    d = sd_box(x, y, 0, 0, 0.98, 0.98, 0.18)
    a = fill(d, 0.02)
    top = 0.5 + 0.5 * y
    col = (0.06 + 0.07 * top, 0.09 + 0.09 * top, 0.15 + 0.12 * top, a)
    border = fill(abs(d + 0.05) - 0.025, 0.02) * a
    return over(col, (0.55, 0.62, 0.75, border * 0.8))


WHITE = (0.96, 0.96, 0.92)
GOLD = (1.0, 0.82, 0.25)
RED = (0.95, 0.27, 0.22)
GREEN = (0.35, 0.85, 0.35)
BLUE = (0.35, 0.65, 1.0)


def icon(glyph):
    """A tile with a glyph: `glyph(x, y)` gives layers [(distance, colour), ...], painted in
    order over a dark outline of their union."""
    def shade(x, y):
        out = icon_base(x, y)
        layers = glyph(x, y)
        d = min(layer[0] for layer in layers)
        out = over(out, (0.0, 0.0, 0.0, fill(d - 0.06) * 0.7))
        for dist, colour in layers:
            out = over(out, colour + (fill(dist),))
        return out
    return shade


def seg(x, y, pts, r):
    """A polyline of round-capped strokes."""
    return min(sd_segment(x, y, pts[i][0], pts[i][1], pts[i + 1][0], pts[i + 1][1], r)
               for i in range(len(pts) - 1))


def ring_d(x, y, cx, cy, r, w):
    return abs(sd_circle(x, y, cx, cy, r)) - w


def bust(x, y, cx=0.0, cy=0.0, k=1.0):
    """A head and shoulders."""
    head = sd_circle(x, y, cx, cy + 0.32 * k, 0.24 * k)
    body = max(sd_circle(x, y, cx, cy - 0.5 * k, 0.5 * k), -(y - (cy - 0.52 * k)))
    return min(head, body)


def arrow(x, y, ax, ay, bx, by, r=0.09, head=0.26):
    """A stroke from a to b with a chevron at b."""
    dx, dy = bx - ax, by - ay
    n = math.hypot(dx, dy)
    ux, uy = dx / n, dy / n
    px, py = -uy, ux
    l = (bx - ux * head + px * head, by - uy * head + py * head)
    rr = (bx - ux * head - px * head, by - uy * head - py * head)
    return min(sd_segment(x, y, ax, ay, bx, by, r), seg(x, y, [l, (bx, by), rr], r))


def crosshair(x, y):
    d = ring_d(x, y, 0, 0, 0.5, 0.075)
    for a in range(4):
        c, sn = math.cos(a * math.pi / 2), math.sin(a * math.pi / 2)
        d = min(d, sd_segment(x, y, 0.36 * c, 0.36 * sn, 0.74 * c, 0.74 * sn, 0.075))
    return min(d, sd_circle(x, y, 0, 0, 0.1))


def bag_d(x, y):
    body = sd_box(x, y, 0, -0.14, 0.5, 0.42, 0.2)
    neck = sd_box(x, y, 0, 0.36, 0.3, 0.1, 0.06)
    tie = sd_segment(x, y, -0.36, 0.5, 0.36, 0.5, 0.07)
    return min(body, neck, tie)


def shield_d(x, y):
    # A flat-topped body over a rounded point.
    return max(min(sd_box(x, y, 0, 0.22, 0.48, 0.34, 0.08), sd_circle(x, y, 0, -0.06, 0.5)),
               y - 0.6)


def sword_d(x, y):
    blade = sd_segment(x, y, -0.2, -0.2, 0.55, 0.55, 0.085)
    guard = sd_segment(x, y, -0.42, 0.02, 0.02, -0.42, 0.075)
    grip = sd_segment(x, y, -0.3, -0.3, -0.52, -0.52, 0.09)
    return min(blade, guard, grip)


def g_autorun(x, y):
    return [(min(seg(x, y, [(-0.55, 0.45), (-0.1, 0.0), (-0.55, -0.45)], 0.11),
                 seg(x, y, [(0.0, 0.45), (0.45, 0.0), (0.0, -0.45)], 0.11)), WHITE)]


def g_wheel(x, y):
    d = sd_circle(x, y, 0, 0, 0.14)
    for i in range(1, 8):
        a = math.radians(90 - i * 45)
        d = min(d, sd_circle(x, y, 0.58 * math.cos(a), 0.58 * math.sin(a), 0.13))
    return [(d, WHITE), (sd_circle(x, y, 0, 0.58, 0.13), GOLD)]


def g_menu(x, y):
    body = min(sd_box(x, y, 0, 0.08, 0.62, 0.3, 0.28),
               sd_circle(x, y, -0.48, -0.2, 0.3), sd_circle(x, y, 0.48, -0.2, 0.3))
    dpad = min(sd_box(x, y, -0.42, 0.05, 0.16, 0.05), sd_box(x, y, -0.42, 0.05, 0.05, 0.16))
    face = min(sd_circle(x, y, 0.42, 0.18, 0.06), sd_circle(x, y, 0.42, -0.08, 0.06),
               sd_circle(x, y, 0.29, 0.05, 0.06), sd_circle(x, y, 0.55, 0.05, 0.06))
    return [(max(body, -min(dpad, face)), WHITE)]


def g_jump(x, y):
    return [(arrow(x, y, 0, -0.3, 0, 0.6, 0.1, 0.3), WHITE),
            (sd_segment(x, y, -0.5, -0.58, 0.5, -0.58, 0.08), GOLD)]


def g_interact(x, y):
    # An open hand: a palm, four fingers and a thumb.
    d = sd_box(x, y, 0.02, -0.3, 0.3, 0.26, 0.14)
    for i, top in enumerate((0.42, 0.58, 0.56, 0.4)):
        fx = -0.21 + i * 0.155
        d = min(d, sd_segment(x, y, fx, -0.15, fx, top, 0.07))
    d = min(d, sd_segment(x, y, -0.3, -0.36, -0.55, -0.05, 0.08))
    return [(d, WHITE)]


def g_back(x, y):
    return [(min(sd_segment(x, y, -0.42, -0.42, 0.42, 0.42, 0.13),
                 sd_segment(x, y, -0.42, 0.42, 0.42, -0.42, 0.13)), RED)]


def g_inspect(x, y):
    return [(min(ring_d(x, y, -0.12, 0.14, 0.38, 0.08),
                 sd_segment(x, y, 0.18, -0.16, 0.56, -0.54, 0.11)), WHITE)]


def g_target(colour):
    return lambda x, y: [(crosshair(x, y), colour)]


def g_self(x, y):
    return [(bust(x, y, 0, -0.02, 1.0), WHITE), (ring_d(x, y, 0, 0, 0.84, 0.045), GOLD)]


def g_attack(x, y):
    return [(sword_d(x, y), WHITE)]


def g_sit(x, y):
    # A chair in profile.
    return [(min(sd_segment(x, y, -0.34, 0.6, -0.34, -0.6, 0.085),
                 sd_segment(x, y, -0.34, -0.05, 0.4, -0.05, 0.085),
                 sd_segment(x, y, 0.4, -0.05, 0.4, -0.6, 0.085)), WHITE)]


def g_potion(x, y):
    flask = min(sd_circle(x, y, 0, -0.2, 0.44), sd_box(x, y, 0, 0.36, 0.14, 0.26, 0.03))
    liquid = max(sd_circle(x, y, 0, -0.2, 0.33), y + 0.14)
    cork = sd_box(x, y, 0, 0.64, 0.2, 0.07, 0.04)
    return [(flask, WHITE), (liquid, RED), (cork, GOLD)]


def g_quest(x, y):
    return [(min(sd_segment(x, y, 0, 0.58, 0, -0.08, 0.14), sd_circle(x, y, 0, -0.5, 0.15)), GOLD)]


def g_bot(x, y):
    head = sd_box(x, y, 0, -0.12, 0.5, 0.38, 0.14)
    eyes = min(sd_circle(x, y, -0.2, -0.04, 0.1), sd_circle(x, y, 0.2, -0.04, 0.1))
    mouth = sd_segment(x, y, -0.18, -0.32, 0.18, -0.32, 0.04)
    antenna = min(sd_segment(x, y, 0, 0.26, 0, 0.52, 0.05), sd_circle(x, y, 0, 0.6, 0.1))
    return [(max(head, -min(eyes, mouth)), WHITE), (antenna, GOLD)]


def g_chat(x, y):
    bubble = min(sd_box(x, y, 0, 0.12, 0.6, 0.38, 0.2),
                 seg(x, y, [(-0.3, -0.2), (-0.42, -0.6), (-0.02, -0.24)], 0.06))
    dots = min(sd_circle(x, y, -0.26, 0.12, 0.08), sd_circle(x, y, 0, 0.12, 0.08),
               sd_circle(x, y, 0.26, 0.12, 0.08))
    return [(max(bubble, -dots), WHITE)]


def g_character(x, y):
    return [(bust(x, y, 0, 0.0, 1.15), WHITE)]


def g_bags(x, y):
    return [(bag_d(x, y), WHITE), (sd_circle(x, y, 0, -0.14, 0.12), GOLD)]


def g_book(x, y):
    cover = sd_box(x, y, 0, 0, 0.5, 0.6, 0.08)
    spine = sd_box(x, y, -0.32, 0, 0.035, 0.6)
    star = min(sd_segment(x, y, 0.1, 0.22, 0.1, -0.22, 0.05), sd_segment(x, y, -0.12, 0, 0.32, 0, 0.05))
    return [(max(cover, -spine), WHITE), (star, BLUE)]


def g_star(x, y):
    # A five-pointed star as the union of its five spikes.
    d = sd_circle(x, y, 0, 0, 0.26)
    for i in range(5):
        a = math.radians(90 + i * 72)
        tx, ty = 0.68 * math.cos(a), 0.68 * math.sin(a)
        # A spike: a stroke that thins toward its tip.
        px, py = x - 0, y - 0
        t = clamp((px * tx + py * ty) / (tx * tx + ty * ty))
        d = min(d, math.hypot(px - tx * t, py - ty * t) - 0.2 * (1 - t))
    return [(d, GOLD)]


def g_log(x, y):
    page = sd_box(x, y, 0, 0, 0.46, 0.62, 0.08)
    lines = min(sd_segment(x, y, -0.24, 0.3, 0.24, 0.3, 0.05),
                sd_segment(x, y, -0.24, 0.02, 0.24, 0.02, 0.05),
                sd_segment(x, y, -0.24, -0.26, 0.08, -0.26, 0.05))
    return [(max(page, -lines), WHITE)]


def g_map(x, y):
    # A compass: a ring and a two-tone needle.
    north = max(abs(x) + abs(y - 0.0) * 0.34 - 0.17, -y)
    south = max(abs(x) + abs(y) * 0.34 - 0.17, y)
    return [(ring_d(x, y, 0, 0, 0.62, 0.075), WHITE), (south, WHITE), (north, RED)]


def g_social(x, y):
    return [(bust(x, y, 0.3, 0.0, 0.8), (0.7, 0.72, 0.75)), (bust(x, y, -0.22, -0.08, 0.95), WHITE)]


def g_hammer(x, y):
    handle = sd_segment(x, y, -0.5, -0.5, 0.2, 0.2, 0.075)
    head = sd_segment(x, y, 0.0, 0.52, 0.5, 0.02, 0.18)
    return [(handle, GOLD), (head, WHITE)]


def g_gear(x, y):
    d = sd_circle(x, y, 0, 0, 0.46)
    for i in range(8):
        a = i * math.pi / 4
        d = min(d, sd_segment(x, y, 0.42 * math.cos(a), 0.42 * math.sin(a),
                              0.64 * math.cos(a), 0.64 * math.sin(a), 0.1))
    return [(max(d, -sd_circle(x, y, 0, 0, 0.2)), WHITE)]


def g_emote(x, y):
    face = sd_circle(x, y, 0, 0, 0.64)
    eyes = min(sd_circle(x, y, -0.24, 0.18, 0.09), sd_circle(x, y, 0.24, 0.18, 0.09))
    smile = max(ring_d(x, y, 0, 0.0, 0.36, 0.055), y + 0.1)
    return [(max(face, -min(eyes, smile)), GOLD)]


def g_follow(x, y):
    # A leader dot with an arrow coming after it.
    return [(arrow(x, y, -0.62, 0, 0.18, 0, 0.09, 0.26), WHITE), (sd_circle(x, y, 0.56, 0, 0.16), GOLD)]


def g_stay(x, y):
    return [(min(sd_box(x, y, -0.24, 0, 0.13, 0.5, 0.05), sd_box(x, y, 0.24, 0, 0.13, 0.5, 0.05)), WHITE)]


def g_flee(x, y):
    lines = min(sd_segment(x, y, 0.3, 0.3, 0.62, 0.3, 0.05), sd_segment(x, y, 0.3, -0.3, 0.62, -0.3, 0.05))
    return [(arrow(x, y, 0.6, 0, -0.6, 0, 0.1, 0.3), WHITE), (lines, GOLD)]


def g_pull(x, y):
    return [(ring_d(x, y, 0.2, -0.2, 0.36, 0.07), RED),
            (arrow(x, y, -0.66, 0.66, 0.12, -0.12, 0.08, 0.24), WHITE)]


def g_summon(x, y):
    return [(min(ring_d(x, y, 0, 0, 0.62, 0.06), ring_d(x, y, 0, 0, 0.36, 0.06)), BLUE),
            (sd_circle(x, y, 0, 0, 0.14), WHITE)]


def g_check(x, y):
    return [(seg(x, y, [(-0.5, 0.0), (-0.14, -0.4), (0.54, 0.44)], 0.13), GREEN)]


def g_loot(x, y):
    return [(bag_d(x, y), GOLD), (min(sd_segment(x, y, 0, 0.06, 0, -0.32, 0.045),
                                      ring_d(x, y, 0, -0.14, 0.14, 0.04)), (0.25, 0.18, 0.05))]


def g_shield(x, y):
    return [(shield_d(x, y), WHITE)]


def g_tank(x, y):
    return [(shield_d(x, y), WHITE), (sword_d(x * 1.5, y * 1.5 + 0.1) / 1.5, RED)]


def g_maxdps(x, y):
    return [(min(seg(x, y, [(-0.45, -0.1), (0, 0.35), (0.45, -0.1)], 0.11),
                 seg(x, y, [(-0.45, -0.55), (0, -0.1), (0.45, -0.55)], 0.11)), RED)]


def g_mana(x, y):
    # A drop: a circle with a pointed top.
    top = max(abs(x) * 1.25 + (y - 0.62) * 0.62, -(y - 0.0))
    return [(min(sd_circle(x, y, 0, -0.18, 0.4), top), BLUE)]


def g_cross(colour):
    return lambda x, y: [(min(sd_box(x, y, 0, 0, 0.16, 0.56, 0.05), sd_box(x, y, 0, 0, 0.56, 0.16, 0.05)), colour)]


def g_dot(x, y):
    return [(sd_circle(x, y, 0, 0, 0.3), WHITE)]


ICONS = {
    # Game actions.
    "AutoRun": g_autorun, "Wheel": g_wheel, "Menu": g_menu, "Jump": g_jump,
    "Interact": g_interact, "Back": g_back, "Inspect": g_inspect,
    "TargetEnemy": g_target(RED), "TargetFriend": g_target(GREEN), "TargetSelf": g_self,
    "Attack": g_attack, "Sit": g_sit, "Consumables": g_potion, "QuestItem": g_quest,
    "BotWheel": g_bot, "QuickChat": g_chat,
    # The window wheel.
    "Character": g_character, "Bags": g_bags, "Spellbook": g_book, "Talents": g_star,
    "QuestLog": g_log, "Map": g_map, "Social": g_social, "Professions": g_hammer,
    "GameMenu": g_gear, "Emote": g_emote,
    # Bot commands.
    "BotFollow": g_follow, "BotStay": g_stay, "BotFlee": g_flee, "BotPull": g_pull,
    "BotSummon": g_summon, "BotAccept": g_check, "BotLoot": g_loot, "BotGuard": g_shield,
    "BotTank": g_tank, "BotMaxDps": g_maxdps, "BotMana": g_mana,
    "BotRelease": g_cross(WHITE), "BotRevive": g_cross(GREEN), "BotOther": g_dot,
}

ART_FILES = {
    "Ring.tga": (128, ring),
    "RingSelect.tga": (128, ring_select),
    "Disc.tga": (64, disc),
}
for _name, _glyph in ICONS.items():
    ART_FILES[_name + ".tga"] = (64, icon(_glyph))


def main():
    os.makedirs(ART, exist_ok=True)
    images = {}
    for name, (size, shade) in ART_FILES.items():
        rows = render(size, shade)
        write_tga(os.path.join(ART, name), rows)
        images[name] = rows
        print("wrote", name)
    if "--preview" in sys.argv:
        # Every image on a mid-grey sheet, 72 px each, ten to a row.
        path = sys.argv[sys.argv.index("--preview") + 1]
        cell, per_row = 72, 10
        names = list(images)
        rows_n = (len(names) + per_row - 1) // per_row
        sheet = [[(0.32, 0.34, 0.3, 1.0)] * (cell * per_row) for _ in range(cell * rows_n)]
        for k, name in enumerate(names):
            rows = images[name]
            n = len(rows)
            ox, oy = (k % per_row) * cell, (k // per_row) * cell
            for py in range(cell - 8):
                for px in range(cell - 8):
                    src = rows[py * n // (cell - 8)][px * n // (cell - 8)]
                    sheet[oy + 4 + py][ox + 4 + px] = over(sheet[oy + 4 + py][ox + 4 + px], src)
        write_png(path, sheet)
        print("preview", path)


if __name__ == "__main__":
    main()
