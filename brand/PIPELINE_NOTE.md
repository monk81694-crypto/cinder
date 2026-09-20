# Brand -> Raster Icon Pipeline Note (STOP - no source art)

Date (UTC): 2026-09-20
Repo: Cinder Electron launcher

## Status: STOPPED - brand/ is empty

Checked on run:
- Test-Path brand/ = False (did not exist before this note; created brand/ only to hold this file)
- brand/*.svg = NO FILES FOUND
- assets/ contains only: img/, icon.ico (61501 B), icon.png (33324 B), logo-512.png (81203 B), logo.svg (4323 B), mark.svg (2341 B)

Per task instructions: if brand/ is empty, create this file describing the plan and STOP.
Did NOT invent logo art. Did NOT hand-draw anything in Pillow.
Did NOT overwrite protected files: assets/icon.ico, assets/icon.png, assets/logo.svg, assets/mark.svg, assets/logo-512.png
  (all wired into app and tests: main.js:207-208, index.html:7,50,578, package.json build.win.icon / build.linux.icon,
   tests/smoke.js:179,183-187, tools/assets.py:65-93, tools/release/main.go:17-21, README.md:3,115,182-183).
Did NOT create assets/icons-tmp/ (no source SVG to render; creating empty PNGs from invented art is forbidden).

## Environment verified on run (read-only, no installs)

1. pip show svglib reportlab -> WARNING: Package(s) not found: reportlab, svglib
2. python -c "import PIL; print(PIL.__version__)" -> 11.3.0 (Pillow present as expected)
3. pip install was NOT attempted: no source SVG exists so rendering is impossible regardless of libraries,
   and the STOP condition takes precedence over installing render deps. Honest report: install step skipped for this reason.

## Plan once winning brand/*.svg lands

### Method 1 (preferred): svglib + reportlab rasterization
Commands to run (in order, stop at first method that works):
1. pip show svglib reportlab
2. If missing (current state): pip install svglib reportlab (90s timeout only, one attempt; on failure move to Method 2)
3. Render each brand/*.svg to PNG at 16,32,48,64,128,256,512,1024 into assets/icons-tmp/ (new files only),
   e.g. assets/icons-tmp/<basename>-16.png ... <basename>-1024.png
   Suggested: svglib.svg2rlg + reportlab renderPM.renderToFile(..., fmt="PNG") per size, or render at 1024 then downscale.
   Exact script to be recorded in final report on that run.
4. Inspect one output size, e.g.:
   python -c "from PIL import Image; im=Image.open('assets/icons-tmp/<basename>-512.png'); print(im.size, im.mode)"

### Method 2 (fallback): Pillow-only assembly (NO SVG rasterization possible)
Pillow cannot rasterize SVG. This path can ONLY assemble, and ONLY if Method 1 already produced PNGs:
- IMPORTANT CONFLICT NOTE: task step 2 says "save assets/icon.ico ... plus assets/icon.iconset ... plus assets/tray.png",
  but task step 3 forbids overwriting assets/icon.ico (also wired into electron-builder + smoke tests).
  Therefore on a future run: do NOT write assets/icon.ico directly.
  Write instead to new paths only, e.g.:
  - assets/icons-tmp/icon.ico (Pillow multi-size 16..256: img.save(..., sizes=[(16,16),(32,32),(48,48),(64,64),(128,128),(256,256)]))
  - assets/icons-tmp/icon.iconset/icon_16x16.png ... icon_512x512@2x.png (copy/rename from step-1 PNGs)
  - assets/icons-tmp/tray.png (22px) and assets/icons-tmp/tray@2x.png (44px)
  Then propose a separate rename PR after review, never silent overwrite.
- Do NOT hand-draw new art in Pillow to fake missing brand SVGs.

### Verification required on a future raster run
- ICO magic: first 4 bytes must be 00 00 01 00, e.g.:
  python -c "d=open('assets/icons-tmp/icon.ico','rb').read(4); print(d.hex(' ')); assert d==bytes.fromhex('00 00 01 00')"
- List every file created with byte sizes, e.g.:
  Get-ChildItem -Recurse assets/icons-tmp, assets/icon.iconset | Format-Table FullName, Length
- Keep protected files untouched; verify with git status that only new paths are added.

## What remains unverified (even after a future successful raster run)
- .icns conversion needs macOS iconutil (iconset -> icon.icns); cannot verify on Windows. State this in final report.
- Visual QA of rendered PNGs at small sizes (16/32px hinting) needs human review.
- electron-builder win/linux icon wiring after any future rename needs a full dist build test (not done here).
- This note run created exactly ONE file (this file); no icons, no .ico, no PNGs were produced.

## Addendum (same run, ~23:07 UTC): brand/concepts/ appeared concurrently

After this note was written, a re-list of brand/ showed a new untracked subfolder NOT created by this task:
- brand/concepts/a-ember-bars.svg (279 B, LastWriteTime 2026-09-20 23:07:08)
- brand/concepts/a-kindling.svg (221 B, LastWriteTime 2026-09-20 23:07:08)
- brand/concepts/a-spark-gap.svg (213 B, LastWriteTime 2026-09-20 23:07:08)
- brand/ itself was created by this task at ~23:06 to hold this note; concepts/ timestamps are later.

These are concept candidates in a subfolder, NOT top-level brand/*.svg winners, and no winning selection is declared.
Per STOP instruction they were NOT rasterized: no assets/icons-tmp/ created, no PNG/ICO/ICNS written, no Pillow hand-drawn art.
Read-only check only: existing assets/icon.ico header = 00 00 01 00 (valid Windows icon magic, pre-existing file untouched).
Next step for a future run: once a winner is copied to brand/*.svg top-level, re-run Method 1 above against that file.
