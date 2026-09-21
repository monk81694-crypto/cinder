#!/usr/bin/env python3
"""Cinder website link + basics checker (stdlib only).
Usage: python website/check.py [--root website]
Exit 0 = all pass, 1 = failures, 2 = site incomplete.
"""
import argparse
import os
import re
import sys
from html.parser import HTMLParser

PAGES = ["index", "features", "download", "changelog", "docs", "faq"]
ALLOWED_HTTP = ("github.com", "api.github.com", "modrinth.com",
                "minotar.net", "mojang.com", "minecraft.net")


class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.title = ""
        self.h1 = 0
        self.meta_desc = False
        self.imgs = []
        self.links = []
        self._in_title = False

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "title":
            self._in_title = True
        elif tag == "h1":
            self.h1 += 1
        elif tag == "meta" and a.get("name") == "description" and (a.get("content") or "").strip():
            self.meta_desc = True
        elif tag == "img":
            self.imgs.append(a.get("alt"))  # None = missing, "" = decorative (OK)
        elif tag == "a" and "href" in a:
            self.links.append(a["href"])
        elif tag in ("link", "script", "source") and ("href" in a or "src" in a):
            self.links.append(a.get("href", a.get("src", "")))

    def handle_data(self, data):
        if self._in_title:
            self.title += data.strip()

    def handle_endtag(self, tag):
        if tag == "title":
            self._in_title = False


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=os.path.join(os.path.dirname(__file__)))
    args = ap.parse_args()
    root = os.path.abspath(args.root)
    fails = []
    missing = [p for p in PAGES if not os.path.isfile(os.path.join(root, p + ".html"))]
    if missing:
        print("MISSING PAGES:", ", ".join(missing))
        return 2
    for p in PAGES:
        path = os.path.join(root, p + ".html")
        src = open(path, encoding="utf-8").read()
        pg = Page()
        pg.feed(src)
        if not pg.title:
            fails.append(f"{p}: empty <title>")
        if pg.h1 != 1:
            fails.append(f"{p}: h1 count = {pg.h1}, want 1")
        if not pg.meta_desc:
            fails.append(f"{p}: meta description missing")
        for i, alt in enumerate(pg.imgs):
            if alt is None:
                fails.append(f"{p}: img #{i} missing alt attribute")
        if "not affiliated with" not in src:
            fails.append(f"{p}: Mojang legal line missing")
        for link in pg.links:
            if not link or link.startswith(("http", "mailto:", "#", "data:")):
                if link.startswith("http") and not any(h in link for h in ALLOWED_HTTP):
                    fails.append(f"{p}: off-allowlist URL {link}")
                continue
            target = os.path.normpath(os.path.join(root, link.split("#")[0]))
            if not os.path.exists(target):
                fails.append(f"{p}: broken link -> {link}")
    print(f"checked {len(PAGES)} pages")
    if fails:
        print("FAIL:")
        for f in fails:
            print(" -", f)
        return 1
    print("ALL CHECKS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
