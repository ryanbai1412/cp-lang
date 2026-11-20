#!/usr/bin/env python3
"""Build the self-contained offline edition; no fonts or remote assets."""

from pathlib import Path
import re

import build

ROOT = Path(__file__).resolve().parent


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
        pages[path.name] = page
    payload = dict(
        pages=pages,
        sidebar=sidebar,
        search_dialog=search_dialog,
        css=(ROOT / "assets/style.css").read_text(),
        theme=(ROOT / "assets/theme.js").read_text(),
        app=(ROOT / "assets/app.js").read_text(),
        playground=(ROOT / "assets/playground.js").read_text(),
        course=(ROOT / "assets/course.js").read_text(),
        compiler=(target / "assets/compiler.js").read_text(),
        search=(target / "search.json").read_text(),
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
