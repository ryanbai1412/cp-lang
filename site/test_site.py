#!/usr/bin/env python3
"""Build/link checks and native execution of every prototype-tested snippet."""

from html.parser import HTMLParser
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest
from urllib.parse import unquote, urlsplit

import build

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent
sys.path.insert(0, str(REPO / "proto"))
from cpc.compiler import compile_program  # noqa: E402
from cpc.parser import parse  # noqa: E402


def compile_native(source, name, directory):
    cpp, _ = compile_program(parse(source))
    path, executable = Path(directory) / f"{name}.cpp", Path(directory) / name
    path.write_text(cpp)
    subprocess.run(
        ["g++", "-std=c++17", "-O2", str(path), "-o", str(executable)],
        check=True, capture_output=True, timeout=60)
    return executable


class Page(HTMLParser):
    def __init__(self, source):
        super().__init__()
        self.links, self.ids, self.duplicates = [], set(), []
        self.buttons = []
        self.feed(source)

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if tag == "button":
            self.buttons.append(attrs)
        if "id" in attrs:
            if attrs["id"] in self.ids:
                self.duplicates.append(attrs["id"])
            self.ids.add(attrs["id"])
        if tag in ("a", "link", "script"):
            self.links.append(attrs.get("href", attrs.get("src", "")))


class SiteTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        build.build()

    def test_all_local_links_and_fragments_resolve(self):
        target = ROOT / "_build"
        pages = {p.name: Page(p.read_text()) for p in target.glob("*.html")}
        self.assertEqual(len(pages), len(build.PAGES) + 2)
        for name, page in pages.items():
            self.assertEqual(page.duplicates, [], name)
            for link in page.links:
                url = urlsplit(link)
                if url.scheme or url.netloc:
                    continue
                path = unquote(url.path) or name
                self.assertTrue((target / path).is_file(), f"{name}: {link}")
                if url.fragment:
                    self.assertIn(
                        unquote(url.fragment), pages[path].ids, f"{name}: {link}"
                    )

    def test_search_covers_every_chapter(self):
        index = json.loads((ROOT / "_build/search.json").read_text())
        self.assertEqual(len(index), len(build.PAGES))
        for page in index:
            self.assertTrue(page["text"].strip())
            self.assertTrue((ROOT / "_build" / page["url"]).is_file())

    def test_responsive_toolbar_controls_have_accessible_names(self):
        controls = {"search-trigger", "theme-toggle", "menu-toggle"}
        for path in (ROOT / "_build").glob("*.html"):
            if path.name == "trace.html":
                continue
            page = Page(path.read_text())
            found = set()
            for button in page.buttons:
                matches = controls.intersection(button.get("class", "").split())
                if matches:
                    self.assertTrue(button.get("aria-label", "").strip(), path.name)
                    found.update(matches)
            self.assertEqual(found, controls, path.name)

    def test_renderer_escapes_code_and_uses_unique_headings(self):
        renderer = build.Renderer()
        self.assertIn("&lt;script&gt;", renderer.block_code("<script>", "text"))
        self.assertIn('id="repeat"', renderer.heading("Repeat", 2))
        self.assertIn('id="repeat-2"', renderer.heading("Repeat", 2))

    def test_no_unexpanded_template_fields(self):
        for path in (ROOT / "_build").glob("*.html"):
            self.assertNotRegex(path.read_text(), r"\{\{[a-z_]+\}\}")

    def test_offline_bundle_has_all_pages_without_font_blobs(self):
        import bundle
        path = bundle.bundle()
        source = path.read_text()
        payload = json.loads(re.search(
            r'<script id="offline-data" type="application/json">(.*?)</script>',
            source, re.S).group(1))
        self.assertEqual(set(payload["pages"]),
                         {name + ".html" for name, *_ in build.PAGES}
                         | {"playground.html", "trace.html"})
        self.assertNotIn("@font-face", source)
        self.assertNotIn("data:font", source)
        self.assertLess(path.stat().st_size, 800_000)
        self.assertEqual(len(json.loads(payload["search"])), len(build.PAGES))
        self.assertIn("course", payload)

    def test_course_has_six_sequenced_lessons_and_real_controls(self):
        self.assertEqual(len(build.LESSONS), 6)
        self.assertGreaterEqual(len(build.EXERCISES), 18)
        for lesson in build.LESSONS:
            source = (ROOT / "_build" / f'{lesson["slug"]}.html').read_text()
            page = Page(source)
            for exercise in lesson["exercises"]:
                self.assertIn(exercise["id"], page.ids)
                self.assertIn(f'id="{exercise["id"]}-feedback" role="status"', source)
            self.assertIn('id="course-data" type="application/json"', source)
            self.assertIn("Solution and explanation", source)
        data = build.course_data("course")
        self.assertEqual(set(data["exercises"]), set(build.EXERCISES))
        self.assertEqual(set(data["revisions"]), set(build.EXERCISES))

    def test_exercise_prose_and_options_cannot_inject_markup(self):
        from course import exercise_html
        question = dict(id="escape-check", kind="choice", title="<script>alert(1)</script>",
                        prompt="<img src=x>", hint="<svg/onload=alert(1)>", solution="<b>not markup</b>",
                        options=['<input checked>', '" onclick="alert(1)'], answer=0)
        source = exercise_html(question, build.highlight)
        self.assertNotIn("<script>", source)
        self.assertNotIn("<img", source)
        self.assertIn("&lt;input checked&gt;", source)

    def test_every_course_page_can_validate_progress_from_every_chapter(self):
        expected = set(build.EXERCISES)
        for name in ["course", *[lesson["slug"] for lesson in build.LESSONS]]:
            with self.subTest(page=name):
                data = build.course_data(name)
                self.assertEqual(set(data["exercises"]), expected)
                for key, exercise in data["exercises"].items():
                    original = build.EXERCISES[key]
                    self.assertEqual(exercise["kind"], original["kind"])
                    if exercise["kind"] == "choice":
                        self.assertEqual(exercise["answer"], original["answer"])
                    else:
                        self.assertEqual([case["expected"] for case in exercise["tests"]],
                                         [case["expected"] for case in original["tests"]])

    def test_course_worked_programs_run_as_shown(self):
        cases = {
            "max-and-evens": [("5\n-3 8 -4 7 8\n", "8 3")],
            "range-sums": [("5 3\n3 1 4 1 5\n1 5\n2 3\n4 4\n", "14 5 1")],
            "best-window": [("5 2\n3 -1 4 1 5\n", "6"),
                            ("5 5\n3 -1 4 1 5\n", "12"),
                            ("3 1\n-2 -9 -4\n", "-2")],
            "ordinals": [("5\n", "5 3 2 2 YES")],
            "user-index": [("2 3\n", "6 23 5 2")],
            "grid": [("2 3\n", "21 21 YES")],
            "slices": [("5\n9 2 7 4 5\n", "3 7 3 13\n9 2 4 7 5\n2 4 7 0")],
        }
        programs = {}
        for name in ("first-program", "domains", "index-slices"):
            source = (ROOT / "content" / f"course-{name}.md").read_text()
            programs.update(re.findall(r"```cp Prototype tested · ([a-z-]+)\.cp\n(.*?)```", source, re.S))
        self.assertEqual(set(programs), set(cases))
        with tempfile.TemporaryDirectory(prefix="cpl-course-") as temporary:
            for name, source in programs.items():
                with self.subTest(program=name):
                    executable = compile_native(source, name, temporary)
                    for data, expected in cases[name]:
                        run = subprocess.run([str(executable)], input=data, text=True,
                                             capture_output=True, check=True, timeout=5)
                        self.assertEqual(run.stdout.split(), expected.split())

    def test_every_prototype_example_runs_as_native_cpp(self):
        fixtures = {
            "bounds": [("5\n3 1 4 1 5\n", "14"), ("1\n-9\n", "-9")],
            "sum": [("5\n3 1 4 1 5\n", "14"), ("1\n-9\n", "-9")],
            "accumulation": [("4\n0 1 2 3\n", "6"), ("1\n0\n", "0")],
            "optional": [("", "0 10 10")],
            "ascii": [("5\nabaca\n", "3"), ("1\nz\n", "0")],
            "memo": [("4\n", "45"), ("1\n", "1")],
        }
        # Course predictions are checked against actual compiled CP source,
        # not merely against a second copy of their answer key.
        for lesson in build.LESSONS:
            for exercise in lesson["exercises"]:
                if exercise["kind"] == "output":
                    fixtures[exercise["fixture"]].extend(
                        (case["input"], case["expected"]) for case in exercise["tests"])
        referenced = set()
        for path in (ROOT / "content").glob("*.md"):
            referenced.update(re.findall(r"```example (\w+)", path.read_text()))
        self.assertEqual(referenced, set(fixtures))
        self.assertEqual(
            {p.stem for p in (ROOT / "examples").glob("*.cp")}, set(fixtures)
        )
        with tempfile.TemporaryDirectory(prefix="cpl-docs-") as temporary:
            for name, cases in fixtures.items():
                with self.subTest(example=name):
                    source = (ROOT / "examples" / f"{name}.cp").read_text()
                    executable = compile_native(source, name, temporary)
                    for data, expected in cases:
                        run = subprocess.run(
                            [str(executable)],
                            input=data,
                            text=True,
                            capture_output=True,
                            check=True,
                            timeout=5,
                        )
                        self.assertEqual(run.stdout.split(), expected.split())
                        # The explicitly labeled browser runner must agree with
                        # the real prototype, including ASCII and memo examples.
                        runner = (ROOT / "assets/playground.js").read_text()
                        browser = subprocess.run(
                            ["node", "-e", runner + "\nconsole.log(ExampleRunner.run("
                             + json.dumps(name) + "," + json.dumps(data) + "))"],
                            text=True, capture_output=True, check=True, timeout=5)
                        self.assertEqual(browser.stdout.split(), run.stdout.split())


if __name__ == "__main__":
    unittest.main(verbosity=2)
