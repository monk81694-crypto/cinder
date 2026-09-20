#!/usr/bin/env python3
"""Render Cinder brand art to raster app icons (stdlib + Pillow + svglib).

Usage:  python tools/make-icons.py
Reads brand/logo.svg (white/ember chevron), composites it on a dark
rounded tile, and writes PNGs + ICO + tray art. Re-run after any logo change.
Requires: pip install svglib reportlab rlPyCairo Pillow
"""
from pathlib import Path
from svglib.svglib import svg2rlg
from reportlab.graphics import renderPM
from reportlab.lib.utils import ImageReader
from PIL import Image, ImageDraw
import io

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "brand" / "logo.svg"
OUT_ICONS = ROOT / "assets" / "icons"
TILE = (16, 16, 36, 255)  # dark indigo tile, matches app rail
SIZES = [16, 32, 48, 64, 128, 256, 512, 1024]


def render_mark(px: int) -> Image.Image:
    d = svg2rlg(str(SRC))
    assert d is not None, f"could not parse {SRC}"
    s = float(px) / float(d.width)
    d.width = px
    d.height = px
    d.scale(s, s)
    buf = io.BytesIO()
    renderPM.drawToFile(d, buf, fmt="PNG", backendFmt="RGBA", bg=None)  # type: ignore[arg-type] — None means transparent, works at runtime
    buf.seek(0)
    return Image.open(buf).convert("RGBA")


def tiled_icon(px: int) -> Image.Image:
    img = Image.new("RGBA", (px, px), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    draw.rounded_rectangle([0, 0, px - 1, px - 1], radius=int(px * 0.24), fill=TILE)
    mark_px = int(px * 0.62)
    mark = render_mark(mark_px)
    img.alpha_composite(mark, (int((px - mark_px) / 2), int((px - mark_px) / 2)))
    return img


def main() -> None:
    OUT_ICONS.mkdir(parents=True, exist_ok=True)
    for n in SIZES:
        p = OUT_ICONS / f"icon-{n}.png"
        tiled_icon(n).save(p)
        print(f"wrote {p.relative_to(ROOT)} {n}x{n}")
    # root assets wired into the app + tests
    tiled_icon(256).save(ROOT / "assets" / "icon.png")
    tiled_icon(512).save(ROOT / "assets" / "logo-512.png")
    print("wrote assets/icon.png 256x256")
    print("wrote assets/logo-512.png 512x512")
    # multi-size Windows icon from the 256 master
    master = Image.open(ROOT / "assets" / "icon.png")
    master.save(ROOT / "assets" / "icon.ico", format="ICO",
                sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print("wrote assets/icon.ico (6 sizes)")
    # tray art (mark on tile, small)
    tiled_icon(22).save(ROOT / "assets" / "tray.png")
    tiled_icon(44).save(ROOT / "assets" / "tray@2x.png")
    print("wrote assets/tray.png + tray@2x.png")
    # macOS iconset folder (convert on a Mac: iconutil -c icns Cinder.iconset)
    iconset = ROOT / "assets" / "Cinder.iconset"
    iconset.mkdir(exist_ok=True)
    for name, size in [("icon_16x16", 16), ("icon_16x16@2x", 32),
                       ("icon_32x32", 32), ("icon_32x32@2x", 64),
                       ("icon_128x128", 128), ("icon_128x128@2x", 256),
                       ("icon_256x256", 256), ("icon_256x256@2x", 512),
                       ("icon_512x512", 512), ("icon_512x512@2x", 1024)]:
        tiled_icon(size).save(iconset / f"{name}.png")
    print("wrote assets/Cinder.iconset/ (10 files; run iconutil on macOS for .icns)")


if __name__ == "__main__":
    main()
