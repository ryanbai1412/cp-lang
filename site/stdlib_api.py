#!/usr/bin/env python3
"""Generate the Stdlib API reference page from `cpl doc --format json stdlib`."""

from __future__ import annotations

import html
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CPL = ROOT / "target" / "debug" / "cpl"


def load_stdlib_docs(cpl_bin: Path = CPL) -> dict:
    if not cpl_bin.is_file():
        subprocess.run(
            ["cargo", "build", "-p", "cpl-cli"],
            cwd=ROOT,
            check=True,
            capture_output=True,
        )
    res = subprocess.run(
        [str(cpl_bin), "doc", "--format", "json", "stdlib"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(res.stdout)


def _source_line(root: Path, rel_path: str, line_no: int) -> str:
    p = root / rel_path
    if not p.is_file() or line_no < 1:
        return ""
    lines = p.read_text(encoding="utf-8").splitlines()
    return lines[line_no - 1].lstrip() if line_no <= len(lines) else ""


def iter_public_items(doc_data: dict, root: Path = ROOT):
    """Yield (module, item, pub_methods) for every public top-level item in stdlib."""
    for mod in sorted(doc_data.get("modules", []), key=lambda m: m.get("name", "")):
        rel = mod.get("path", "")
        if not rel or not (root / rel).is_file():
            continue
        lines = (root / rel).read_text(encoding="utf-8").splitlines()

        def is_pub(line_no: int) -> bool:
            if 1 <= line_no <= len(lines):
                return lines[line_no - 1].lstrip().startswith("pub ")
            return False

        pub_items = []
        for it in mod.get("items", []):
            if not is_pub(it.get("line", 0)):
                continue
            pub_methods = [
                mem
                for mem in it.get("members", [])
                if mem.get("kind") == "fn" and is_pub(mem.get("line", 0))
            ]
            pub_items.append((it, pub_methods))
        yield mod, pub_items


def undocumented_public_functions(doc_data: dict, root: Path = ROOT) -> list[tuple[str, str]]:
    """Return [(module_path, fn_name), ...] for every pub fn lacking a /// doc comment."""
    missing: list[tuple[str, str]] = []
    for mod, pub_items in iter_public_items(doc_data, root):
        rel = mod.get("path", mod.get("name", ""))
        for it, pub_methods in pub_items:
            if it.get("kind") == "fn" and not (it.get("doc") or "").strip():
                missing.append((rel, it.get("name", "?")))
            for mem in pub_methods:
                if not (mem.get("doc") or "").strip():
                    missing.append((rel, f"{it.get('name', '?')}.{mem.get('name', '?')}"))
    return missing


def render_stdlib_api_html(cpl_bin: Path = CPL, root: Path = ROOT) -> tuple[str, list[tuple[str, str]]]:
    data = load_stdlib_docs(cpl_bin)
    modules_data = list(iter_public_items(data, root))
    total_types = sum(
        1 for _, items in modules_data for it, _ in items if it.get("kind") != "fn"
    )
    total_fns = sum(
        (1 if it.get("kind") == "fn" else 0) + len(methods)
        for _, items in modules_data
        for it, methods in items
    )
    missing_fns = undocumented_public_functions(data, root)

    headings: list[tuple[str, str]] = []
    parts = [
        '<p class="lead">Generated directly from <code>///</code> doc comments in '
        '<code>stdlib/*.cp</code> via <code>cpl doc --format json stdlib</code>. '
        'Every public function documents its behavior, laws, and time/space cost, and the fast '
        'gate fails if any <code>pub fn</code> lacks documentation.</p>',
        '<div class="gallery-stats">'
        f'<span><strong>{len(modules_data)}</strong> modules</span>'
        f'<span><strong>{total_types}</strong> public types &amp; constants</span>'
        f'<span><strong>{total_fns - len(missing_fns)}/{total_fns}</strong> documented public functions</span>'
        '</div>',
        '<p class="stdlib-toc">'
        + " · ".join(
            f'<a href="#mod-{html.escape(mod["name"])}"><code>std.{html.escape(mod["name"])}</code></a>'
            for mod, _ in modules_data
        )
        + "</p>",
    ]

    for mod, pub_items in modules_data:
        mname = mod["name"]
        anchor = f"mod-{mname}"
        headings.append((anchor, f"std.{mname}"))
        parts.append(f'<h2 id="{html.escape(anchor)}"><code>std.{html.escape(mname)}</code></h2>')
        if (mod.get("doc") or "").strip():
            parts.append(f'<p>{html.escape(mod["doc"].strip())}</p>')
        if not pub_items:
            parts.append("<p><em>No public declarations (demo/tour module).</em></p>")
            continue
        parts.append('<dl class="api-list">')
        for it, pub_methods in pub_items:
            sig = html.escape(it.get("signature", it.get("name", "")))
            doc = (it.get("doc") or "").strip()
            kind = html.escape(it.get("kind", "item"))
            parts.append(
                f'<dt><span class="api-kind">{kind}</span> <code>{sig}</code></dt>'
                f'<dd>{html.escape(doc) if doc else "<em>(undocumented)</em>"}</dd>'
            )
            for mem in pub_methods:
                msig = html.escape(f"{it['name']}.{mem.get('signature', mem.get('name', ''))}")
                mdoc = (mem.get("doc") or "").strip()
                parts.append(
                    f'<dt><span class="api-kind">method</span> <code>{msig}</code></dt>'
                    f'<dd>{html.escape(mdoc) if mdoc else "<em>(undocumented)</em>"}</dd>'
                )
        parts.append("</dl>")

    return "\n".join(parts), headings
