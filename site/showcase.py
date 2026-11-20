"""Render the CP Lang 0.2.2 Before/After Showcase from full real C++17 and CP Lang sources."""

import html
from pathlib import Path
import re

REPO_ROOT = Path(__file__).resolve().parent.parent
REPO_URL = "https://github.com/ryanbai1412/cp-lang/blob/main"

SHOWCASE_ITEMS = [
    dict(
        id="coci21c1p3",
        title="COCI '21 #1 — Logičari",
        topic="Tree & cycle DP · Dsu, @memo, ?Int",
        cp_path="proto/ports/scout_batch_35/today/coci21c1p3.cp",
        cpp_path="corpus/submissions/4080960.cpp",
        cp_loc=48,
        raw_loc=177,
        stripped_loc=115,
        summary=(
            "A functional graph has one cycle per component. Drop one cycle edge with Dsu, "
            "build the remaining tree's child lists, and let @memo solve the four endpoint cases "
            "over ?Int without sentinel tables."
        ),
    ),
    dict(
        id="noip10p4",
        title="NOIP '10 — Water Diversion",
        topic="Grid flood fill & interval cover · typed rows and columns",
        cp_path="proto/ports/scout_batch_37/today/noip10p4.cp",
        cpp_path="corpus/submissions/5205786.cpp",
        cp_loc=39,
        raw_loc=132,
        stripped_loc=104,
        summary=(
            "Separate row and column domains keep grid coordinates distinct. A cached flood fill "
            "computes the reachable bottom-row intervals, then a greedy sweep covers them "
            "with the fewest pump stations."
        ),
    ),
    dict(
        id="noi10p1",
        title="NOI '10 — Energy Collection",
        topic="Harmonic GCD inclusion-exclusion · [1..=k]Int, .rev",
        cp_path="proto/ports/scout_batch_37/today/noi10p1.cp",
        cpp_path="corpus/submissions/5670455.cpp",
        cp_loc=13,
        raw_loc=66,
        stripped_loc=42,
        summary=(
            "A 1-based array [1..=k]Int and a reversed inclusive range express GCD "
            "inclusion-exclusion directly. Each step subtracts counts already assigned to larger multiples."
        ),
    ),
    dict(
        id="coci21c6p2",
        title="COCI '21 #6 — Zemljište",
        topic="2D sub-rectangle sweep · [0..n, 0..m], nested fn, parallel assignment",
        cp_path="proto/ports/scout_batch_35/coci21c6p2.cp",
        cpp_path="corpus/submissions/4529998.cpp",
        cp_loc=19,
        raw_loc=71,
        stripped_loc=41,
        summary=(
            "2D array typing, a local mutating helper, and parallel tuple assignment keep the "
            "two-pointer column sweep tight with no 1e18 sentinel."
        ),
    ),
    dict(
        id="cheerio1s5",
        title="Cheerio '1 S5 — Block Clearing",
        topic="String prefix & interval DP · character domains, range-indexed tables",
        cp_path="proto/ports/scout_batch_40/today/cheerio1s5.cp",
        cpp_path="corpus/submissions/5339756.cpp",
        cp_loc=20,
        raw_loc=74,
        stripped_loc=46,
        summary=(
            "A nominal character domain identifies the input positions. Prefix counts and "
            "two range-indexed DP tables track the best score while respecting the block budget."
        ),
    ),
]


def _strip_header_comments(source):
    lines = source.splitlines()
    start = 0
    while start < len(lines) and (lines[start].startswith("//") or not lines[start].strip()):
        start += 1
    return "\n".join(lines[start:]).rstrip()


def _extract_cpp_solution(source):
    """Strip only the repetitive #include/dbg template preamble, keeping 100% of the solution."""
    normalized = source.replace("\r\n", "\n").replace("\t", "    ")
    match = re.search(r"^#define sz\(x\).*?\n", normalized, re.MULTILINE)
    body = normalized[match.end() :] if match else normalized
    return body.strip()


def _default_block(code, language, header_html, highlight, *, collapsible_lines=None, copy=True):
    cleaned = code.rstrip()
    line_count = cleaned.count("\n") + (1 if cleaned else 0)
    collapsible = collapsible_lines is not None and line_count > collapsible_lines
    classes = ["code-block"]
    if collapsible:
        classes.extend(["is-collapsible", "is-collapsed"])
    copy_btn = (
        '<button class="copy-button" type="button" aria-label="Copy code">Copy</button>'
        if copy
        else ""
    )
    fade_and_btn = (
        f'<div class="code-fade" aria-hidden="true"></div>'
        f'<button class="code-expand-btn" type="button" aria-expanded="false" '
        f'data-lines="{line_count}">Show all {line_count} lines</button>'
        if collapsible
        else ""
    )
    return (
        f'<figure class="{" ".join(classes)}" data-lang="{html.escape(language)}" data-lines="{line_count}">'
        f"<figcaption>{header_html}{copy_btn}</figcaption>"
        f'<div class="code-body"><pre tabindex="0"><code>{highlight(cleaned, language)}</code></pre>'
        f"{fade_and_btn}</div></figure>"
    )


def showcase_html(highlight, render_block=None):
    total_v2 = sum(item["cp_loc"] for item in SHOWCASE_ITEMS)
    total_raw = sum(item["raw_loc"] for item in SHOWCASE_ITEMS)
    total_stripped = sum(item["stripped_loc"] for item in SHOWCASE_ITEMS)
    raw_pct = round((1 - total_v2 / total_raw) * 100)
    stripped_pct = round((1 - total_v2 / total_stripped) * 100)

    cards = []
    for item in SHOWCASE_ITEMS:
        cp_source = _strip_header_comments((REPO_ROOT / item["cp_path"]).read_text())
        cpp_source = _extract_cpp_solution((REPO_ROOT / item["cpp_path"]).read_text())
        raw_cut = round((1 - item["cp_loc"] / item["raw_loc"]) * 100)
        strip_cut = round((1 - item["cp_loc"] / item["stripped_loc"]) * 100)

        cpp_header = (
            f'<span>Before · C++17 ({item["raw_loc"]} raw / {item["stripped_loc"]} stripped) · '
            f'<a href="{REPO_URL}/{item["cpp_path"]}" style="white-space:nowrap">full file ↗</a></span>'
        )
        cp_header = (
            f'<span>After · CP Lang 0.2.2 ({item["cp_loc"]} lines) · '
            f'<a href="{REPO_URL}/{item["cp_path"]}" style="white-space:nowrap">source ↗</a></span>'
        )

        if render_block is not None:
            cpp_block = render_block(
                cpp_source,
                "cpp",
                "",
                copy=True,
                collapsible_lines=20,
                header_html=cpp_header,
            )
            cp_block = render_block(
                cp_source,
                "cp",
                "",
                copy=True,
                collapsible_lines=34,
                header_html=cp_header,
            )
        else:
            cpp_block = _default_block(
                cpp_source, "cpp", cpp_header, highlight, collapsible_lines=20, copy=True
            )
            cp_block = _default_block(
                cp_source, "cp", cp_header, highlight, collapsible_lines=34, copy=True
            )

        cards.append(
            f'<section class="showcase-card" id="showcase-{item["id"]}">'
            f'<div class="showcase-head">'
            f'<div><h3>{html.escape(item["title"])}</h3>'
            f'<p class="showcase-topic">{html.escape(item["topic"])}</p></div>'
            f'<div class="showcase-metrics">'
            f'<span class="metric-pill"><strong>{item["cp_loc"]}</strong> vs {item["raw_loc"]} raw (−{raw_cut}%)</span>'
            f'<span class="metric-pill"><strong>{item["cp_loc"]}</strong> vs {item["stripped_loc"]} stripped (−{strip_cut}%)</span>'
            f'</div></div>'
            f'<p class="showcase-summary">{html.escape(item["summary"])}</p>'
            f'<div class="showcase-grid">{cpp_block}{cp_block}</div></section>'
        )

    banner = (
        f'<div class="showcase-totals">'
        f'<span><strong>5 real contest problems:</strong> {total_v2} lines of CP Lang 0.2.2 vs '
        f'{total_raw} lines of full C++17 (−{raw_pct}%), or {total_stripped} lines with C++ template '
        f'and debug boilerplate stripped (−{stripped_pct}%).</span>'
        f'<span>Each C++17 panel below shows the real accepted solution body (only the <code>#include</code> / '
        f'<code>dbg</code> header is omitted). Click <strong>Show all lines</strong> to inspect every line.</span>'
        f'</div>'
    )
    return f'<div class="showcase">{banner}{"".join(cards)}</div>\n'
