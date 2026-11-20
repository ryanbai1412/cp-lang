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
    for path in sorted(target.glob("*.html")):
        page = path.read_text()
        # Shared assets and the full-text index are stored once, not per chapter.
        page = re.sub(r'<script id="search-index" type="application/json">.*?</script>', "", page, flags=re.S)
        page = re.sub(r'<link rel="icon"[^>]*>', "", page)
        pages[path.name] = page
    payload = dict(
        pages=pages,
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
