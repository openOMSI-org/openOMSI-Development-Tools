#!/usr/bin/env python3
"""Writes apps/devtools/ui/js/symbols.js: the Material Symbols (Rounded, filled, Apache-2.0) the
page uses, by the page's own names. Each glyph is fetched once from
https://fonts.gstatic.com/s/i/short-term/release/materialsymbolsrounded/<name>/fill1/48px.svg
and kept in tools/.symbols-cache (not committed).

    python3 tools/symbols.py
"""
import json, re, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE / ".symbols-cache"
OUT = HERE.parent / "apps" / "devtools" / "ui" / "js" / "symbols.js"

NAMES = {
    "home": "home", "check": "check", "code": "code", "folder": "folder", "folder_open": "folder_open", "file": "description",
    "file_add": "note_add", "folder_add": "create_new_folder", "plus": "add", "play": "play_arrow",
    "build": "construction", "key": "key", "search": "search", "book": "menu_book", "settings": "settings",
    "ok": "check_circle", "error": "error", "warn": "warning", "x": "close", "refresh": "refresh",
    "download": "download", "upload": "upload_file", "trash": "delete", "edit": "edit", "copy": "content_copy",
    "external": "open_in_new", "back": "arrow_back", "down": "expand_more", "right": "chevron_right",
    "left": "chevron_left", "bolt": "bolt", "verified": "verified", "fingerprint": "fingerprint",
    "package": "deployed_code", "rocket": "rocket_launch", "terminal": "terminal", "bug": "bug_report",
    "tune": "tune", "shield": "shield", "info": "info", "save": "save", "image": "image", "music": "music_note",
    "data": "data_object", "sync": "sync", "lock": "lock", "help": "help", "task": "task_alt", "stop": "stop",
    "bus": "directions_bus", "inspect": "manage_search", "sparkle": "auto_awesome", "pen": "draw",
    "unlock": "lock_open", "list": "list", "wrench": "build", "dot": "fiber_manual_record", "up": "expand_less",
    "gamepad": "sports_esports", "article": "article", "palette": "palette", "update": "system_update_alt",
}

CACHE.mkdir(exist_ok=True)
paths = {}
for key, name in NAMES.items():
    f = CACHE / f"{name}.svg"
    if not f.exists():
        url = f"https://fonts.gstatic.com/s/i/short-term/release/materialsymbolsrounded/{name}/fill1/48px.svg"
        f.write_bytes(urllib.request.urlopen(url, timeout=30).read())
    svg = f.read_text()
    paths[key] = " ".join(re.findall(r'<path[^>]* d="([^"]+)"', svg))
OUT.write_text(
    "// Google's Material Symbols Rounded (filled, Apache-2.0), by the page's own names. Made by\n"
    "// tools/symbols.py; viewBox 0 -960 960 960.\n"
    "export const SYMBOLS = " + json.dumps(paths, indent=1) + ";\n"
)
print(f"{len(paths)} symbols -> {OUT}")
