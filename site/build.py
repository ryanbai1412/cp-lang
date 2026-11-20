#!/usr/bin/env python3
"""Build the CP Lang guide. No network, JavaScript bundler, or server required."""

import html
import json
from pathlib import Path
import re
import shutil

import mistune

ROOT = Path(__file__).resolve().parent
REPO = "https://github.com/ryanbai1412/cp-lang"
PAGES = [
    (
        "index",
        "Overview",
        "Start here",
        "The competitive programmer’s field guide.",
        "6 min",
    ),
    (
        "getting-started",
        "Getting started",
        "Start here",
        "From a fresh checkout to your first solution.",
        "5 min",
    ),
    (
        "tour",
        "A quick language tour",
        "Start here",
        "The familiar parts, with less ceremony.",
        "8 min",
    ),
    (
        "indices",
        "Index spaces & bounds",
        "The language",
        "Give every position a place to belong.",
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
        "One character. One byte. One indexing model.",
        "5 min",
    ),
    (
        "testing",
        "Memoization & stress tests",
        "In practice",
        "Write the recurrence. Keep a brute beside it.",
        "8 min",
    ),
    (
        "stdlib",
        "The standard library",
        "In practice",
        "A small core. A library you can understand.",
        "7 min",
    ),
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
        toc = "".join(
            f'<a href="#{anchor}">{text}</a>' for anchor, text in renderer.headings
        )
        pager = []
        for index, direction in [(i - 1, "Previous"), (i + 1, "Next")]:
            if 0 <= index < len(PAGES):
                other, label, *_ = PAGES[index]
                pager.append(
                    f'<a class="pager-{direction.lower()}" href="{other}.html"><span>{direction} {"←" if direction == "Previous" else "→"}</span>{label}</a>'
                )
        heading = (
            "Less bookkeeping.<br>More problem solving."
            if name == "index"
            else html.escape(title)
        )
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
            repo=REPO,
            page_class="overview" if name == "index" else "chapter",
        )
        page = template
        for key, value in values.items():
            page = page.replace("{{" + key + "}}", value)
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
    print(f"Built {len(PAGES)} pages → {target}")


if __name__ == "__main__":
    build()
