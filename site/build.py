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

ROOT = Path(__file__).resolve().parent
REPO = "https://github.com/ryanbai1412/cp-lang"
REFERENCE_PAGES = [
    (
        "index",
        "Overview",
        "Start here",
        "A competitive-programming language targeting C++17.",
        "6 min",
    ),
    (
        "getting-started",
        "Getting started",
        "Start here",
        "Install the CLI and run a prototype example.",
        "5 min",
    ),
    (
        "tour",
        "A quick language tour",
        "Start here",
        "Values, functions, loops, and program structure.",
        "8 min",
    ),
    (
        "indices",
        "Index spaces & bounds",
        "The language",
        "Typed positions, boundaries, and indexed containers.",
        "10 min",
    ),
    (
        "borrowing",
        "Ownership & borrowing",
        "The language",
        "Temporary access, without lifetime annotations.",
        "9 min",
    ),
    (
        "strings",
        "ASCII strings",
        "The language",
        "Byte arrays with typed character positions.",
        "5 min",
    ),
    (
        "testing",
        "Memoization & stress tests",
        "In practice",
        "Cache recurrences and compare fast and reference solutions.",
        "8 min",
    ),
    (
        "stdlib",
        "The standard library",
        "In practice",
        "Containers, algorithms, and local libraries.",
        "7 min",
    ),
]

LESSONS = load_lessons()
EXERCISES = {exercise["id"]: exercise for lesson in LESSONS for exercise in lesson["exercises"]}
PAGES = [
    ("course", "Interactive course", "Course",
     "Six progressive chapters. Worked examples, exercises, and checked answers.", "6 chapters"),
    *[(lesson["slug"], lesson["title"], "Course", lesson["description"], lesson["duration"])
      for lesson in LESSONS],
    *REFERENCE_PAGES,
]


def slug(text):
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")


TOKEN = re.compile(
    r'(?P<comment>//[^\n]*|\#[^\n]*)|(?P<string>"(?:\\.|[^"\\])*"|\'[^\'\n]*\')|(?P<decorator>@\w+)|(?P<number>\b\d[\de_]*\b)|(?P<keyword>\b(?:let|var|fn|mut|return|if|else|for|in|while|domain|struct|impl|sealed|law|lends|use|type|enum|match|and|or|not|true|false|none|spec|requires|inline|const)\b)|(?P<type>\b[A-Z][A-Za-z0-9_]*\b)'
)


def highlight(code, language):
    if language != "cp":
        return html.escape(code)
    parts, end = [], 0
    for token in TOKEN.finditer(code):
        parts += [
            html.escape(code[end : token.start()]),
            f'<span class="tok-{token.lastgroup}">{html.escape(token.group())}</span>',
        ]
        end = token.end()
    return "".join(parts) + html.escape(code[end:])


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
        if language == "exercise":
            if len(fields) != 2 or fields[1] not in EXERCISES:
                raise ValueError(f"Unknown exercise directive: {info}")
            return exercise_html(EXERCISES[fields[1]], highlight)
        label = " ".join(fields[1:]) or {
            "cp": "CP Lang · design",
            "sh": "Terminal",
            "text": "Output",
        }.get(language, language)
        if language == "example":
            name = fields[1]
            code = (ROOT / "examples" / f"{name}.cp").read_text()
            language, label = "cp", f"{name}.cp · prototype tested"
        return f'<figure class="code-block"><figcaption><span>{html.escape(label)}</span><button class="copy-button" type="button" aria-label="Copy code">Copy</button></figcaption><pre tabindex="0"><code>{highlight(code.rstrip(), language)}</code></pre></figure>\n'


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


def build():
    target = ROOT / "_build"
    target.mkdir(exist_ok=True)
    shutil.copytree(ROOT / "assets", target / "assets", dirs_exist_ok=True)
    template = (ROOT / "template.html").read_text()
    search = []
    for i, (name, title, section, description, duration) in enumerate(PAGES):
        source = (ROOT / "content" / f"{name}.md").read_text()
        renderer = Renderer()
        body = mistune.create_markdown(renderer=renderer)(source)
        is_course = section == "Course"
        if is_course:
            body = progress_html(LESSONS, name if name != "course" else None) + body
        toc = "".join(
            f'<a href="#{anchor}">{text}</a>' for anchor, text in renderer.headings
        )
        pager = []
        sequence = ([("course", "Interactive course")]
                    + [(lesson["slug"], lesson["title"]) for lesson in LESSONS]
                    + [("playground", "Try the examples")]) if is_course else [page[:2] for page in REFERENCE_PAGES]
        position = next(j for j, page in enumerate(sequence) if page[0] == name)
        for index, direction in [(position - 1, "Previous"), (position + 1, "Next")]:
            if 0 <= index < len(sequence):
                other, label = sequence[index]
                pager.append(
                    f'<a class="pager-{direction.lower()}" href="{other}.html"><span>{direction} {"←" if direction == "Previous" else "→"}</span>{label}</a>'
                )
        heading = "CP Lang documentation" if name == "index" else html.escape(title)
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
            source_path=f"site/content/{name}.md",
            repo=REPO,
            page_class="overview" if name == "index" else "chapter",
            docs_current='' if is_course else 'aria-current="page"',
            course_current='aria-current="page"' if is_course else '',
            series="THE CP LANG COURSE" if is_course else "THE CP LANG GUIDE",
            playground_current="",
        )
        page = template
        for key, value in values.items():
            page = page.replace("{{" + key + "}}", value)
        if is_course:
            page = page.replace('</body>', '<script id="course-data" type="application/json">'
                + safe_json(course_data(name)) + '</script></body>')
        (target / f"{name}.html").write_text(page)
        plain = html.unescape(re.sub(r"<[^>]*>", " ", body))
        search.append(
            dict(
                title=title,
                section=section,
                url=f"{name}.html",
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
    trace = (ROOT.parent / "tools/trace/index.html").read_text()
    (target / "trace.html").write_text(trace.replace("../../site/_build/", ""))
    print(f"Built {len(PAGES)} chapters + playground + trace → {target}")


def safe_json(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c").replace("\u2028", "\\u2028").replace("\u2029", "\\u2029")


def course_data(name):
    """Ship compact checks for cross-chapter progress; keep feedback local."""
    checks = {}
    for lesson in LESSONS:
        for exercise in lesson["exercises"]:
            check = {"kind": exercise["kind"]}
            if exercise["kind"] == "choice":
                check["answer"] = exercise["answer"]
                if lesson["slug"] == name:
                    check["explanation"] = exercise["explanation"]
            else:
                check["tests"] = [{"expected": case["expected"]} for case in exercise["tests"]]
            checks[exercise["id"]] = check
    revisions = {key: hashlib.sha256(safe_json(exercise).encode()).hexdigest()[:16]
                 for key, exercise in EXERCISES.items()}
    return dict(lessons=[dict(slug=lesson["slug"], exercises=[exercise["id"] for exercise in lesson["exercises"]])
                         for lesson in LESSONS], exercises=checks, revisions=revisions)


def build_playground(target, template):
    examples = [dict(id=name, source=(ROOT / "examples" / f"{name}.cp").read_text())
                for name in ("sum", "bounds", "accumulation", "ascii", "optional", "memo")]
    values = dict(title="Playground", heading="Example playground", section="Tools",
        description="Run documented examples with your own input, entirely offline.",
        duration="", number="", navigation=navigation("playground"),
        content=(ROOT / "playground.html").read_text(), toc="", pager="",
        name="playground", repo=REPO, page_class="playground", source_path="site/playground.html",
        docs_current="", course_current="", series="CP LANG TOOLS", playground_current='aria-current="page"')
    page = template
    for key, value in values.items():
        page = page.replace("{{" + key + "}}", value)
    page = page.replace('</body>', '<script id="playground-examples" type="application/json">'
        + safe_json(examples) + '</script><script src="assets/playground.js"></script></body>')
    (target / "playground.html").write_text(page)


if __name__ == "__main__":
    build()
