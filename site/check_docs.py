#!/usr/bin/env python3
"""Fast documentation gate for scripts/check.sh --fast.

Checks every documented `cp` code fence (`cpl check`), runs every `site/examples/*.cp`
fixture and write-code exercise reference through `cpl run`, verifies the one-page
cheat-sheet line budget, and checks all local links/fragments in `site/_build/`.
"""

from html.parser import HTMLParser
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
from urllib.parse import unquote, urlsplit

import mistune

import build

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent
CPL = REPO / "target/debug/cpl"

EXAMPLE_FIXTURES = {
    "bounds": [("5\n3 1 4 1 5\n", "14"), ("1\n-9\n", "-9")],
    "sum": [("5\n3 1 4 1 5\n", "14"), ("1\n-9\n", "-9")],
    "knapsack": [("3 7\n3 4\n4 5\n2 3\n", "9"), ("2 3\n4 10\n5 20\n", "0")],
    "range_array": [("6 4\n0 2 4 2 1 2\n", "1 1 3 0 1")],
    "accumulation": [("4\n0 1 2 3\n", "6"), ("1\n0\n", "0")],
    "optional": [("5\n3 7 4 1 8\n", "8"), ("3\n1 3 5\n", "-1")],
    "ascii": [("5\nabaca\n", "3"), ("1\nz\n", "0")],
    "memo": [("4\n", "45"), ("1\n", "1")],
}


def documentation_paths():
    return [*sorted((ROOT / "content").glob("*.md")), REPO / "docs/one-page-cheat-sheet.md"]


def documented_programs():
    markdown = mistune.create_markdown(renderer="ast")

    def blocks(tokens):
        for token in tokens:
            if token["type"] == "block_code":
                info = token.get("attrs", {}).get("info", "").split()
                if info and info[0].lower() == "cp":
                    yield token["raw"]
            yield from blocks(token.get("children", []))

    for path in documentation_paths():
        for number, source in enumerate(blocks(markdown(path.read_text())), 1):
            yield path, number, source


class LinkPage(HTMLParser):
    def __init__(self, source):
        super().__init__()
        self.links, self.ids, self.duplicates = [], set(), []
        self.feed(source)

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if "id" in attrs:
            if attrs["id"] in self.ids:
                self.duplicates.append(attrs["id"])
            self.ids.add(attrs["id"])
        if tag in ("a", "link", "script"):
            self.links.append(attrs.get("href", attrs.get("src", "")))


def check_cheat_sheet_budget():
    lines = len((REPO / "docs/one-page-cheat-sheet.md").read_text().splitlines())
    if lines > 250:
        sys.exit(f"check_docs: docs/one-page-cheat-sheet.md has {lines} lines (cap 250)")
    return lines


def check_doc_fences():
    programs = list(documented_programs())
    if not programs:
        sys.exit("check_docs: found 0 documented cp blocks")
    with tempfile.TemporaryDirectory(prefix="cpl-docs-fast-") as temporary:
        tmp = Path(temporary)
        library = REPO / "examples/local-lib"
        (tmp / "cp.toml").write_text(f'[libs]\nlib = "{library.as_posix()}"\n')
        files = []
        mapping = {}
        for path, number, source in programs:
            cp = tmp / f"{path.stem}-{number:03d}.cp"
            cp.write_text(source)
            files.append(str(cp))
            mapping[str(cp)] = f"{path.relative_to(REPO)} block #{number}"
        res = subprocess.run([str(CPL), "check", *files], cwd=REPO, capture_output=True, text=True, timeout=60)
        if res.returncode != 0:
            out = res.stdout + res.stderr
            for k, v in mapping.items():
                out = out.replace(k, v)
            sys.exit(f"check_docs: documented cp block failed `cpl check`:\n{out}")
    return len(programs)


def run_examples():
    fixtures = {k: list(v) for k, v in EXAMPLE_FIXTURES.items()}
    for lesson in build.LESSONS:
        for exercise in lesson["exercises"]:
            if exercise["kind"] == "output":
                fixtures.setdefault(exercise["fixture"], []).extend(
                    (case["input"], case["expected"]) for case in exercise["tests"]
                )
    on_disk = {p.stem for p in (ROOT / "examples").glob("*.cp")}
    if on_disk != set(fixtures):
        sys.exit(f"check_docs: site/examples mismatch: extra={on_disk - set(fixtures)} missing={set(fixtures) - on_disk}")
    cases_run = 0
    for name, cases in sorted(fixtures.items()):
        source_path = ROOT / "examples" / f"{name}.cp"
        for data, expected in cases:
            res = subprocess.run(
                [str(CPL), "run", str(source_path)],
                cwd=REPO,
                input=data,
                capture_output=True,
                text=True,
                timeout=15,
            )
            if res.returncode != 0:
                sys.exit(f"check_docs: example {name}.cp failed:\n{res.stderr}")
            if res.stdout.split() != expected.split():
                sys.exit(
                    f"check_docs: example {name}.cp output mismatch: "
                    f"got {res.stdout.split()!r}, expected {expected.split()!r}"
                )
            cases_run += 1
    return len(fixtures), cases_run


def run_course_references():
    refs = [ex for ex in build.EXERCISES.values() if ex["kind"] == "code"]
    cases_run = 0
    with tempfile.TemporaryDirectory(prefix="cpl-refs-fast-") as temporary:
        tmp = Path(temporary)
        for ex in refs:
            cp = tmp / f'{ex["id"]}.cp'
            cp.write_text(ex["reference"])
            for case in ex["tests"]:
                res = subprocess.run(
                    [str(CPL), "run", str(cp)],
                    cwd=REPO,
                    input=case["input"],
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                if res.returncode != 0:
                    sys.exit(f'check_docs: exercise reference {ex["id"]} failed:\n{res.stderr}')
                if res.stdout.split() != case["expected"].split():
                    sys.exit(
                        f'check_docs: exercise reference {ex["id"]} output mismatch: '
                        f'got {res.stdout.split()!r}, expected {case["expected"].split()!r}'
                    )
                cases_run += 1
    return len(refs), cases_run


def check_links():
    target = ROOT / "_build"
    if not target.is_dir():
        sys.exit("check_docs: site/_build missing; run site/build.py or site/bundle.py first")
    pages = {p.name: LinkPage(p.read_text()) for p in sorted(target.glob("*.html"))}
    for name, page in pages.items():
        if page.duplicates:
            sys.exit(f"check_docs: duplicate ids in {name}: {page.duplicates}")
        for link in page.links:
            url = urlsplit(link)
            if url.scheme or url.netloc:
                continue
            path = unquote(url.path) or name
            if not (target / path).is_file():
                sys.exit(f"check_docs: broken link in {name}: {link}")
            if url.fragment and unquote(url.fragment) not in pages[path].ids:
                sys.exit(f"check_docs: broken fragment in {name}: {link}")
    return len(pages)


def main():
    t0 = time.monotonic()
    if not CPL.is_file():
        sys.exit(f"check_docs: {CPL} not built; run cargo build -p cpl-cli")
    cheat_lines = check_cheat_sheet_budget()
    fence_count = check_doc_fences()
    ex_count, ex_cases = run_examples()
    ref_count, ref_cases = run_course_references()
    page_count = check_links()
    ms = int((time.monotonic() - t0) * 1000)
    print(
        f"docs gate: {fence_count} cp fences, {ex_count} examples ({ex_cases} runs), "
        f"{ref_count} course refs ({ref_cases} runs), {page_count} HTML pages, "
        f"cheat-sheet {cheat_lines}/250 lines — ok in {ms} ms"
    )


if __name__ == "__main__":
    main()
