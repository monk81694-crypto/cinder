# Cinder Electron Launcher - Raster Pipeline Proof

Date: 2026-09-20 UTC
Input: brand/concepts/b-flint-angle.svg (317 bytes)
Output dir: assets/icons-tmp/
Workdir for all commands below: repo root (folder containing brand/ and assets/)

## 1. Input read first
SVG content (5 lines, viewBox 0 0 64 64, no width or height attributes):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" role="img">
  <title>Flint Angle</title>
  <polyline points="16,16 40,32" fill="none" stroke="#1A1D21" stroke-width="8" stroke-linecap="square"/>
  <polyline points="40,32 16,48" fill="none" stroke="#FF6B1A" stroke-width="8" stroke-linecap="square"/>
</svg>
```

Note: svg2rlg reports drawing size 48.0 x 48.0 for this file (default when width and height are absent). Scaling below is relative to that parsed size.

## 2. Renderer that worked

Primary path WORKED: svglib + reportlab + rlPyCairo. Fallback cairosvg was NOT needed and was NOT installed.

| Component | Version | Note |
|---|---|---|
| Python | 3.12.10 | windows-amd64 |
| Pillow | 11.3.0 | preinstalled, used for verify and ICO |
| svglib | 2.2.0 | via pip install svglib reportlab |
| reportlab | 5.0.1 | via pip install svglib reportlab |
| rlPyCairo | 0.4.0 | via pip install rlPyCairo, REQUIRED by reportlab 5 renderPM |
| pycairo | 1.29.1 | pulled in as rlPyCairo dependency (win_amd64 wheel) |
| cairosvg | not installed | fallback not attempted because primary succeeded |

Why rlPyCairo was needed: reportlab 5.0.1 renderPM.drawToFile defaults to backend rlPyCairo. Without it the call fails with RenderPMError cannot import desired renderPM backend rlPyCairo. Installing rlPyCairo fixed it. No hand-drawn art was used.

## 3. Exact commands run (in order, from repo root)

```powershell
pip install svglib reportlab
pip install rlPyCairo
New-Item -ItemType Directory -Path "assets/icons-tmp" -Force
python assets/icons-tmp/render.py
python assets/icons-tmp/verify.py
python assets/icons-tmp/make_ico.py
```

Helper scripts were created under assets/icons-tmp/ only (allowed output dir) and are the rerunnable pipeline:

- assets/icons-tmp/render.py renders icon-16.png through icon-1024.png plus tray.png and tray@2x.png
- assets/icons-tmp/verify.py prints Pillow size, mode, byte size for each PNG
- assets/icons-tmp/make_ico.py builds icon-test.ico from icon-256.png with sizes 16,32,48,64,128,256

Core render logic per size N (same for tray sizes 22 and 44):

```python
from svglib.svglib import svg2rlg
from reportlab.graphics import renderPM
d = svg2rlg("brand/concepts/b-flint-angle.svg")  # reload fresh each size
s = float(N) / float(d.width)
d.width = N
d.height = N
d.scale(s, s)
renderPM.drawToFile(d, "assets/icons-tmp/icon-%d.png" % N, fmt="PNG", backendFmt="RGBA", bg=None)
```

ICO build logic:

```python
from PIL import Image
im = Image.open("assets/icons-tmp/icon-256.png")
im.save("assets/icons-tmp/icon-test.ico", format="ICO", sizes=[(16,16),(32,32),(48,48),(64,64),(128,128),(256,256)])
```

Transparency: backendFmt RGBA with bg None yields RGBA PNGs with transparent background. Default RGB path yields white background and was NOT used.

## 4. Per-size verification (Pillow 11.3.0, actual pixel dimensions)

Command: python assets/icons-tmp/verify.py (opens each PNG with PIL.Image.open and prints size, mode, byte length)

| File | Expected px | Actual px | Mode | Bytes | Status |
|---|---|---|---|---|---|
| assets/icons-tmp/icon-16.png | 16x16 | 16x16 | RGBA | 303 | PASS |
| assets/icons-tmp/icon-32.png | 32x32 | 32x32 | RGBA | 590 | PASS |
| assets/icons-tmp/icon-48.png | 48x48 | 48x48 | RGBA | 791 | PASS |
| assets/icons-tmp/icon-64.png | 64x64 | 64x64 | RGBA | 963 | PASS |
| assets/icons-tmp/icon-128.png | 128x128 | 128x128 | RGBA | 1548 | PASS |
| assets/icons-tmp/icon-256.png | 256x256 | 256x256 | RGBA | 2827 | PASS |
| assets/icons-tmp/icon-512.png | 512x512 | 512x512 | RGBA | 5856 | PASS |
| assets/icons-tmp/icon-1024.png | 1024x1024 | 1024x1024 | RGBA | 12330 | PASS |
| assets/icons-tmp/tray.png | 22x22 | 22x22 | RGBA | 468 | PASS |
| assets/icons-tmp/tray@2x.png | 44x44 | 44x44 | RGBA | 672 | PASS |

All 10 PNGs verified at exact pixel dimensions with transparent RGBA mode.

## 5. ICO verification

Command: python assets/icons-tmp/make_ico.py plus struct header parse and Pillow info check.

- File: assets/icons-tmp/icon-test.ico
- Byte size: 9574
- Magic bytes: 00 00 01 00 (first 4 bytes, reserved 0, type 1 ICO) - PASS
- Image count from header: 6 - PASS
- Pillow info sizes: 16x16, 32x32, 48x48, 64x64, 128x128, 256x256 - PASS

| Entry | Size | Bit depth | Payload bytes | Offset |
|---|---|---|---|---|
| 0 | 16x16 | 32 | 493 | 102 |
| 1 | 32x32 | 32 | 803 | 595 |
| 2 | 48x48 | 32 | 1302 | 1398 |
| 3 | 64x64 | 32 | 1535 | 2700 |
| 4 | 128x128 | 32 | 2512 | 4235 |
| 5 | 256x256 | 32 | 2827 | 6747 |

Source for ICO: assets/icons-tmp/icon-256.png. Pillow resized from that source into the 6 requested sizes. Total file 9574 bytes.

## 6. Notes for the integrator

### How to re-run on final brand/logo.svg

1. In assets/icons-tmp/render.py change SRC from brand/concepts/b-flint-angle.svg to brand/logo.svg.
2. Delete test outputs or point OUTDIR to a fresh folder, then run:

```powershell
python assets/icons-tmp/render.py
python assets/icons-tmp/verify.py
python assets/icons-tmp/make_ico.py
```

3. If the final SVG has explicit width and height, the same scale math still holds because s equals N divided by parsed d.width.
4. For production, copy and rename: icon-256.png to assets/icon.png expectation, icon-test.ico to assets/icon.ico expectation, after visual review. Do NOT overwrite existing assets until final art is approved.
5. Keep backendFmt RGBA and bg None for transparency. If a solid background is ever required, remove those two arguments to get white RGB output.

### macOS .icns cannot be built on Windows

icns requires macOS iconutil, so from Windows only deliver an iconset folder plus a build script. Suggested layout from the PNGs above:

```text
assets/icons-tmp/Cinder.iconset/icon_16x16.png       <- copy of icon-16.png
assets/icons-tmp/Cinder.iconset/icon_16x16@2x.png    <- copy of icon-32.png
assets/icons-tmp/Cinder.iconset/icon_32x32.png       <- copy of icon-32.png
assets/icons-tmp/Cinder.iconset/icon_32x32@2x.png    <- copy of icon-64.png
assets/icons-tmp/Cinder.iconset/icon_128x128.png     <- copy of icon-128.png
assets/icons-tmp/Cinder.iconset/icon_128x128@2x.png  <- copy of icon-256.png
assets/icons-tmp/Cinder.iconset/icon_256x256.png     <- copy of icon-256.png
assets/icons-tmp/Cinder.iconset/icon_256x256@2x.png  <- copy of icon-512.png
assets/icons-tmp/Cinder.iconset/icon_512x512.png     <- copy of icon-512.png
assets/icons-tmp/Cinder.iconset/icon_512x512@2x.png  <- copy of icon-1024.png
```

Then on macOS run:

```bash
iconutil -c icns assets/icons-tmp/Cinder.iconset -o assets/icon.icns
```

This proof did NOT create the iconset or icns, to stay within requested outputs.

### Protected files untouched

No writes outside assets/icons-tmp/ and brand/PIPELINE_RESULT.md. Verified still present with prior timestamps:

- assets/icon.ico (61501 bytes)
- assets/icon.png (33324 bytes)
- assets/logo.svg (4323 bytes)
- assets/mark.svg (2341 bytes)
- assets/logo-512.png (81203 bytes)

### Anything unverified

- Visual fidelity at 16px and tray sizes was confirmed only as pixel dimensions and RGBA mode, not by human design review.
- Electron packaging with the test ICO was not run.
- icns output was not built because iconutil needs macOS.
