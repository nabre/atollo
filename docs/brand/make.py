"""NEXKORA brand assets, variant A (geometric monogram N), from brand.png.

One source for every mark: run `python docs/brand/make.py` to rewrite the
SVGs in docs/brand/ and ui/src/assets/brand/, the PNG logos in ui/src/assets/
and the window icon src-tauri/icons/icon.ico. The PNGs need resvg-py
(`pip install resvg-py`) and Pillow.
"""

import io
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT_DOCS = ROOT / "docs" / "brand"
OUT_UI = ROOT / "ui" / "src" / "assets" / "brand"

# Brand palette (brand.png, "Color palette").
FOREST = "#0F3D2E"
EMERALD = "#10B981"
MINT = "#D1F0E0"
ORANGE = "#F97316"
AMBER = "#FBBF24"
CHARCOAL = "#111827"
CLOUD = "#F8FAFC"

# --- Symbol: the N on a 512 grid -------------------------------------------
# Three strokes, as a folded ribbon: the left bar, the diagonal over it, the
# right bar over the diagonal, which turns from emerald to amber to orange.
LEFT = "120,128 188,128 188,384 120,384"
DIAG = "120,128 196,128 392,384 316,384"
RIGHT = "324,152 392,112 392,384 324,384"
# Shadows where one stroke folds under the next.
FOLD_LEFT = "120,128 188,128 188,217"
FOLD_RIGHT = "324,384 324,295 392,384"


def glyph(fill=None, defs_id="n"):
    """The N strokes; fill=None gives the brand gradients."""
    if fill:
        return (
            f'<polygon points="{LEFT}" fill="{fill}"/>'
            f'<polygon points="{DIAG}" fill="{fill}"/>'
            f'<polygon points="{RIGHT}" fill="{fill}"/>'
        )
    i = defs_id
    return (
        "<defs>"
        f'<linearGradient id="{i}l" x1="0" y1="1" x2="0" y2="0">'
        f'<stop offset="0" stop-color="#0B8A60"/><stop offset="1" stop-color="{EMERALD}"/></linearGradient>'
        f'<linearGradient id="{i}d" x1="0" y1="0" x2="1" y2="1">'
        f'<stop offset="0" stop-color="#34D399"/><stop offset="1" stop-color="{EMERALD}"/></linearGradient>'
        f'<linearGradient id="{i}r" x1="0" y1="1" x2="0" y2="0">'
        f'<stop offset="0" stop-color="{EMERALD}"/><stop offset=".45" stop-color="{AMBER}"/>'
        f'<stop offset="1" stop-color="{ORANGE}"/></linearGradient>'
        "</defs>"
        f'<polygon points="{LEFT}" fill="url(#{i}l)"/>'
        f'<polygon points="{FOLD_LEFT}" fill="#000" opacity=".28"/>'
        f'<polygon points="{DIAG}" fill="url(#{i}d)"/>'
        f'<polygon points="{FOLD_RIGHT}" fill="#000" opacity=".25"/>'
        f'<polygon points="{RIGHT}" fill="url(#{i}r)"/>'
    )


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
    # The app icon: colored N on the deep forest tile.
    "symbol": svg(512, 512, tile("url(#bg)", glyph(), APP_TILE_DEFS), "NEXKORA"),
    # The colored N alone, for any background.
    "symbol-glyph": svg(512, 512, glyph(), "NEXKORA"),
    # One-color variations (brand.png, "Symbol variations").
    "symbol-mono-dark": svg(512, 512, glyph(CHARCOAL), "NEXKORA"),
    "symbol-mono-light": svg(512, 512, glyph("#FFFFFF"), "NEXKORA"),
    "symbol-charcoal": svg(512, 512, tile(CHARCOAL, glyph("#FFFFFF")), "NEXKORA"),
    "symbol-mint": svg(512, 512, tile(MINT, glyph(FOREST)), "NEXKORA"),
}

# --- Wordmark: NEXKORA, geometric capitals, cap height 100 ------------------
S = 22  # stroke


def rect(x0, y0, x1, y1):
    return f"{x0},{y0} {x1},{y0} {x1},{y1} {x0},{y1}"


LETTERS = {
    "N": (84, [rect(0, 0, S, 100), rect(62, 0, 84, 100), "0,0 25,0 84,100 59,100"], []),
    "E": (64, [rect(0, 0, S, 100), rect(0, 0, 64, 21), rect(0, 40, 56, 60), rect(0, 79, 64, 100)], []),
    "X": (90, ["0,0 27,0 90,100 63,100", "63,0 90,0 27,100 0,100"], []),
    "K": (82, [rect(0, 0, S, 100), "16,62 52,0 80,0 38,70", "30,44 82,100 54,100 20,64"], []),
    "O": (100, [], ["M50 0A50 50 0 1 1 50 100A50 50 0 1 1 50 0ZM50 22A28 28 0 1 0 50 78A28 28 0 1 0 50 22Z"]),
    "R": (
        78,
        [rect(0, 0, S, 100), "38,56 62,56 80,100 54,100"],
        ["M0 0H44A31 31 0 0 1 44 62H0ZM22 21V41H43A10 10 0 0 0 43 21Z"],
    ),
    "A": (94, ["0,100 35,0 59,0 94,100 69,100 47,36 25,100"], []),
}
A_TRIANGLE = "47,62 61,100 33,100"  # the orange accent inside the A
GAP = 12


def wordmark(ink, accent=ORANGE):
    x = 0
    parts = []
    for ch in "NEXKORA":
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
    return svg(WM_W, 100, body, "NEXKORA")


def lockup_svg(ink):
    """Horizontal lockup: the colored N, then the wordmark."""
    _, body = wordmark(ink)
    # Glyph box 120..392 x 128..384 scaled to height 100.
    k = 100 / 256
    sym = f'<g transform="translate({-120 * k:.3f} {-128 * k:.3f}) scale({k:.5f})">{glyph(defs_id="lk")}</g>'
    w = 272 * k + 40 + WM_W
    return svg(round(w), 100, sym + f'<g transform="translate({272 * k + 40:.3f} 0)">{body}</g>', "NEXKORA")


WORDMARKS = {
    "wordmark-light": wordmark_svg(FOREST),  # on light backgrounds
    "wordmark-dark": wordmark_svg(CLOUD),  # on dark backgrounds
    "lockup-light": lockup_svg(FOREST),
    "lockup-dark": lockup_svg(CLOUD),
}


# --- Backgrounds: the hero of brand.png, nested folded strokes --------------
# 1600x900, drawn to be cut (preserveAspectRatio slice) by any window shape.


def hero(base0, base1, band_opacity, glow):
    bands = []
    # Nested chevrons rising to the right, as the board's cover.
    for i, (o, w) in enumerate([(0, 150), (190, 120), (350, 90), (480, 60)]):
        x = 820 + o
        pts = f"{x},900 {x + 380},{180 + o * 0.6:.0f} {x + 380 + w},{180 + o * 0.6:.0f} {x + w},900"
        bands.append(f'<polygon points="{pts}" fill="url(#hb)" opacity="{band_opacity * (1 - i * 0.18):.2f}"/>')
        rise = f"{x + 380 + w},{180 + o * 0.6:.0f} {1600},{-120 + o * 0.6:.0f} {1600},{-120 + o * 0.6 + w * 1.4:.0f} {x + 380 + w},{180 + o * 0.6 + w * 1.4:.0f}"
        bands.append(f'<polygon points="{rise}" fill="url(#hw)" opacity="{band_opacity * (1 - i * 0.18):.2f}"/>')
    defs = (
        "<defs>"
        f'<linearGradient id="hg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{base0}"/>'
        f'<stop offset="1" stop-color="{base1}"/></linearGradient>'
        f'<linearGradient id="hb" x1="0" y1="1" x2="0" y2="0"><stop offset="0" stop-color="{EMERALD}" stop-opacity=".2"/>'
        f'<stop offset="1" stop-color="{EMERALD}"/></linearGradient>'
        f'<linearGradient id="hw" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="{AMBER}"/>'
        f'<stop offset="1" stop-color="{ORANGE}"/></linearGradient>'
        f'<radialGradient id="hl" cx=".85" cy=".2" r=".6"><stop offset="0" stop-color="{ORANGE}" stop-opacity="{glow}"/>'
        f'<stop offset="1" stop-color="{ORANGE}" stop-opacity="0"/></radialGradient>'
        "</defs>"
    )
    body = defs + '<rect width="1600" height="900" fill="url(#hg)"/>' + "".join(bands)
    body += '<rect width="1600" height="900" fill="url(#hl)"/>'
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" '
        f'preserveAspectRatio="xMidYMid slice">{body}</svg>\n'
    )


BACKGROUNDS = {
    # On the classic grays of the app (theme.css).
    "hero-dark": hero("#1F1F1F", "#141414", 0.55, 0.2),
    "hero-light": hero("#FFFFFF", "#F2F2F2", 0.28, 0.12),
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


def write_rasters():
    sym = SYMBOLS["symbol"]
    for size in (32, 256):
        render(sym, size).save(ROOT / "ui" / "src" / "assets" / f"logo-{size}.png")
    render(sym, 512).save(OUT_DOCS / "symbol-512.png")
    sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256]
    big = render(sym, 256)
    big.save(
        ROOT / "src-tauri" / "icons" / "icon.ico",
        sizes=[(s, s) for s in sizes],
        append_images=[render(sym, s) for s in sizes[:-1]],
    )


if __name__ == "__main__":
    write_svgs()
    if "--svg-only" not in sys.argv:
        write_rasters()
    print("ok")
