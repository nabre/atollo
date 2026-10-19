"""Atollo brand assets, atoll ring symbol + wordmark ATOLLO.

One source for every mark: run `python docs/brand/make.py` to rewrite the
SVGs in docs/brand/ and ui/src/assets/brand/, the PNG logos in ui/src/assets/,
the window icons in tauri/icons/ and docs/brand/brand.png. The PNGs need
resvg-py (`pip install resvg-py`) and Pillow.

brand.png is set in Geist and Geist Mono (MARKETING.md, section 6): pass
`--fonts DIR`, a folder with their static TTFs (npm package `geist`,
dist/fonts/geist-sans and geist-mono, without the variable and italic
files). Without it resvg takes the system's fonts.
"""

import io
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT_DOCS = ROOT / "docs" / "brand"
OUT_UI = ROOT / "ui" / "src" / "assets" / "brand"

# Brand palette (docs/brand/MARKETING.md §5.1).
FOREST = "#0F3D2E"
EMERALD = "#10B981"
MINT = "#D1F0E0"
ORANGE = "#F97316"
AMBER = "#FBBF24"
CHARCOAL = "#111827"
CLOUD = "#F8FAFC"

# --- Symbol: the atoll seen from above on a 512 grid ------------------------
# A sunken reef ring (120..392 x 120..392) around a turquoise lagoon, with
# three crescent islets on it, thick in the middle and tapering at the passes:
# the western islet in emerald, the northern one turning from emerald through
# amber to coral orange, and a small coral orange islet to the south-east.
# Angles run clockwise from east, as SVG draws them.
import math

LAGOON = "#5EEAD4"  # shallow turquoise water, only inside the lagoon
ISLETS = {"west": (108, 238, 1.0), "north": (256, 384, 1.0), "south": (48, 90, 0.7)}


class Atoll:
    """One drawing of the atoll: ring center and radius, islet and reef widths,
    and the waves below as (y, first x, crests) with wavelength, amplitude and
    stroke. The full symbol and the small-size one differ only here."""

    def __init__(self, cx, cy, r, islet_w, reef_w, waves, wave_l, wave_a, wave_w):
        self.cx, self.cy, self.r = cx, cy, r
        self.islet_w, self.reef_w = islet_w, reef_w
        self.waves, self.wave_l, self.wave_a, self.wave_w = waves, wave_l, wave_a, wave_w

    def box(self):
        """x0, y0, x1, y1 around the islets and the waves."""
        half = self.r + self.islet_w / 2 + 3
        bottom = max(y for y, _, _ in self.waves) + self.wave_a / 2 + self.wave_w / 2
        return self.cx - half, self.cy - half, self.cx + half, bottom


# The full symbol, on the 512 grid of the app icon, and the one drawn for
# 16..48 pixels: bigger islets, no sunken reef (it blurs into the lagoon) and a
# single thick wave.
FULL = Atoll(256, 236, 112, 52, 12, [(410, 172, 3.5), (442, 208, 2)], 48, 10, 11)
SMALL = Atoll(256, 222, 124, 72, 0, [(438, 160, 3)], 64, 16, 24)


def islet_path(cx, cy, r, a0, a1, w_max, w_min):
    """Closed crescent along a ring, round-tipped by its width profile."""
    n = 48
    outer, inner = [], []
    for k in range(n + 1):
        t = k / n
        w = w_min + (w_max - w_min) * math.sin(math.pi * t) ** 0.6
        a = math.radians(a0 + (a1 - a0) * t)
        c, s = math.cos(a), math.sin(a)
        outer.append(f"{cx + (r + w / 2) * c:.1f} {cy + (r + w / 2) * s:.1f}")
        inner.append(f"{cx + (r - w / 2) * c:.1f} {cy + (r - w / 2) * s:.1f}")
    return "M" + "L".join(outer + inner[::-1]) + "Z"


def wave_path(x0, y, crests, length, amp):
    """A stylized wave from x0 along y, `crests` wavelengths long."""
    d = f"M{x0:g} {y:g}Q{x0 + length / 4:g} {y - amp:g} {x0 + length / 2:g} {y:g}"
    return d + "".join(f"T{x0 + length * k / 2:g} {y:g}" for k in range(2, round(crests * 2) + 1))


def stroked(d, paint, width, opacity=1):
    op = f' opacity="{opacity}"' if opacity != 1 else ""
    return (
        f'<path d="{d}" fill="none" stroke="{paint}" stroke-width="{width:g}" '
        f'stroke-linecap="round" stroke-linejoin="round"{op}/>'
    )


def atoll(a, paints, reef_paint, reef_op, wave_paint, wave_ops, lagoon=None):
    """Lagoon, sunken reef, islets (paints by name) and waves of the Atoll `a`."""
    out = ""
    if lagoon:
        out += f'<circle cx="{a.cx}" cy="{a.cy}" r="{a.r - a.reef_w / 2 - 2:g}" fill="{lagoon}"/>'
    if a.reef_w:
        out += (
            f'<circle cx="{a.cx}" cy="{a.cy}" r="{a.r}" fill="none" stroke="{reef_paint}" '
            f'stroke-width="{a.reef_w}" opacity="{reef_op}"/>'
        )
    for name, (a0, a1, scale) in ISLETS.items():
        p = paints[name]
        d = islet_path(a.cx, a.cy, a.r, a0, a1, a.islet_w * scale, max(a.reef_w, a.islet_w * 0.2))
        out += f'<path d="{d}" fill="{p}" stroke="{p}" stroke-width="6" stroke-linejoin="round"/>'
    for (y, x0, crests), op in zip(a.waves, wave_ops):
        out += stroked(wave_path(x0, y, crests, a.wave_l, a.wave_a), wave_paint, a.wave_w, op)
    return out


def glyph_defs(i):
    """Lagoon and islet gradients, ids prefixed by `i`."""
    return (
        "<defs>"
        f'<radialGradient id="{i}g" cx=".5" cy=".5" r=".5">'
        f'<stop offset="0" stop-color="{LAGOON}" stop-opacity=".42"/>'
        f'<stop offset="1" stop-color="{EMERALD}" stop-opacity=".16"/></radialGradient>'
        f'<linearGradient id="{i}l" x1="0" y1="1" x2="0.4" y2="0">'
        f'<stop offset="0" stop-color="#0B8A60"/><stop offset="1" stop-color="#34D399"/></linearGradient>'
        f'<linearGradient id="{i}r" x1="0" y1="0" x2="1" y2="1">'
        f'<stop offset="0" stop-color="{EMERALD}"/><stop offset=".45" stop-color="{AMBER}"/>'
        f'<stop offset="1" stop-color="{ORANGE}"/></linearGradient>'
        "</defs>"
    )


def glyph(fill=None, defs_id="a", a=FULL, wave=MINT):
    """The Atollo atoll; fill=None gives the brand gradients, with the waves
    in `wave` (mint on dark grounds, emerald on light ones)."""
    if fill:
        return atoll(a, dict.fromkeys(ISLETS, fill), fill, ".45", fill, (".8", ".55"))
    i = defs_id
    paints = {"west": f"url(#{i}l)", "north": f"url(#{i}r)", "south": ORANGE}
    # Mint waves fade on dark grounds, emerald ones need more weight on light ones.
    ops = ("1",) if a is SMALL else (".7", ".4") if wave == MINT else (".9", ".55")
    return glyph_defs(i) + atoll(a, paints, MINT, ".38", wave, ops, f"url(#{i}g)")


def svg(w, h, body, title):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'
        f"<title>{title}</title>{body}</svg>\n"
    )


def tile(fill, inner, defs=""):
    return f'{defs}<rect x="16" y="16" width="480" height="480" rx="112" fill="{fill}"/>{inner}'


APP_TILE_DEFS = (
    '<defs><linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">'
    f'<stop offset="0" stop-color="#14503C"/><stop offset="1" stop-color="#082419"/></linearGradient></defs>'
)

SYMBOLS = {
    # The app icon: colored atoll on the deep forest tile.
    "symbol": svg(512, 512, tile("url(#bg)", glyph(), APP_TILE_DEFS), "Atollo"),
    # The same drawn for 16..48 pixels: favicons, small icon sizes.
    "symbol-small": svg(512, 512, tile("url(#bg)", glyph(a=SMALL), APP_TILE_DEFS), "Atollo"),
    # The colored atoll alone, for any background.
    "symbol-glyph": svg(512, 512, glyph(), "Atollo"),
    # One-color variations.
    "symbol-mono-dark": svg(512, 512, glyph(CHARCOAL), "Atollo"),
    "symbol-mono-light": svg(512, 512, glyph("#FFFFFF"), "Atollo"),
    "symbol-charcoal": svg(512, 512, tile(CHARCOAL, glyph("#FFFFFF")), "Atollo"),
    "symbol-mint": svg(512, 512, tile(MINT, glyph(FOREST)), "Atollo"),
}

# --- Wordmark: ATOLLO, geometric capitals, cap height 100 -------------------
S = 22  # stroke


def rect(x0, y0, x1, y1):
    return f"{x0},{y0} {x1},{y0} {x1},{y1} {x0},{y1}"


LETTERS = {
    "A": (94, ["0,100 35,0 59,0 94,100 69,100 47,36 25,100"], []),
    "T": (76, [rect(0, 0, 76, 21), rect(27, 0, 49, 100)], []),
    "O": (100, [], ["M50 0A50 50 0 1 1 50 100A50 50 0 1 1 50 0ZM50 22A28 28 0 1 0 50 78A28 28 0 1 0 50 22Z"]),
    "L": (64, [rect(0, 0, S, 100), rect(0, 79, 64, 100)], []),
}
A_TRIANGLE = "47,62 61,100 33,100"  # the orange accent inside the A
GAP = 12


def wordmark(ink, accent=ORANGE):
    x = 0
    parts = []
    for ch in "ATOLLO":
        w, polys, paths = LETTERS[ch]
        g = [f'<polygon points="{p}"/>' for p in polys]
        g += [f'<path d="{d}" fill-rule="evenodd"/>' for d in paths]
        if ch == "A":
            g.append(f'<polygon points="{A_TRIANGLE}" fill="{accent}"/>')
        parts.append(f'<g transform="translate({x} 0)">{"".join(g)}</g>')
        x += w + GAP
    return x - GAP, f'<g fill="{ink}">{"".join(parts)}</g>'


WM_W, _ = wordmark(FOREST)


def wordmark_svg(ink):
    _, body = wordmark(ink)
    return svg(WM_W, 100, body, "Atollo")


def lockup_svg(ink, wave):
    """Horizontal lockup: the colored atoll, then the wordmark."""
    _, body = wordmark(ink)
    # Glyph box: the islets' outer edge down to the lower wave, scaled to height 100.
    x0, y0, x1, y1 = FULL.box()
    k = 100 / (y1 - y0)
    gw = (x1 - x0) * k
    sym = f'<g transform="translate({-x0 * k:.3f} {-y0 * k:.3f}) scale({k:.5f})">{glyph(defs_id="lk", wave=wave)}</g>'
    w = gw + 40 + WM_W
    return svg(round(w), 100, sym + f'<g transform="translate({gw + 40:.3f} 0)">{body}</g>', "Atollo")


WORDMARKS = {
    "wordmark-light": wordmark_svg(FOREST),  # on light backgrounds
    "wordmark-dark": wordmark_svg(CLOUD),  # on dark backgrounds
    "lockup-light": lockup_svg(FOREST, EMERALD),
    "lockup-dark": lockup_svg(CLOUD, MINT),
}


# --- Backgrounds: the atoll in its ocean ------------------------------------
# 1600x900, drawn to be cut (preserveAspectRatio slice) by any window shape:
# the left half stays calm for the content, the atoll sits up right, rows of
# the symbol's waves roll in below it and fade out toward the left.
HERO_ATOLL = Atoll(1300, 300, 200, 92, 16, [], 0, 0, 0)
HERO_WAVE_L, HERO_WAVE_A, HERO_WAVE_W = 96, 18, 12


def hero_waves(paint):
    rows = []
    for j, y in enumerate(range(660, 900, 52)):
        x0 = 560 + (j % 2) * HERO_WAVE_L / 2 - j * 20
        crests = (1700 - x0) / HERO_WAVE_L
        rows.append(stroked(wave_path(x0, y, crests, HERO_WAVE_L, HERO_WAVE_A), paint, HERO_WAVE_W))
    return "".join(rows)


def hero(base0, base1, band_opacity, glow, wave):
    i = "ha"
    paints = {"west": f"url(#{i}l)", "north": f"url(#{i}r)", "south": ORANGE}
    defs = (
        "<defs>"
        f'<linearGradient id="hg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{base0}"/>'
        f'<stop offset="1" stop-color="{base1}"/></linearGradient>'
        f'<linearGradient id="hf" gradientUnits="userSpaceOnUse" x1="560" y1="0" x2="1400" y2="0">'
        f'<stop offset="0" stop-color="{wave}" stop-opacity="0"/><stop offset="1" stop-color="{wave}"/></linearGradient>'
        f'<radialGradient id="hl" cx=".85" cy=".2" r=".6"><stop offset="0" stop-color="{ORANGE}" stop-opacity="{glow}"/>'
        f'<stop offset="1" stop-color="{ORANGE}" stop-opacity="0"/></radialGradient>'
        "</defs>"
    )
    body = defs + glyph_defs(i) + '<rect width="1600" height="900" fill="url(#hg)"/>'
    body += '<rect width="1600" height="900" fill="url(#hl)"/>'
    body += f'<g opacity="{band_opacity}">'
    body += atoll(HERO_ATOLL, paints, wave, ".5", wave, (), f"url(#{i}g)")
    body += f'<g opacity=".8">{hero_waves("url(#hf)")}</g></g>'
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" '
        f'preserveAspectRatio="xMidYMid slice">{body}</svg>\n'
    )


BACKGROUNDS = {
    # On the classic grays of the app (theme.css).
    "hero-dark": hero("#1F1F1F", "#141414", 0.45, 0.2, MINT),
    "hero-light": hero("#FFFFFF", "#F2F2F2", 0.32, 0.12, EMERALD),
}


def write_svgs():
    OUT_UI.mkdir(parents=True, exist_ok=True)
    for name, text in {**SYMBOLS, **WORDMARKS, **BACKGROUNDS}.items():
        for d in (OUT_DOCS, OUT_UI):
            (d / f"{name}.svg").write_text(text, encoding="utf-8")


def render(text, size):
    import resvg_py
    from PIL import Image

    png = bytes(resvg_py.svg_to_bytes(svg_string=text, width=size, height=size))
    return Image.open(io.BytesIO(png)).convert("RGBA")


def render_wh(text, w, h):
    import resvg_py
    from PIL import Image

    fonts = {"font_dirs": [FONTS_DIR]} if FONTS_DIR else {}
    png = bytes(resvg_py.svg_to_bytes(svg_string=text, width=w, height=h, **fonts))
    return Image.open(io.BytesIO(png)).convert("RGBA")


# `--fonts DIR`: the Geist TTFs for brand.png (see the docstring).
FONTS_DIR = sys.argv[sys.argv.index("--fonts") + 1] if "--fonts" in sys.argv[:-1] else None


def brand_board_svg():
    """1600x1200 Brand Overview board for Atollo (docs/brand/brand.png)."""
    _, wm_forest = wordmark(FOREST)
    _, wm_cloud = wordmark(CLOUD)
    font = "Geist, system-ui, sans-serif"
    mono = "Geist Mono, monospace"

    # Variant B (Hexagonal Atoll Ring) and Variant C (Connected Nodes) on 512 grid
    var_b = (
        '<defs><linearGradient id="vb1" x1="0" y1="1" x2="1" y2="0">'
        f'<stop offset="0" stop-color="#0B8A60"/><stop offset="1" stop-color="#34D399"/></linearGradient>'
        '<linearGradient id="vb2" x1="0" y1="1" x2="1" y2="0">'
        f'<stop offset="0" stop-color="{EMERALD}"/><stop offset=".5" stop-color="{AMBER}"/>'
        f'<stop offset="1" stop-color="{ORANGE}"/></linearGradient></defs>'
        '<path d="M256 104 L388 180 L388 332 L256 408 L124 332 L124 180 Z '
        'M256 168 L180 212 L180 300 L256 344 L332 300 L332 212 Z" fill="url(#vb1)" fill-rule="evenodd"/>'
        '<polygon points="256,104 388,180 388,332 332,300 332,212 256,168" fill="url(#vb2)"/>'
        f'<polygon points="256,220 296,288 216,288" fill="{ORANGE}"/>'
    )
    var_c = (
        '<defs><linearGradient id="vc1" x1="0" y1="1" x2="1" y2="0">'
        f'<stop offset="0" stop-color="{EMERALD}"/><stop offset=".55" stop-color="{AMBER}"/>'
        f'<stop offset="1" stop-color="{ORANGE}"/></linearGradient></defs>'
        '<path d="M168 344 L256 160 L344 344 M196 288 L316 288" stroke="url(#vc1)" stroke-width="44" stroke-linecap="round" stroke-linejoin="round" fill="none"/>'
        f'<circle cx="256" cy="160" r="42" fill="{ORANGE}"/>'
        f'<circle cx="168" cy="344" r="42" fill="{EMERALD}"/>'
        f'<circle cx="344" cy="344" r="42" fill="{AMBER}"/>'
    )

    swatches = [
        ("Deep Forest", "#0F3D2E", 1146, 434),
        ("Emerald", "#10B981", 1254, 434),
        ("Soft Mint", "#D1F0E0", 1362, 434),
        ("Coral Orange", "#F97316", 1470, 434),
        ("Sun Amber", "#FBBF24", 1146, 540),
        ("Charcoal", "#111827", 1254, 540),
        ("Cloud", "#F8FAFC", 1362, 540),
    ]
    swatch_svg = []
    for label, hex_c, sx, sy in swatches:
        stroke = ' stroke="#E2E8F0" stroke-width="1.5"' if hex_c == "#F8FAFC" else ""
        swatch_svg.append(
            f'<rect x="{sx}" y="{sy}" width="92" height="58" rx="10" fill="{hex_c}"{stroke}/>'
            f'<text x="{sx}" y="{sy + 76}" font-family="{font}" font-size="13" font-weight="600" fill="#1E293B">{label}</text>'
            f'<text x="{sx}" y="{sy + 93}" font-family="{mono}" font-size="12" fill="#64748B">{hex_c}</text>'
        )

    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 1200" width="1600" height="1200">
  <defs>
    <linearGradient id="bg-tile" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#14503C"/>
      <stop offset="1" stop-color="#082419"/>
    </linearGradient>
    <linearGradient id="hero-bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#082017"/>
      <stop offset="0.6" stop-color="#0F3D2E"/>
      <stop offset="1" stop-color="#1F2937"/>
    </linearGradient>
    <linearGradient id="hero-rib1" x1="0" y1="1" x2="1" y2="0">
      <stop offset="0" stop-color="#10B981" stop-opacity="0.15"/>
      <stop offset="1" stop-color="#10B981" stop-opacity="0.85"/>
    </linearGradient>
    <linearGradient id="hero-rib2" x1="0" y1="1" x2="1" y2="0">
      <stop offset="0" stop-color="#10B981"/>
      <stop offset="0.5" stop-color="#FBBF24"/>
      <stop offset="1" stop-color="#F97316"/>
    </linearGradient>
    <clipPath id="hero-clip">
      <rect x="622" y="16" width="490" height="348" rx="18"/>
    </clipPath>
  </defs>

  <!-- Canvas background -->
  <rect width="1600" height="1200" fill="#F8FAFC"/>

  <!-- 1. BRAND OVERVIEW (Top Left) -->
  <rect x="16" y="16" width="590" height="348" rx="18" fill="#FFFFFF" stroke="#E2E8F0" stroke-width="1.5"/>
  <text x="42" y="48" font-family="{font}" font-size="12" font-weight="700" letter-spacing="2" fill="#64748B">BRAND OVERVIEW</text>
  <g transform="translate(42 72) scale(0.84)">{wm_forest}</g>
  <text x="42" y="198" font-family="{font}" font-size="27" font-weight="600" fill="#0F172A">Your sovereign dev &amp; AI workspace.</text>
  <text x="42" y="236" font-family="{font}" font-size="16" fill="#475569">Atollo is a Universal Portable Dev &amp; AI Workspace that unifies</text>
  <text x="42" y="258" font-family="{font}" font-size="16" fill="#475569">editor, toolchains, databases, and AI agents into one portable folder.</text>
  <g font-family="{font}" font-size="14" font-weight="600" fill="#1E293B">
    <rect x="42" y="302" width="28" height="28" rx="6" fill="#D1F0E0"/>
    <text x="48" y="321" font-family="{mono}" font-size="12" font-weight="700" fill="#0F3D2E">&lt;/&gt;</text>
    <text x="78" y="321">Editor</text>

    <rect x="158" y="302" width="28" height="28" rx="6" fill="#D1F0E0"/>
    <text x="165" y="321" font-family="{mono}" font-size="12" font-weight="700" fill="#0F3D2E">&gt;_</text>
    <text x="194" y="321">Terminal</text>

    <rect x="288" y="302" width="28" height="28" rx="6" fill="#D1F0E0"/>
    <circle cx="302" cy="316" r="6" fill="#10B981"/>
    <text x="324" y="321">AI Agents</text>

    <rect x="426" y="302" width="28" height="28" rx="6" fill="#D1F0E0"/>
    <polygon points="440,307 449,316 440,325 431,316" fill="#0F3D2E"/>
    <text x="462" y="321">6 OS/Arch Targets</text>
  </g>

  <!-- 2. HERO BANNER (Top Center) -->
  <g clip-path="url(#hero-clip)">
    <rect x="622" y="16" width="490" height="348" fill="url(#hero-bg)"/>
    <g transform="translate(830 40) scale(0.62)" opacity="0.9">{glyph(defs_id="hb")}</g>
    <g opacity="0.35">{"".join(stroked(wave_path(800 + (j % 2) * 24, 326 + j * 18, 6, 48, 8), "url(#hero-rib1)", 6) for j in range(2))}</g>
    <text x="650" y="52" font-family="{font}" font-size="11" font-weight="700" letter-spacing="2.2" fill="#D1F0E0">UNIVERSAL PORTABLE</text>
    <text x="650" y="70" font-family="{font}" font-size="11" font-weight="700" letter-spacing="2.2" fill="#D1F0E0">DEV &amp; AI WORKSPACE</text>
    <text x="650" y="282" font-family="{font}" font-size="11" font-weight="700" letter-spacing="2" fill="#E2E8F0">CODE</text>
    <text x="650" y="300" font-family="{font}" font-size="11" font-weight="700" letter-spacing="2" fill="#E2E8F0">CONFIGURE</text>
    <text x="650" y="318" font-family="{font}" font-size="11" font-weight="700" letter-spacing="2" fill="#E2E8F0">ORCHESTRATE</text>
    <text x="650" y="336" font-family="{font}" font-size="11" font-weight="700" letter-spacing="2" fill="#34D399">UNPLUG &amp; GO</text>
  </g>

  <!-- 3. BRAND PRINCIPLES (Top Right) -->
  <rect x="1128" y="16" width="456" height="348" rx="18" fill="#FFFFFF" stroke="#E2E8F0" stroke-width="1.5"/>
  <text x="1150" y="48" font-family="{font}" font-size="12" font-weight="700" letter-spacing="2" fill="#64748B">BRAND PRINCIPLES</text>
  <!-- 4 subcards -->
  <rect x="1148" y="64" width="200" height="134" rx="12" fill="#F8FAFC" stroke="#E2E8F0"/>
  <circle cx="1174" cy="92" r="12" fill="#D1F0E0"/><polygon points="1174,83 1182,97 1166,97" fill="#10B981"/>
  <text x="1164" y="128" font-family="{font}" font-size="16" font-weight="700" fill="#0F172A">Sovereign</text>
  <text x="1164" y="148" font-family="{font}" font-size="12.5" fill="#64748B">Zero install, zero admin</text>
  <text x="1164" y="165" font-family="{font}" font-size="12.5" fill="#64748B">rights, zero host trace.</text>

  <rect x="1364" y="64" width="200" height="134" rx="12" fill="#F8FAFC" stroke="#E2E8F0"/>
  <circle cx="1390" cy="92" r="12" fill="#FFEDD5"/><polygon points="1392,82 1384,93 1391,93 1388,102 1398,90 1391,90" fill="#F97316"/>
  <text x="1380" y="128" font-family="{font}" font-size="16" font-weight="700" fill="#0F172A">Polyglot</text>
  <text x="1380" y="148" font-family="{font}" font-size="12.5" fill="#64748B">20+ languages, DBs and</text>
  <text x="1380" y="165" font-family="{font}" font-size="12.5" fill="#64748B">servers on demand.</text>

  <rect x="1148" y="212" width="200" height="134" rx="12" fill="#F8FAFC" stroke="#E2E8F0"/>
  <circle cx="1174" cy="240" r="12" fill="#D1F0E0"/><circle cx="1174" cy="240" r="6" fill="#0F3D2E"/>
  <text x="1164" y="276" font-family="{font}" font-size="16" font-weight="700" fill="#0F172A">Isolated</text>
  <text x="1164" y="296" font-family="{font}" font-size="12.5" fill="#64748B">Protected lagoon across</text>
  <text x="1164" y="313" font-family="{font}" font-size="12.5" fill="#64748B">Win, macOS and Linux.</text>

  <rect x="1364" y="212" width="200" height="134" rx="12" fill="#F8FAFC" stroke="#E2E8F0"/>
  <circle cx="1390" cy="240" r="12" fill="#FFEDD5"/><circle cx="1390" cy="240" r="5" fill="#F97316"/>
  <text x="1380" y="276" font-family="{font}" font-size="16" font-weight="700" fill="#0F172A">Synchronized</text>
  <text x="1380" y="296" font-family="{font}" font-size="12.5" fill="#64748B">Claude, Codex &amp; Antigravity</text>
  <text x="1380" y="313" font-family="{font}" font-size="12.5" fill="#64748B">aligned from one folder.</text>

  <!-- 4. LOGO & ICON SYSTEM (Middle Left) -->
  <rect x="16" y="380" width="710" height="456" rx="18" fill="#FFFFFF" stroke="#E2E8F0" stroke-width="1.5"/>
  <text x="42" y="412" font-family="{font}" font-size="12" font-weight="700" letter-spacing="2" fill="#64748B">LOGO &amp; ICON SYSTEM</text>

  <!-- Variant A -->
  <text x="42" y="446" font-family="{font}" font-size="13" font-weight="700" fill="#0F172A">VARIANT A</text>
  <text x="42" y="466" font-family="{font}" font-size="12.5" fill="#64748B">Atoll reef ring</text>
  <text x="42" y="483" font-family="{font}" font-size="12.5" fill="#64748B">Lagoon, islets and waves.</text>
  <g transform="translate(50 502) scale(0.305)">{tile("url(#bg-tile)", glyph(defs_id="va"))}</g>
  <g transform="translate(44 694) scale(0.095)">{glyph(CHARCOAL)}</g>
  <g transform="translate(106 694) scale(0.095)">{tile(CHARCOAL, glyph("#FFFFFF"))}</g>
  <g transform="translate(168 694) scale(0.095)">{tile(MINT, glyph(FOREST))}</g>
  <text x="42" y="792" font-family="{font}" font-size="11.5" fill="#64748B">Primary Icon · Symbol Variations</text>

  <!-- Variant B -->
  <line x1="254" y1="432" x2="254" y2="808" stroke="#F1F5F9" stroke-width="1.5"/>
  <text x="274" y="446" font-family="{font}" font-size="13" font-weight="700" fill="#0F172A">VARIANT B</text>
  <text x="274" y="466" font-family="{font}" font-size="12.5" fill="#64748B">Atoll lagoon ring &amp; peak</text>
  <text x="274" y="483" font-family="{font}" font-size="12.5" fill="#64748B">Modular and insular.</text>
  <g transform="translate(282 502) scale(0.305)">{tile("url(#bg-tile)", var_b)}</g>
  <g transform="translate(276 694) scale(0.095)">{var_b}</g>
  <g transform="translate(338 694) scale(0.095)">{tile(CHARCOAL, var_b)}</g>
  <g transform="translate(400 694) scale(0.095)">{tile(MINT, var_b)}</g>
  <text x="274" y="792" font-family="{font}" font-size="11.5" fill="#64748B">Primary Icon · Symbol Variations</text>

  <!-- Variant C -->
  <line x1="486" y1="432" x2="486" y2="808" stroke="#F1F5F9" stroke-width="1.5"/>
  <text x="506" y="446" font-family="{font}" font-size="13" font-weight="700" fill="#0F172A">VARIANT C</text>
  <text x="506" y="466" font-family="{font}" font-size="12.5" fill="#64748B">Connected A-nodes</text>
  <text x="506" y="483" font-family="{font}" font-size="12.5" fill="#64748B">Multi-agent orchestration.</text>
  <g transform="translate(514 502) scale(0.305)">{tile("url(#bg-tile)", var_c)}</g>
  <g transform="translate(508 694) scale(0.095)">{var_c}</g>
  <g transform="translate(570 694) scale(0.095)">{tile(CHARCOAL, var_c)}</g>
  <g transform="translate(632 694) scale(0.095)">{tile(MINT, var_c)}</g>
  <text x="506" y="792" font-family="{font}" font-size="11.5" fill="#64748B">Primary Icon · Symbol Variations</text>

  <!-- 5. WORDMARK & LOCKUPS (Middle Center) -->
  <rect x="742" y="380" width="370" height="456" rx="18" fill="#FFFFFF" stroke="#E2E8F0" stroke-width="1.5"/>
  <text x="766" y="412" font-family="{font}" font-size="12" font-weight="700" letter-spacing="2" fill="#64748B">WORDMARK</text>
  <text x="766" y="444" font-family="{font}" font-size="12.5" fill="#64748B">Standalone Wordmark</text>
  <g transform="translate(766 462) scale(0.56)">{wm_forest}</g>

  <line x1="766" y1="538" x2="1088" y2="538" stroke="#F1F5F9" stroke-width="1.5"/>
  <text x="766" y="562" font-family="{font}" font-size="12.5" fill="#64748B">Horizontal Lockup</text>
  <g transform="translate(760 578) scale(0.115)">{glyph(defs_id="hlk", wave=EMERALD)}</g>
  <g transform="translate(830 584) scale(0.44)">{wm_forest}</g>

  <line x1="766" y1="652" x2="1088" y2="652" stroke="#F1F5F9" stroke-width="1.5"/>
  <text x="766" y="676" font-family="{font}" font-size="12.5" fill="#64748B">Stacked Lockup</text>
  <g transform="translate(812 686) scale(0.155)">{glyph(defs_id="slk", wave=EMERALD)}</g>
  <g transform="translate(766 772) scale(0.32)">{wm_forest}</g>

  <line x1="968" y1="652" x2="968" y2="818" stroke="#F1F5F9" stroke-width="1.5"/>
  <text x="984" y="676" font-family="{font}" font-size="12.5" fill="#64748B">Symbol Only</text>
  <g transform="translate(984 696) scale(0.18)">{tile("url(#bg-tile)", glyph(defs_id="so"))}</g>

  <!-- 6. COLOR PALETTE & TYPOGRAPHY (Middle Right) -->
  <rect x="1128" y="380" width="456" height="456" rx="18" fill="#FFFFFF" stroke="#E2E8F0" stroke-width="1.5"/>
  <text x="1146" y="412" font-family="{font}" font-size="12" font-weight="700" letter-spacing="2" fill="#64748B">COLOR PALETTE</text>
  {"".join(swatch_svg)}

  <line x1="1146" y1="654" x2="1564" y2="654" stroke="#F1F5F9" stroke-width="1.5"/>
  <text x="1146" y="680" font-family="{font}" font-size="12" font-weight="700" letter-spacing="2" fill="#64748B">TYPOGRAPHY</text>
  <text x="1146" y="704" font-family="{font}" font-size="12.5" fill="#64748B">Primary Typeface</text>
  <text x="1146" y="736" font-family="{font}" font-size="25" font-weight="700" fill="#0F172A">Geist</text>
  <text x="1146" y="764" font-family="{font}" font-size="12.5" fill="#475569">Aa Bb Cc Dd Ee Ff Gg</text>
  <text x="1146" y="782" font-family="{font}" font-size="12.5" fill="#475569">Hh Ii Jj Kk Ll Mm Nn</text>
  <text x="1146" y="800" font-family="{font}" font-size="12.5" fill="#475569">0123456789</text>

  <line x1="1356" y1="690" x2="1356" y2="818" stroke="#F1F5F9" stroke-width="1.5"/>
  <text x="1372" y="704" font-family="{font}" font-size="12.5" fill="#64748B">Monospace</text>
  <text x="1372" y="736" font-family="{mono}" font-size="22" font-weight="600" fill="#0F172A">Geist Mono</text>
  <text x="1372" y="764" font-family="{mono}" font-size="12.5" fill="#475569">Aa Bb Cc Dd Ee Ff Gg</text>
  <text x="1372" y="782" font-family="{mono}" font-size="12.5" fill="#475569">win32-x64 darwin-arm64</text>
  <text x="1372" y="800" font-family="{mono}" font-size="12.5" fill="#475569">0123456789</text>

  <!-- 7. TYPE SCALE EXAMPLE (Bottom Left) -->
  <rect x="16" y="852" width="540" height="332" rx="18" fill="#FFFFFF" stroke="#E2E8F0" stroke-width="1.5"/>
  <text x="42" y="884" font-family="{font}" font-size="12" font-weight="700" letter-spacing="2" fill="#64748B">TYPE SCALE EXAMPLE</text>
  <text x="42" y="924" font-family="{font}" font-size="12" fill="#64748B">H1</text>
  <text x="96" y="928" font-family="{font}" font-size="28" font-weight="700" fill="#0F172A">Your sovereign workspace.</text>
  <text x="42" y="968" font-family="{font}" font-size="12" fill="#64748B">H2</text>
  <text x="96" y="972" font-family="{font}" font-size="22" font-weight="600" fill="#0F172A">Code. Configure. Unplug.</text>
  <text x="42" y="1008" font-family="{font}" font-size="12" fill="#64748B">H3</text>
  <text x="96" y="1012" font-family="{font}" font-size="18" font-weight="500" fill="#1E293B">Six architectures in one folder.</text>
  <text x="42" y="1050" font-family="{font}" font-size="12" fill="#64748B">Body</text>
  <text x="96" y="1050" font-family="{font}" font-size="14" fill="#475569">Atollo unifies VSCodium, 20+ on-demand toolchains, local</text>
  <text x="96" y="1070" font-family="{font}" font-size="14" fill="#475569">databases, and synchronized AI agents with zero host footprint.</text>
  <text x="42" y="1132" font-family="{font}" font-size="12" fill="#64748B">Mono</text>
  <rect x="96" y="1104" width="432" height="46" rx="10" fill="#111827"/>
  <text x="116" y="1132" font-family="{mono}" font-size="15" font-weight="600" fill="#34D399">$ atollo sync &amp;&amp; atollo check</text>

  <!-- 8. PRODUCT APPLICATION PREVIEW (Bottom Right) -->
  <rect x="572" y="852" width="1012" height="332" rx="18" fill="#FFFFFF" stroke="#E2E8F0" stroke-width="1.5"/>
  <text x="598" y="884" font-family="{font}" font-size="12" font-weight="700" letter-spacing="2" fill="#64748B">PRODUCT APPLICATION PREVIEW</text>
  <!-- Dark IDE window -->
  <rect x="594" y="898" width="968" height="270" rx="12" fill="#1F1F1F" stroke="#2B2B2B" stroke-width="1.5"/>
  <rect x="594" y="898" width="968" height="32" rx="12" fill="#181818"/>
  <circle cx="614" cy="914" r="5" fill="#F14C4C"/>
  <circle cx="630" cy="914" r="5" fill="#CCA700"/>
  <circle cx="646" cy="914" r="5" fill="#10B981"/>
  <g transform="translate(664 906) scale(0.032)">{tile("url(#bg-tile)", glyph(defs_id="pw"))}</g>
  <g transform="translate(686 907) scale(0.13)">{wm_cloud}</g>
  <text x="774" y="918" font-family="{font}" font-size="12" fill="#9D9D9D">— Universal Portable Dev &amp; AI Workspace</text>

  <!-- Sidebar -->
  <rect x="594" y="930" width="176" height="218" fill="#181818"/>
  <rect x="604" y="942" width="156" height="26" rx="6" fill="#0B3D2C"/>
  <text x="618" y="959" font-family="{font}" font-size="12.5" font-weight="600" fill="#D1F0E0">Progetti (3)</text>
  <text x="618" y="988" font-family="{font}" font-size="12.5" fill="#CCCCCC">Agenti AI (Sync)</text>
  <text x="618" y="1014" font-family="{font}" font-size="12.5" fill="#CCCCCC">Database &amp; Server</text>
  <text x="618" y="1040" font-family="{font}" font-size="12.5" fill="#CCCCCC">Strumenti (6/6)</text>
  <text x="618" y="1066" font-family="{font}" font-size="12.5" fill="#CCCCCC">Guida &amp; Impostazioni</text>

  <!-- Editor & Terminal Center -->
  <text x="790" y="956" font-family="{mono}" font-size="12.5" fill="#6EE7B7">// Portable project environment (win32-x64 · darwin-arm64 · linux-x64)</text>
  <text x="790" y="978" font-family="{mono}" font-size="12.5" fill="#CCCCCC">export const workspace = {{</text>
  <text x="808" y="998" font-family="{mono}" font-size="12.5" fill="#34D399">name: "Atollo",</text>
  <text x="808" y="1018" font-family="{mono}" font-size="12.5" fill="#FBBF24">agents: ["claude-code", "codex", "antigravity"],</text>
  <text x="808" y="1038" font-family="{mono}" font-size="12.5" fill="#CCCCCC">isolatedPath: true,</text>
  <text x="790" y="1058" font-family="{mono}" font-size="12.5" fill="#CCCCCC">}};</text>
  <line x1="770" y1="1074" x2="1280" y2="1074" stroke="#2B2B2B"/>
  <text x="790" y="1096" font-family="{mono}" font-size="12" fill="#10B981">✓ PATH isolated to kit/data · 0 host services</text>
  <text x="790" y="1116" font-family="{mono}" font-size="12" fill="#10B981">✓ Claude Code, Codex &amp; Antigravity MCP/skills synchronized</text>
  <text x="790" y="1136" font-family="{mono}" font-size="12" fill="#34D399">✓ Ready on win32-x64, win32-arm64, darwin, linux</text>

  <!-- Right AI Panel -->
  <rect x="1280" y="930" width="282" height="218" fill="#181818"/>
  <text x="1298" y="956" font-family="{font}" font-size="12.5" font-weight="700" fill="#D1F0E0">Atollo AI Agent Hub</text>
  <rect x="1296" y="970" width="250" height="44" rx="6" fill="#202020" stroke="#2B2B2B"/>
  <text x="1310" y="989" font-family="{font}" font-size="12" font-weight="600" fill="#34D399">Claude Code · Codex · Antigravity</text>
  <text x="1310" y="1005" font-family="{font}" font-size="11" fill="#9D9D9D">Unified AGENTS.md, skills &amp; MCP</text>
  <rect x="1296" y="1022" width="250" height="44" rx="6" fill="#202020" stroke="#2B2B2B"/>
  <text x="1310" y="1041" font-family="{font}" font-size="12" font-weight="600" fill="#FBBF24">Multi-OS Router (6/6)</text>
  <text x="1310" y="1057" font-family="{font}" font-size="11" fill="#9D9D9D">Native binaries per host target</text>

  <!-- Emerald Status Bar -->
  <rect x="594" y="1146" width="968" height="22" rx="4" fill="#10B981"/>
  <rect x="594" y="1146" width="84" height="22" rx="4" fill="#0F3D2E"/>
  <text x="610" y="1161" font-family="{font}" font-size="11.5" font-weight="700" fill="#D1F0E0">Atollo</text>
  <text x="1420" y="1161" font-family="{mono}" font-size="11" font-weight="600" fill="#04140D">win32-x64 · v0.2.0</text>
</svg>
"""


def icon(size):
    """The app icon at `size` pixels: the small-size drawing up to 48."""
    return render(SYMBOLS["symbol-small" if size <= 48 else "symbol"], size)


def write_rasters():
    icons = ROOT / "tauri" / "icons"
    for size in (32, 256):
        icon(size).save(ROOT / "ui" / "src" / "assets" / f"logo-{size}.png")
    icon(32).save(ROOT / "ui" / "public" / "favicon.png")
    icon(256).save(icons / "icon.png")
    icon(512).save(OUT_DOCS / "symbol-512.png")
    render_wh(brand_board_svg(), 1600, 1200).save(OUT_DOCS / "brand.png")
    sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256]
    icon(256).save(
        icons / "icon.ico",
        sizes=[(s, s) for s in sizes],
        append_images=[icon(s) for s in sizes[:-1]],
    )
    icon(32).save(icons / "32x32.png")
    icon(128).save(icons / "128x128.png")
    icon(256).save(icons / "128x128@2x.png")
    icon(1024).save(icons / "icon.icns")


if __name__ == "__main__":
    write_svgs()
    if "--svg-only" not in sys.argv:
        write_rasters()
    print("ok")


