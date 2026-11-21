"""Error and lint index HTML renderer driven directly from the Rust diagnostics registry.

The catalog is loaded at build/check time via `cpl explain --json` (which merges
`crates/cpl-diag/src/registry.rs` and `crates/cpl-lint/src/lib.rs`), so every
compiler diagnostic and lint code is defined once in Rust and never drifts from
the error-index pages.
"""

from __future__ import annotations

import html
import json
from pathlib import Path
import re
import subprocess

REPO = Path(__file__).resolve().parent.parent
CPL = REPO / "target/debug/cpl"

CATEGORY_ORDER = [
    ("Lexical errors (E00xx)", "E00"),
    ("Syntax errors (E01xx)", "E01"),
    ("Names and modules (E02xx)", "E02"),
    ("Types and domains (E03xx)", "E03"),
    ("Ownership and mutability (E04xx)", "E04"),
    ("Backends and compatibility (E05xx)", "E05"),
    ("Lends, projections and traits (E06xx)", "E06"),
    ("Warnings (W01xx)", "W01"),
    ("Runtime traps (R00xx)", "R00"),
    ("Lints (L0xxx)", "L0"),
    ("Internal compiler error (E9999)", "E99"),
]


def load_error_catalog(cpl_bin: Path = CPL) -> list[dict]:
    """Load the complete diagnostic + lint registry from `cpl explain --json`."""
    if not cpl_bin.is_file():
        subprocess.run(
            ["cargo", "build", "-p", "cpl-cli"],
            cwd=REPO,
            check=True,
            capture_output=True,
        )
    res = subprocess.run(
        [str(cpl_bin), "explain", "--json"],
        cwd=REPO,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(res.stdout)


ERROR_CATALOG: list[dict] = load_error_catalog()
ERROR_BY_CODE: dict[str, dict] = {e["code"]: e for e in ERROR_CATALOG}


def discover_rust_codes(repo_root: Path) -> set[str]:
    """Extract every diagnostic/lint code constant from Rust sources."""
    diag_src = (repo_root / "crates/cpl-diag/src/lib.rs").read_text()
    lint_src = (repo_root / "crates/cpl-lint/src/lib.rs").read_text()
    codes = set(re.findall(r'Code\("([EWLR]\d{4})"\)', diag_src))
    codes |= set(re.findall(r'Code\("([EWLR]\d{4})"\)', lint_src))
    return codes


def error_slug(code: str) -> str:
    return f"error-{code.lower()}"


error_page_name = error_slug


def _category_for(code: str) -> str:
    for label, prefix in CATEGORY_ORDER:
        if code.startswith(prefix):
            return label
    return "Other diagnostics"


def _inline_code(text: str) -> str:
    parts = re.split(r"(`[^`]+`)", text)
    out = []
    for p in parts:
        if p.startswith("`") and p.endswith("`") and len(p) >= 2:
            out.append(f"<code>{html.escape(p[1:-1])}</code>")
        else:
            out.append(html.escape(p))
    return "".join(out)


def render_error_index_html() -> str:
    grouped: dict[str, list[dict]] = {}
    for entry in ERROR_CATALOG:
        grouped.setdefault(_category_for(entry["code"]), []).append(entry)

    out = [
        "<p>Every diagnostic, warning, runtime trap, and lint code emitted by the CP Lang compiler "
        "is generated directly from the Rust diagnostics registry (<code>cpl explain --json</code>). "
        "Click any code for its full explanation, failing example, and verified fix.</p>",
    ]
    for label, _ in CATEGORY_ORDER:
        items = grouped.get(label, [])
        if not items:
            continue
        slug = re.sub(r"[^a-z0-9]+", "-", label.lower()).strip("-")
        out.append(f'<h2 id="{slug}">{html.escape(label)}</h2>')
        out.append("<table><thead><tr><th>Code</th><th>Summary</th></tr></thead><tbody>")
        for e in items:
            code = e["code"]
            lint_tag = f" <code>@allow({e['lint_name']})</code>" if e.get("lint_name") else ""
            out.append(
                f'<tr><td><a href="{error_page_name(code)}.html"><code>{html.escape(code)}</code></a></td>'
                f"<td><strong>{html.escape(e['title'])}</strong> — {_inline_code(e['summary'])}{lint_tag}</td></tr>"
            )
        out.append("</tbody></table>")
    return "\n".join(out)


def render_error_page_html(entry: dict, highlight_cp) -> str:
    code = entry["code"]
    mode = entry["mode"]
    out = [
        '<p><a href="errors.html">← All error &amp; lint codes</a></p>',
        f"<p><strong>{html.escape(entry['summary'])}</strong></p>",
        f"<p>{_inline_code(entry['explanation'])}</p>",
    ]
    if entry.get("lint_name"):
        out.append(
            f"<p>Lint name: <code>{html.escape(entry['lint_name'])}</code> "
            f"(silence locally with <code>@allow({html.escape(entry['lint_name'])})</code>).</p>"
        )
    out.append(
        f"<p>CLI lookup: <code>cpl explain {html.escape(code)}</code></p>"
    )
    if entry["trigger"]:
        label = {
            "check": f"Fails `cpl check` with {code}",
            "lint": f"Triggers `cpl lint` ({code})",
            "run": f"Traps at runtime ({code})",
            "reserved": f"Example ({code})",
        }.get(mode, f"Triggers {code}")
        out.append('<h2 id="trigger">Example that triggers it</h2>')
        out.append(
            f'<div class="code-block"><div class="code-label">{html.escape(label)}</div>'
            f'<pre><code>{highlight_cp(entry["trigger"])}</code></pre></div>'
        )
    if entry["fix"]:
        out.append('<h2 id="fix">How to fix it</h2>')
        out.append(
            f'<div class="code-block"><div class="code-label">Fixed CP Lang</div>'
            f'<pre><code>{highlight_cp(entry["fix"])}</code></pre></div>'
        )
    if not entry["trigger"] and not entry["fix"]:
        out.append(
            "<p><em>This code is reserved or internal and has no minimal user trigger snippet.</em></p>"
        )
    return "\n".join(out)


render_error_detail_html = render_error_page_html
