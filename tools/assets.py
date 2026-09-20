#!/usr/bin/env python3
"""Cinder launcher asset checker / fetcher (stdlib only).

Validates art kept in assets/img/ plus root assets, and can
re-download the img files from embedded Wikimedia thumbnail URLs.

Usage:
    python tools/assets.py [--check]
    python tools/assets.py --fetch
"""
from __future__ import annotations

import argparse
import struct
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASSETS_DIR = ROOT / "assets"
IMG_DIR = ASSETS_DIR / "img"

UA = "CinderLauncher/1.0 (+assets-check; stdlib urllib)"

JPEG_MAGIC = b"\xff\xd8\xff"
PNG_MAGIC = b"\x89PNG"
ICO_MAGIC = b"\x00\x00\x01\x00"

KB = 1024

MANIFEST = [
    {
        "rel": "img/news-builds.jpg",
        "magic": JPEG_MAGIC,
        "magic_name": "JPEG FFD8FF",
        "min_size": 20 * KB,
        "kind": "jpeg",
        "url": "https://thumb.wikimedia.org/wikipedia/commons/thumb/b/b2/Minecraft_-_End_city.jpg/1280px-Minecraft_-_End_city.jpg",
    },
    {
        "rel": "img/news-mods.jpg",
        "magic": JPEG_MAGIC,
        "magic_name": "JPEG FFD8FF",
        "min_size": 20 * KB,
        "kind": "jpeg",
        "url": "https://thumb.wikimedia.org/wikipedia/commons/thumb/d/dc/Minecraft_-_Frozen_ocean.jpg/1280px-Minecraft_-_Frozen_ocean.jpg",
    },
    {
        "rel": "img/news-servers.png",
        "magic": PNG_MAGIC,
        "magic_name": "PNG 89504E47",
        "min_size": 20 * KB,
        "kind": "png",
        "url": "https://thumb.wikimedia.org/wikipedia/commons/thumb/6/6a/Minecraft_Trails_and_Tales_Art.png/1280px-Minecraft_Trails_and_Tales_Art.png",
    },
    {
        "rel": "img/stage-bg.png",
        "magic": PNG_MAGIC,
        "magic_name": "PNG 89504E47",
        "min_size": 20 * KB,
        "kind": "png",
        "url": "https://thumb.wikimedia.org/wikipedia/commons/thumb/0/02/Minecraft_-_Deep_Dark.png/1280px-Minecraft_-_Deep_Dark.png",
    },
    {
        "rel": "../brand/logo.svg",
        "magic": None,
        "magic_name": "SVG text (<svg)",
        "min_size": 200,
        "kind": "svg",
    },
    {
        "rel": "../brand/wordmark.svg",
        "magic": None,
        "magic_name": "SVG text (<svg)",
        "min_size": 200,
        "kind": "svg",
    },
    {
        "rel": "icon.ico",
        "magic": ICO_MAGIC,
        "magic_name": "ICO 00000100",
        "min_size": 10 * KB,
        "kind": "ico",
    },
    {
        "rel": "icon.png",
        "magic": PNG_MAGIC,
        "magic_name": "PNG 89504E47",
        "min_size": 1 * KB,
        "kind": "png",
    },
    {
        "rel": "logo-512.png",
        "magic": PNG_MAGIC,
        "magic_name": "PNG 89504E47",
        "min_size": 5 * KB,
        "kind": "png",
    },
]


def png_dimensions(path: Path):
    """Return (width, height) from a PNG IHDR chunk, or None."""
    try:
        with open(path, "rb") as f:
            sig = f.read(8)
            if sig != b"\x89PNG\r\n\x1a\n":
                return None
            while True:
                header = f.read(8)
                if len(header) < 8:
                    return None
                (length, ctype) = struct.unpack(">I4s", header)
                data = f.read(length + 4)
                if len(data) < length + 4:
                    return None
                if ctype == b"IHDR":
                    if length < 8:
                        return None
                    (w, h) = struct.unpack(">II", data[:8])
                    return (w, h)
                if ctype == b"IEND":
                    return None
    except OSError:
        return None


def jpeg_dimensions(path: Path):
    """Return (width, height) by scanning JPEG SOF markers, or None."""
    try:
        data = Path(path).read_bytes()
    except OSError:
        return None
    if len(data) < 4 or data[0:2] != b"\xff\xd8":
        return None
    sof = set([0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7,
               0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF])
    i = 2
    n = len(data)
    while i + 1 < n:
        if data[i] != 0xFF:
            i += 1
            continue
        while i < n and data[i] == 0xFF:
            i += 1
        if i >= n:
            break
        marker = data[i]
        i += 1
        if marker == 0x00 or marker == 0xD8 or marker == 0xD9:
            continue
        if marker == 0x01 or 0xD0 <= marker <= 0xD7:
            continue
        if i + 1 >= n:
            break
        (seg_len,) = struct.unpack(">H", data[i:i + 2])
        if seg_len < 2:
            break
        if marker in sof:
            if i + 7 < n:
                h = struct.unpack(">H", data[i + 3:i + 5])[0]
                w = struct.unpack(">H", data[i + 5:i + 7])[0]
                if w and h:
                    return (w, h)
            break
        i += seg_len
    return None


def get_dimensions(path: Path, kind: str):
    """Dispatch to the struct-based parser for the asset kind."""
    if kind == "png":
        return png_dimensions(path)
    if kind == "jpeg":
        return jpeg_dimensions(path)
    return None


def check_file(entry: dict):
    """Validate one manifest entry. Returns (ok, line)."""
    rel = entry["rel"]
    full = ASSETS_DIR / rel
    min_size = entry["min_size"]
    magic = entry.get("magic")
    kind = entry.get("kind", "")

    if not full.is_file():
        return (False, "FAIL %s: missing (expected %s)" % (rel, full))

    try:
        size = full.stat().st_size
    except OSError as e:
        return (False, "FAIL %s: cannot stat: %s" % (rel, e))

    if size < min_size:
        return (False, "FAIL %s: too small (%d bytes < %d bytes minimum)"
                % (rel, size, min_size))

    if magic is not None:
        try:
            with open(full, "rb") as f:
                head = f.read(4)
        except OSError as e:
            return (False, "FAIL %s: cannot read: %s" % (rel, e))
        if not head.startswith(magic):
            return (False, "FAIL %s: bad magic %s (expected %s, size %d bytes)"
                    % (rel, head[:4].hex(), entry.get("magic_name", "?"), size))
    else:
        try:
            with open(full, "rb") as f:
                head = f.read(8 * KB)
        except OSError as e:
            return (False, "FAIL %s: cannot read: %s" % (rel, e))
        if b"<svg" not in head:
            return (False, "FAIL %s: missing <svg tag (size %d bytes)" % (rel, size))

    dims = get_dimensions(full, kind)
    if dims is None and kind in ("png", "jpeg"):
        return (False, "FAIL %s: could not parse %s dimensions (size %d bytes)"
                % (rel, kind.upper(), size))

    if dims is not None:
        dim_part = " %dx%d" % (dims[0], dims[1])
    else:
        dim_part = ""
    kind_label = kind.upper() if kind else "FILE"
    return (True, "OK %s (%d bytes%s %s)" % (rel, size, dim_part, kind_label))


def check_all() -> int:
    """Validate every manifest entry. Prints OK/FAIL lines + summary."""
    ok = 0
    fail = 0
    for entry in MANIFEST:
        good, line = check_file(entry)
        print(line, flush=True)
        if good:
            ok += 1
        else:
            fail += 1
    print("checked %d files: %d OK, %d FAIL" % (len(MANIFEST), ok, fail), flush=True)
    return 0 if fail == 0 else 1


def fetch_all() -> int:
    """Re-download the four img files via urllib (with UA header)."""
    IMG_DIR.mkdir(parents=True, exist_ok=True)
    failures = 0
    for entry in MANIFEST:
        url = entry.get("url")
        if not url:
            continue
        rel = entry["rel"]
        dest = ASSETS_DIR / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        print("fetch %s -> %s" % (url, rel), flush=True)
        req = urllib.request.Request(url, headers={"User-Agent": UA})
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                data = resp.read()
            tmp = dest.with_name(dest.name + ".tmp")
            with open(tmp, "wb") as f:
                f.write(data)
            # validate the download before replacing the good file
            if len(data) < entry.get("min_size", 0) or not data.startswith(entry.get("magic", b"")):
                tmp.unlink(missing_ok=True)
                print("FAIL %s: downloaded bytes failed validation, kept existing file" % rel, flush=True)
                failures += 1
                continue
            tmp.replace(dest)
            print("saved %s (%d bytes)" % (rel, len(data)), flush=True)
        except Exception as e:
            print("FAIL %s: download error: %s" % (rel, e), flush=True)
            failures += 1
    if failures:
        print("fetch: %d download(s) failed" % failures, flush=True)
    return failures


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description="Check (default) or re-fetch Cinder launcher assets.")
    p.add_argument("--check", action="store_true",
                   help="validate assets (default when no flag given)")
    p.add_argument("--fetch", action="store_true",
                   help="re-download img files, then validate")
    return p


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    if args.fetch:
        dl_failures = fetch_all()
        rc = check_all()
        if dl_failures or rc:
            return 1
        return 0
    return check_all()


if __name__ == "__main__":
    sys.exit(main())
