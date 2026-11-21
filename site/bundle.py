#!/usr/bin/env python3
"""Build the self-contained offline edition; no fonts or remote assets."""

import base64
import gzip
import json
from pathlib import Path
import re

import build

ROOT = Path(__file__).resolve().parent


def pack(value):
    """`value` as JSON, gzipped and base64-encoded."""
    text = json.dumps(value, separators=(",", ":"), ensure_ascii=False)
    return base64.b64encode(gzip.compress(text.encode(), mtime=0)).decode()


def unpack(packed):
    return json.loads(gzip.decompress(base64.b64decode(packed)))


def bundle():
    build.build()
    target = ROOT / "_build"
    pages = {}
    sidebar = ""
    search_dialog = ""
    for path in sorted(target.glob("*.html")):
        page = path.read_text()
        # Shared assets, chrome, and the full-text index are stored once, not per chapter.
        page = re.sub(r'<script id="search-index" type="application/json">.*?</script>', "", page, flags=re.S)
        page = re.sub(r'<link rel="icon"[^>]*>', "", page)
        m_side = re.search(r'<aside class="sidebar" id="sidebar">.*?</aside>', page, flags=re.S)
        if m_side:
            if not sidebar:
                sidebar = m_side.group(0).replace(' aria-current="page"', "")
            page = page[:m_side.start()] + "<!--sidebar-->" + page[m_side.end():]
        m_dlg = re.search(r'<dialog class="search-dialog".*?</dialog>', page, flags=re.S)
        if m_dlg:
            if not search_dialog:
                search_dialog = m_dlg.group(0)
            page = page[:m_dlg.start()] + "<!--search-dialog-->" + page[m_dlg.end():]
        m_course = re.search(r'<script id="course-data" type="application/json">.*?</script>', page, flags=re.S)
        if m_course:
            page = page[:m_course.start()] + "<!--course-data-->" + page[m_course.end():]
        pages[path.name] = page
    full_course_data = build.safe_json(build.course_data("*"))
    # Everything but the compiler is text that gzip shrinks about sixfold, so
    # it ships as one gzip+base64 blob that offline.html unpacks with
    # DecompressionStream, as compiler.js already does for the WASM.
    docs = dict(
        pages=pages,
        sidebar=sidebar,
        search_dialog=search_dialog,
        course_data=full_course_data,
        css=(ROOT / "assets/style.css").read_text(),
        theme=(ROOT / "assets/theme.js").read_text(),
        app=(ROOT / "assets/app.js").read_text(),
        playground=(ROOT / "assets/playground.js").read_text(),
        course=(ROOT / "assets/course.js").read_text(),
        # Entry text repeats each page's prose; offline.html rebuilds it from the pages.
        search=json.dumps([{k: v for k, v in e.items() if k != "text"}
                           for e in json.loads((target / "search.json").read_text())]),
    )
    payload = dict(
        compiler=(target / "assets/compiler.js").read_text(),
        docs=pack(docs),
    )
    shell = (ROOT / "offline.html").read_text()
    shell = shell.replace("{{payload}}", build.safe_json(payload))
    output = ROOT.parent / "dist/cp-lang.html"
    output.parent.mkdir(exist_ok=True)
    output.write_text(shell)
    print(f"Offline edition → {output} ({output.stat().st_size / 1024:.0f} KiB)")
    return output


if __name__ == "__main__":
    bundle()
