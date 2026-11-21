#!/usr/bin/env python3
"""Build the CP Lang guide. No network, JavaScript bundler, or server required."""

import html
import hashlib
import json
from pathlib import Path
import re
import shutil

import mistune
from course import catalog_html, exercise_html, load_lessons, progress_html
from compiler import build_compiler, editor_html
from cookbook import all_recipes, render_cookbook_html
from errors import ERROR_CATALOG, error_slug, render_error_index_html, render_error_page_html
from ports_gallery import render_ports_gallery_html
from showcase import showcase_html

ROOT = Path(__file__).resolve().parent
REPO = "https://github.com/ryanbai1412/cp-lang"
REFERENCE_PAGES = [
    (
        "index",
        "Overview",
        "Start here",
        "A competitive programming language that compiles to fast, clean C++17.",
        "5 min",
    ),
    (
        "getting-started",
        "Getting started & CLI",
        "Start here",
        "Install the cpl toolchain, run programs locally, and compile submittable C++17.",
        "5 min",
    ),
    (
        "tour",
        "Language tour",
        "Start here",
        "A guided walkthrough of syntax, program layout, and everyday contest idioms.",
        "8 min",
    ),
    (
        "cheat-sheet",
        "One-page cheat sheet",
        "Quick reference",
        "Every CP Lang construct, collection, and stdlib helper on one printable page.",
        "1 page",
    ),
    (
        "errors",
        "Error & lint index",
        "Quick reference",
        "Every diagnostic, warning, runtime trap, and lint code with verified trigger and fix.",
        f"{len(ERROR_CATALOG)} codes",
    ),
    (
        "types-and-control",
        "Types, numbers & control flow",
        "Language reference",
        "Numeric rules, ranges, Tropical Optionals, structs, enums, traits, and loops.",
        "12 min",
    ),
    (
        "io-and-formatting",
        "I/O, formatting & contest modes",
        "Language reference",
        "Typed read(), print() formatting, @!multitest, interactive ask, and IOI graders.",
        "9 min",
    ),
    (
        "indices",
        "Index spaces, domains & slices",
        "Language reference",
        "Typed positions, .l/.r boundaries, grids, slices, arenas, and coordinate compression.",
        "11 min",
    ),
    (
        "borrowing",
        "Ownership, lends & borrowing",
        "Language reference",
        "Value semantics, mut parameters, lends projections, and @law disjoint without lifetimes.",
        "9 min",
    ),
    (
        "strings",
        "Strings & ASCII text",
        "Language reference",
        "0..len byte strings, slicing, parsing, nominal character domains, and string algorithms.",
        "6 min",
    ),
    (
        "testing",
        "Memoization, stress testing & Trace",
        "In practice",
        "@memo caching, @brute/@checker/@interactor verification, and the visual Trace debugger.",
        "10 min",
    ),
    (
        "stdlib",
        "Standard & local libraries",
        "In practice",
        "Complete API reference for all std modules plus local library packaging and recipes.",
        "14 min",
    ),
    (
        "cookbook",
        "Cookbook from real ports",
        "In practice",
        "Real validated contest solutions organized by algorithm and data structure.",
        f"{len(all_recipes())} recipes",
    ),
    (
        "ports",
        "Ports gallery (CP / dream / C++)",
        "In practice",
        "Side-by-side comparison of validated CP Lang ports, dream programs, and accepted C++.",
        "gallery",
    ),
    (
        "cpp-guide",
        "From C++17 & performance",
        "In practice",
        "C++17-to-CP-Lang translation guide, emitted memory layouts, and zero-overhead escape hatches.",
        "10 min",
    ),
]

LESSONS = load_lessons()
EXERCISES = {exercise["id"]: exercise for lesson in LESSONS for exercise in lesson["exercises"]}
PAGES = [
    (
        "course",
        "Interactive course",
        "Course",
        f"{len(LESSONS)} progressive chapters from your first program to contest fluency, with {len(EXERCISES)} checked exercises.",
        f"{len(LESSONS)} chapters",
    ),
    *[(lesson["slug"], lesson["title"], "Course", lesson["description"], lesson["duration"])
      for lesson in LESSONS],
    *REFERENCE_PAGES,
]


def slug(text):
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")


TOKEN_CP = re.compile(
    r'(?P<comment>//[^\n]*)'
    r'|(?P<string>"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\')'
    r'|(?P<decorator>@!?\w+|\'[a-zA-Z_]\w*)'
    r'|(?P<number>\b(?:0x[0-9a-fA-F_]+|0b[01_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?)\b)'
    r'|(?P<keyword>\b(?:let|var|const|fn|struct|enum|domain|type|use|pub|if|elif|else|for|while|repeat|loop|in|by|where|match|return|break|continue|and|or|not|assert|mut|self|unchecked|true|false|none|trait|impl|lends|yield)\b)'
    r'|(?P<type>\b[A-Z][A-Za-z0-9_]*\b)'
    r'|(?P<function>\b[a-z_][a-zA-Z0-9_]*(?=\s*(?:::<|\()))'
)

TOKEN_CPP = re.compile(
    r'(?P<comment>/\*[\s\S]*?\*/|//[^\n]*)'
    r'|(?P<preproc>^[ \t]*#[ \t]*[a-zA-Z_]\w*)'
    r'|(?P<string>"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\')'
    r'|(?P<number>\b(?:0x[0-9a-fA-F\']+|\d[\d\']*(?:\.\d[\d\']*)?(?:[eE][+-]?\d+)?(?:ULL|LL|UL|U|L)?)\b)'
    r'|(?P<keyword>\b(?:alignas|alignof|auto|bool|break|case|catch|char|class|const|constexpr|continue|decltype|default|delete|do|double|else|enum|explicit|extern|false|float|for|friend|goto|if|inline|int|long|mutable|namespace|new|noexcept|nullptr|operator|private|protected|public|return|short|signed|sizeof|static|static_assert|struct|switch|template|this|throw|true|try|typedef|typename|union|unsigned|using|virtual|void|volatile|while)\b)'
    r'|(?P<type>\b(?:vector|array|string|string_view|pair|tuple|map|multimap|set|multiset|unordered_map|unordered_set|queue|deque|stack|priority_queue|bitset|optional|variant|function|span|int8_t|int16_t|int32_t|int64_t|uint8_t|uint16_t|uint32_t|uint64_t|size_t|ptrdiff_t|istream|ostream|__int128|ll|ld|vi|vll|pii)\b)'
    r'|(?P<function>\b[a-zA-Z_][a-zA-Z0-9_]*(?=\s*\())',
    re.MULTILINE,
)

TOKEN_SH = re.compile(
    r'(?P<comment>#[^\n]*)'
    r'|(?P<string>"(?:\\.|[^"\\])*"|\'[^\'\n]*\')'
    r'|(?P<decorator>(?<!\w)--?[a-zA-Z0-9][a-zA-Z0-9_-]*)'
    r'|(?P<keyword>\b(?:cpl|g\+\+|clang\+\+|cargo|git|python3|printf|cd|mkdir|node|export)\b)'
    r'|(?P<number>\b\d+\b)'
)

TOKEN_TOML = re.compile(
    r'(?P<comment>#[^\n]*)'
    r'|(?P<decorator>^\[[^\]]+\])'
    r'|(?P<string>"(?:\\.|[^"\\])*"|\'[^\'\n]*\')'
    r'|(?P<keyword>\b(?:true|false)\b)'
    r'|(?P<number>\b\d[\d_]*\b)',
    re.MULTILINE,
)

TOKENIZERS = {
    "cp": TOKEN_CP,
    "cpp": TOKEN_CPP,
    "c++": TOKEN_CPP,
    "sh": TOKEN_SH,
    "bash": TOKEN_SH,
    "shell": TOKEN_SH,
    "toml": TOKEN_TOML,
}


def highlight(code, language):
    lang = (language or "text").lower()
    pattern = TOKENIZERS.get(lang)
    if pattern is None:
        return html.escape(code)
    parts, end = [], 0
    for token in pattern.finditer(code):
        parts += [
            html.escape(code[end : token.start()]),
            f'<span class="tok-{token.lastgroup}">{html.escape(token.group())}</span>',
        ]
        end = token.end()
    return "".join(parts) + html.escape(code[end:])


def code_block_html(
    code,
    language,
    label,
    *,
    copy=True,
    collapsible_lines=28,
    header_html=None,
    extra_class="",
):
    """Render a syntax-highlighted code figure with optional Show more/less expansion."""
    cleaned = code.rstrip()
    line_count = cleaned.count("\n") + (1 if cleaned else 0)
    collapsible = collapsible_lines is not None and line_count > collapsible_lines
    classes = ["code-block"]
    if extra_class:
        classes.append(extra_class)
    if collapsible:
        classes.extend(["is-collapsible", "is-collapsed"])
    lang_attr = html.escape((language or "text").lower())
    caption_inner = header_html if header_html is not None else f"<span>{html.escape(label)}</span>"
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
        f'<figure class="{" ".join(classes)}" data-lang="{lang_attr}" data-lines="{line_count}">'
        f"<figcaption>{caption_inner}{copy_btn}</figcaption>"
        f'<div class="code-body"><pre tabindex="0"><code>{highlight(cleaned, language)}</code></pre>'
        f"{fade_and_btn}</div></figure>\n"
    )


class Renderer(mistune.HTMLRenderer):
    def __init__(self):
        super().__init__(escape=True)
        self.headings = []
        self.ids = set()

    def heading(self, text, level, **attrs):
        base = slug(re.sub(r"<[^>]*>", "", html.unescape(text)))
        anchor, n = base, 1
        while anchor in self.ids:
            n += 1
            anchor = f"{base}-{n}"
        self.ids.add(anchor)
        if level == 2:
            self.headings.append((anchor, text))
        return f'<h{level} id="{anchor}">{text}<a class="heading-link" href="#{anchor}" aria-label="Link to this section">#</a></h{level}>\n'

    def block_code(self, code, info=None):
        fields = (info or "text").split()
        language = fields[0]
        if language == "course":
            return catalog_html(LESSONS)
        if language == "showcase":
            return showcase_html(highlight, code_block_html)
        if language == "exercise":
            if len(fields) != 2 or fields[1] not in EXERCISES:
                raise ValueError(f"Unknown exercise directive: {info}")
            return exercise_html(EXERCISES[fields[1]], highlight)
        label = " ".join(fields[1:]) or {
            "cp": "CP Lang 0.2.2",
            "cpp": "C++17",
            "c++": "C++17",
            "sh": "Terminal",
            "bash": "Terminal",
            "toml": "TOML",
            "text": "Output",
        }.get(language, language)
        if language == "example":
            name = fields[1]
            code = (ROOT / "examples" / f"{name}.cp").read_text()
            extra = " ".join(fields[2:]) or "WASM runnable"
            language, label = "cp", f"{name}.cp · {extra}"
        return code_block_html(code, language, label)


def navigation(current):
    out, group = [], None
    for i, (name, title, section, _, _) in enumerate(PAGES):
        if group != section:
            if group:
                out.append("</div>")
            out.append(f'<div class="nav-group"><h2>{section}</h2>')
            group = section
        active = ' aria-current="page"' if name == current else ""
        out.append(
            f'<a href="{name}.html"{active}><span class="nav-number">{i:02}</span>{html.escape(title)}</a>'
        )
    return "".join(out) + "</div>"


TRACE_SAMPLES = ["knapsack", "dijkstra", "tree", "grid", "array"]


def trace_html():
    """Use the exact viewer embedded by cpl and VS Code; only add site navigation/data."""
    samples = [json.loads((ROOT.parent / f"tools/trace/samples/{name}.json").read_text()) for name in TRACE_SAMPLES]
    payload = json.dumps(samples, ensure_ascii=False, separators=(",", ":"), allow_nan=False)
    payload = payload.replace("<", "\\u003c").replace("\u2028", "\\u2028").replace("\u2029", "\\u2029")
    page = (ROOT.parent / "tools/trace/index.html").read_text()
    parts = {
        '<html lang="en" data-context="standalone">': '<html lang="en" data-context="site">',
        '<nav hidden data-site-nav': '<nav data-site-nav',
        '<script id="trace-samples" type="application/json">[]</script>':
            f'<script id="trace-samples" type="application/json">{payload}</script>',
    }
    for marker, content in parts.items():
        assert page.count(marker) == 1, marker
        page = page.replace(marker, content)
    return page


def interpolate_markdown(source):
    """Replace dynamic site-wide counters so copy never drifts from manifests."""
    replacements = {
        "{{chapter_count}}": str(len(LESSONS)),
        "{{exercise_count}}": str(len(EXERCISES)),
        "{{reference_count}}": str(len(REFERENCE_PAGES)),
    }
    for marker, value in replacements.items():
        source = source.replace(marker, value)
    return source


def load_page_source(name):
    """Load markdown source for a page, pulling the cheat sheet from its single canonical file."""
    if name == "cheat-sheet":
        raw = (ROOT.parent / "docs" / "one-page-cheat-sheet.md").read_text()
        body = re.sub(r"\A#[^\n]*\n+(?:\*[^\n]*\*\n+)?(?:---\n+)?", "", raw)
        intro = (
            "Every CP Lang 0.2.2 construct, collection, and standard library helper on one printable page. "
            "Press `Ctrl / ⌘ + P` to print or save as PDF, take the [interactive course](course.html) to learn "
            "each feature in depth, or try any snippet live in the [playground](playground.html).\n\n---\n\n"
        )
        return intro + body, "docs/one-page-cheat-sheet.md"
    if name == "errors":
        return "", "site/errors.py"
    if name == "cookbook":
        return "", "site/cookbook.py"
    if name == "ports":
        return "", "site/ports_gallery.py"
    return (ROOT / "content" / f"{name}.md").read_text(), f"site/content/{name}.md"


def build():
    target = ROOT / "_build"
    target.mkdir(exist_ok=True)
    shutil.copytree(ROOT / "assets", target / "assets", dirs_exist_ok=True)
    build_compiler(target)
    template = (ROOT / "template.html").read_text()
    search = []
    for i, (name, title, section, description, duration) in enumerate(PAGES):
        raw_source, source_path = load_page_source(name)
        if name == "errors":
            body = render_error_index_html()
            toc = "".join(
                f'<a href="#{m.group(1)}">{html.unescape(m.group(2))}</a>'
                for m in re.finditer(r'<h2 id="([^"]+)">([^<]+)</h2>', body)
            )
        elif name == "cookbook":
            body = render_cookbook_html(ROOT.parent, lambda src: highlight(src, "cp"))
            toc = "".join(
                f'<a href="#{m.group(1)}">{html.unescape(m.group(2))}</a>'
                for m in re.finditer(r'<h2 id="([^"]+)">([^<]+)</h2>', body)
            )
        elif name == "ports":
            body = render_ports_gallery_html(
                ROOT.parent,
                lambda src: highlight(src, "cp"),
                lambda src: highlight(src, "cpp"),
            )
            toc = "".join(
                f'<a href="#{m.group(1)}">{html.unescape(m.group(2))}</a>'
                for m in re.finditer(r'<h2 id="([^"]+)">([^<]+)</h2>', body)
            )
        else:
            source = interpolate_markdown(raw_source)
            renderer = Renderer()
            body = mistune.create_markdown(renderer=renderer)(source)
            toc = "".join(
                f'<a href="#{anchor}">{text}</a>' for anchor, text in renderer.headings
            )
        is_course = section == "Course"
        if is_course:
            body = progress_html(LESSONS, name if name != "course" else None) + body
        pager = []
        sequence = ([("course", "Interactive course")]
                    + [(lesson["slug"], lesson["title"]) for lesson in LESSONS]
                    + [("playground", "Try the playground")]) if is_course else [page[:2] for page in REFERENCE_PAGES]
        position = next(j for j, page in enumerate(sequence) if page[0] == name)
        for index, direction in [(position - 1, "Previous"), (position + 1, "Next")]:
            if 0 <= index < len(sequence):
                other, label = sequence[index]
                pager.append(
                    f'<a class="pager-{direction.lower()}" href="{other}.html"><span>{direction} {"←" if direction == "Previous" else "→"}</span>{label}</a>'
                )
        heading = "CP Lang" if name == "index" else html.escape(title)
        values = dict(
            title=html.escape(title),
            heading=heading,
            description=html.escape(description),
            section=section,
            duration=duration,
            number=f"{i:02}",
            navigation=navigation(name),
            content=body,
            toc=toc,
            pager="".join(pager),
            name=name,
            source_path=source_path,
            repo=REPO,
            page_class="overview" if name == "index" else ("cheat-sheet" if name == "cheat-sheet" else "chapter"),
            docs_current='' if (is_course or name == "cheat-sheet") else 'aria-current="page"',
            cheat_sheet_current='aria-current="page"' if name == "cheat-sheet" else '',
            course_current='aria-current="page"' if is_course else '',
            series="THE CP LANG COURSE" if is_course else ("CP LANG 0.2.2 REFERENCE" if name == "cheat-sheet" else "CP LANG DOCUMENTATION"),
            playground_current="",
        )
        page = template
        for key, value in values.items():
            page = page.replace("{{" + key + "}}", value)
        if is_course:
            page = page.replace('</body>', '<script id="course-data" type="application/json">'
                + safe_json(course_data(name)) + '</script></body>')
        (target / f"{name}.html").write_text(page)
        searchable = re.sub(
            r'<details class="exercise-help exercise-sandbox">.*?</details>|<noscript>.*?</noscript>',
            " ",
            body,
            flags=re.S,
        )
        plain = html.unescape(re.sub(r"<[^>]*>", " ", searchable))
        search.append(
            dict(
                title=title,
                section=section,
                url=f"{name}.html",
                text=re.sub(r"\s+", " ", plain),
            )
        )
    for idx, entry in enumerate(ERROR_CATALOG):
        code = entry["code"]
        slug = error_slug(code)
        title = f'{code}: {entry["title"]}'
        body = render_error_page_html(entry, lambda src: highlight(src, "cp"))
        pager = []
        if idx > 0:
            prev = ERROR_CATALOG[idx - 1]
            pager.append(
                f'<a class="pager-previous" href="{error_slug(prev["code"])}.html">'
                f'<span>Previous &larr;</span>{html.escape(prev["code"])}: {html.escape(prev["title"])}</a>'
            )
        if idx + 1 < len(ERROR_CATALOG):
            nxt = ERROR_CATALOG[idx + 1]
            pager.append(
                f'<a class="pager-next" href="{error_slug(nxt["code"])}.html">'
                f'<span>Next &rarr;</span>{html.escape(nxt["code"])}: {html.escape(nxt["title"])}</a>'
            )
        values = dict(
            title=html.escape(title),
            heading=html.escape(title),
            description=html.escape(entry["summary"]),
            section="Error index",
            duration=code,
            number=code,
            navigation=navigation("errors"),
            content=body,
            toc="",
            pager="".join(pager),
            name=slug,
            source_path="site/errors.py",
            repo=REPO,
            page_class="chapter",
            docs_current='aria-current="page"',
            cheat_sheet_current="",
            course_current="",
            series="CP LANG ERROR INDEX",
            playground_current="",
        )
        page = template
        for key, value in values.items():
            page = page.replace("{{" + key + "}}", value)
        (target / f"{slug}.html").write_text(page)
        plain = html.unescape(re.sub(r"<[^>]*>", " ", body))
        search.append(
            dict(
                title=title,
                section="Error index",
                url=f"{slug}.html",
                text=re.sub(r"\s+", " ", plain),
            )
        )
    (target / "search.json").write_text(json.dumps(search))
    build_playground(target, template)
    # Embed the search index so file:// and the offline bundle need no fetch.
    for path in target.glob("*.html"):
        page = path.read_text().replace('</head>',
            '<script id="search-index" type="application/json">'
            + safe_json(search) + '</script>\n</head>')
        path.write_text(page)
    (target / "trace.html").write_text(trace_html())
    print(f"Built {len(PAGES)} chapters + playground + trace → {target}")


def safe_json(value):
    return (
        json.dumps(value, ensure_ascii=False, separators=(",", ":"))
        .replace("</", "<\\/")
        .replace("<!--", "\\u003c!--")
        .replace("\u2028", "\\u2028")
        .replace("\u2029", "\\u2029")
    )


def course_data(name):
    """Ship compact checks for cross-chapter progress; keep feedback local."""
    checks = {}
    for lesson in LESSONS:
        local = name == "*" or lesson["slug"] == name
        for exercise in lesson["exercises"]:
            check = {"kind": exercise["kind"]}
            if exercise["kind"] == "choice":
                check["answer"] = exercise["answer"]
                if local:
                    check["explanation"] = exercise["explanation"]
            else:
                # Stored code receipts and predictions use the same expected outputs.
                check["tests"] = (
                    exercise["tests"]
                    if local
                    else [{"expected": case["expected"]} for case in exercise["tests"]]
                )
            checks[exercise["id"]] = check
    revisions = {key: hashlib.sha256((safe_json(exercise) + (
                    (ROOT / "examples" / f'{exercise["fixture"]}.cp').read_text()
                    if exercise["kind"] == "output" else "")).encode()).hexdigest()[:16]
                 for key, exercise in EXERCISES.items()}
    return dict(lessons=[dict(slug=lesson["slug"], exercises=[exercise["id"] for exercise in lesson["exercises"]])
                         for lesson in LESSONS], exercises=checks, revisions=revisions)


STR_SCAN_SOURCE = """let s: Str = read()

var hits = 0
for i in 0..s.len:
    if s[i] == 'a':
        hits += 1
print(hits, s.len)
"""


def build_playground(target, template):
    defaults = dict(
        sum="5\n3 1 4 1 5",
        early_return="4\n3 9 -2 5",
        multitest_divider="4\n0 2\n4 1\n4 2\n4 3",
        bounds="5\n3 1 4 1 5",
        knapsack="3 7\n3 4\n4 5\n2 3",
        range_array="6 4\n0 2 4 2 1 2",
        enum_match="3\n1 10\n2 3\n1 5",
        collections_heap="5 3\n9 2 7 4 1",
        sort_bsearch="5 7\n9 2 7 4 7",
        lends="3 2\n1 5\n3 -2",
        tropical_dp="3 11\n1 5 7",
        optional_coalesce="5\n3 -4 7 -2 6",
        memo_dp="10",
        segtree_monoid="4 3\n2 5 1 4\n2 1 3\n1 2 10\n2 1 4",
        lazy_action="4 3\n1 2 3 4\n1 2 4 3\n2 1 4\n2 2 3",
        fenwick_sum="4 3\n3 1 4 2\n2 1 4\n1 2 5\n2 1 3",
        dsu_mod="5 3\n1 2\n2 3\n4 5",
        colocated_brute="5\n-2 4 -1 3 -5",
        accumulation="4\n0 1 2 3",
        ascii="5\nabaca",
        optional="5\n3 7 4 1 8",
        memo="4",
    )
    order = (
        "sum",
        "early_return",
        "multitest_divider",
        "bounds",
        "knapsack",
        "range_array",
        "enum_match",
        "collections_heap",
        "sort_bsearch",
        "lends",
        "tropical_dp",
        "optional_coalesce",
        "memo_dp",
        "segtree_monoid",
        "lazy_action",
        "fenwick_sum",
        "dsu_mod",
        "colocated_brute",
        "accumulation",
        "ascii",
        "optional",
        "memo",
    )
    sample_map = {}
    for lesson in LESSONS:
        for exercise in lesson["exercises"]:
            if exercise["kind"] == "output":
                sample_map.setdefault(exercise["fixture"], []).extend(
                    {"input": case["input"], "expected": case["expected"]}
                    for case in exercise["tests"]
                )
    fallback_samples = {
        "sum": [{"input": "5\n3 1 4 1 5", "expected": "14"}],
        "early_return": [{"input": "4\n3 9 -2 5", "expected": "9"}],
        "bounds": [{"input": "5\n3 1 4 1 5", "expected": "0 3 4 8 9 14"}],
        "knapsack": [{"input": "3 7\n3 4\n4 5\n2 3", "expected": "9"}],
        "range_array": [{"input": "6 4\n0 2 4 2 1 2", "expected": "1 1 3 0 1"}],
        "tropical_dp": [{"input": "3 11\n1 5 7", "expected": "3"}],
        "memo_dp": [{"input": "10", "expected": "89"}],
        "segtree_monoid": [{"input": "4 3\n2 5 1 4\n2 1 3\n1 2 10\n2 1 4", "expected": "8\n17"}],
        "lazy_action": [{"input": "4 3\n1 2 3 4\n1 2 4 3\n2 1 4\n2 2 3", "expected": "19\n11"}],
        "fenwick_sum": [{"input": "4 3\n3 1 4 2\n2 1 4\n1 2 5\n2 1 3", "expected": "10\n13"}],
        "dsu_mod": [{"input": "5 3\n1 2\n2 3\n4 5", "expected": "2 6"}],
        "colocated_brute": [{"input": "5\n-2 4 -1 3 -5", "expected": "6"}],
        "accumulation": [{"input": "4\n0 1 2 3", "expected": "6"}],
        "ascii": [{"input": "5\nabaca", "expected": "3"}],
        "optional": [{"input": "5\n3 7 4 1 8", "expected": "8"}],
        "memo": [{"input": "4", "expected": "5"}],
    }
    examples = [
        dict(
            id=name,
            source=(ROOT / "examples" / f"{name}.cp").read_text(),
            stdin=defaults[name],
            samples=sample_map.get(name) or fallback_samples.get(name, []),
        )
        for name in order
    ]
    examples.insert(
        6,
        dict(
            id="str-scan",
            source=STR_SCAN_SOURCE,
            stdin="abracadabra",
            samples=[
                {"input": "abracadabra", "expected": "5 11"},
                {"input": "xyz", "expected": "0 3"},
            ],
        ),
    )
    values = dict(title="Playground", heading="Playground", section="Tools",
        description="Write, run, inspect C++17 output, and trace CP Lang 0.2.2 programs directly in your browser.",
        duration="", number="", navigation=navigation("playground"),
        content=(ROOT / "playground.html").read_text().replace("{{editor}}", editor_html("playground")), toc="", pager="",
        name="playground", repo=REPO, page_class="playground", source_path="site/playground.html",
        docs_current="", cheat_sheet_current="", course_current="", series="CP LANG TOOLS", playground_current='aria-current="page"')
    page = template
    for key, value in values.items():
        page = page.replace("{{" + key + "}}", value)
    page = page.replace('</body>', '<script id="playground-examples" type="application/json">'
        + safe_json(examples) + '</script></body>')
    (target / "playground.html").write_text(page)


if __name__ == "__main__":
    build()
