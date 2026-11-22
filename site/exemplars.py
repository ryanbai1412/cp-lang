#!/usr/bin/env python3
"""Generate the 'Reading exemplars' page: Dream vs Validated Port + IOI 2025 sketches."""

from __future__ import annotations

import base64
import html
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PORTS_ROOT = ROOT / "proto" / "ports"
IOI_DIR = ROOT / "design" / "sketches" / "ioi2025"

TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|[0-9]+|[^\s]")


def _count_tokens(src: str) -> int:
    lines = [ln for ln in src.splitlines() if not ln.lstrip().startswith("//")]
    return len(TOKEN_RE.findall("\n".join(lines)))


def _share_href(source: str) -> str:
    payload = json.dumps(
        {"source": source, "stdin": "", "samples": []},
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")
    encoded = base64.urlsafe_b64encode(payload).decode("ascii").rstrip("=")
    return f"playground.html#share={encoded}"


def load_dream_pairs(root: Path = ROOT) -> list[dict]:
    """Return all (dream, validated port) pairs discovered under corpus/problems/."""
    ports_root = root / "corpus" / "problems"
    pairs: list[dict] = []
    if not ports_root.is_dir():
        return pairs
    for dream_path in sorted(ports_root.glob("*/dream.cp")):
        prob_dir = dream_path.parent
        port_path = prob_dir / "port.cp"
        if not port_path.is_file():
            continue
        dream_src = dream_path.read_text(encoding="utf-8").rstrip()
        port_src = port_path.read_text(encoding="utf-8").rstrip()
        dream_tok = _count_tokens(dream_src)
        port_tok = _count_tokens(port_src)
        ratio = round(port_tok / dream_tok, 2) if dream_tok else 1.0
        header_lines = []
        for ln in dream_src.splitlines():
            s = ln.strip()
            if s.startswith("//"):
                header_lines.append(s.lstrip("/").strip())
            elif header_lines:
                break
        pairs.append(
            {
                "id": prob_dir.name,
                "group": "problems",
                "dream_path": str(dream_path.relative_to(root)),
                "port_path": str(port_path.relative_to(root)),
                "dream_src": dream_src,
                "port_src": port_src,
                "dream_tokens": dream_tok,
                "port_tokens": port_tok,
                "ratio": ratio,
                "summary": " ".join(header_lines[:2]),
            }
        )
    pairs.sort(key=lambda p: (p["ratio"], p["id"]))
    return pairs


def load_ioi_sketches(root: Path = ROOT) -> list[dict]:
    ioi_dir = root / "design" / "sketches" / "ioi2025"
    out: list[dict] = []
    if not ioi_dir.is_dir():
        return out
    for p in sorted(ioi_dir.glob("*.cp")):
        src = p.read_text(encoding="utf-8").rstrip()
        out.append(
            {
                "id": p.stem,
                "path": str(p.relative_to(root)),
                "src": src,
                "tokens": _count_tokens(src),
            }
        )
    return out


def render_exemplars_html(highlight_fn, root: Path = ROOT) -> tuple[str, list[tuple[str, str]]]:
    """Render the Reading Exemplars page body and TOC headings."""
    pairs = load_dream_pairs(root)
    ioi = load_ioi_sketches(root)
    headings = [
        ("dream-vs-port", f"Dream vs validated port ({len(pairs)} pairs)"),
        ("ioi-2025", f"IOI 2025 gold-standard sketches ({len(ioi)} problems)"),
    ]

    median_ratio = sorted(p["ratio"] for p in pairs)[len(pairs) // 2] if pairs else 1.0
    featured = pairs[:12]
    featured_ids = {p["id"] for p in featured}
    parts = [
        '<p class="lead">Generated from the dream corpus, each dream paired with its validated port, '
        'and from the IOI 2025 sketches. '
        'Each dream shows how the solution should read at the bar of '
        '<code>design/EXEMPLARS-PORTS.md</code>; the port beside it is what runs and passes '
        'today (sorted closest-to-dream first).</p>',
        '<div class="gallery-stats">'
        f'<span><strong>{len(pairs)}</strong> dream/port pairs</span>'
        f'<span><strong>{median_ratio:.2f}x</strong> median port/dream token ratio</span>'
        f'<span><strong>{len(ioi)}</strong> IOI 2025 sketches</span>'
        '</div>',
        f'<h2 id="dream-vs-port">Dream vs validated port ({len(pairs)} pairs)</h2>',
        '<p>Showing the 12 closest-to-dream pairs side by side (every port has a <strong>Try it</strong> link '
        'into the browser playground), followed by the full index of all pairs.</p>',
    ]

    for p in featured:
        try_href = html.escape(_share_href(p["port_src"]))
        summary_html = f'<p class="exemplar-note">{html.escape(p["summary"])}</p>' if p["summary"] else ""
        parts.append(
            f'<article class="exemplar-card" id="ex-{html.escape(p["id"])}">'
            f'<header class="exemplar-head">'
            f'<h3><code>{html.escape(p["id"])}</code> <span class="badge">{p["ratio"]:.2f}x</span></h3>'
            f'<span class="exemplar-meta">dream {p["dream_tokens"]} tok · port {p["port_tokens"]} tok · '
            f'<a class="try-button" href="{try_href}">Try port in playground</a></span>'
            f'</header>'
            f'{summary_html}'
            f'<div class="exemplar-grid">'
            f'<div class="exemplar-pane"><div class="pane-label">Dream</div>'
            f'<pre><code>{highlight_fn(p["dream_src"], "cp")}</code></pre></div>'
            f'<div class="exemplar-pane"><div class="pane-label">Validated port</div>'
            f'<pre><code>{highlight_fn(p["port_src"], "cp")}</code></pre></div>'
            f'</div></article>'
        )

    parts.append(
        '<h3>Full dream corpus index</h3>'
        '<table class="data-table"><thead><tr>'
        '<th>Problem</th><th>Group</th><th>Dream tok</th><th>Port tok</th><th>Ratio</th><th>Playground</th>'
        '</tr></thead><tbody>'
    )
    for p in pairs:
        if p["id"] in featured_ids:
            label = f'<a href="#ex-{html.escape(p["id"])}"><code>{html.escape(p["id"])}</code></a>'
            action = f'<a href="{html.escape(_share_href(p["port_src"]))}">Try it</a>'
        else:
            label = f'<code>{html.escape(p["id"])}</code>'
            action = '&mdash;'
        parts.append(
            f'<tr><td>{label}</td>'
            f'<td><code>{html.escape(p["group"])}</code></td>'
            f'<td>{p["dream_tokens"]}</td>'
            f'<td>{p["port_tokens"]}</td>'
            f'<td>{p["ratio"]:.2f}x</td>'
            f'<td>{action}</td></tr>'
        )
    parts.append('</tbody></table>')

    parts.append(f'<h2 id="ioi-2025">IOI 2025 gold-standard sketches ({len(ioi)} problems)</h2>')
    parts.append(
        '<p>Design-target sketches from <code>design/sketches/ioi2025/</code> showing how '
        'editorial-first CP Lang reads on Olympiad problems.</p>'
    )
    for sk in ioi:
        parts.append(
            f'<details class="exemplar-sketch" id="ioi-{html.escape(sk["id"])}">'
            f'<summary><strong>{html.escape(sk["id"])}</strong> '
            f'(<code>{html.escape(sk["path"])}</code>, {sk["tokens"]} tokens)</summary>'
            f'<pre><code>{highlight_fn(sk["src"], "cp")}</code></pre>'
            f'</details>'
        )

    return "\n".join(parts), headings
