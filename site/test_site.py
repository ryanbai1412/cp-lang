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
        self.assertEqual(len(pages), len(build.PAGES))
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

    def test_every_prototype_example_runs_as_native_cpp(self):
        fixtures = {
            "sum": [("5\n3 1 4 1 5\n", "14"), ("1\n-9\n", "-9")],
            "accumulation": [("4\n0 1 2 3\n", "6"), ("1\n0\n", "0")],
            "optional": [("", "0 10 10")],
            "ascii": [("5\nabaca\n", "3"), ("1\nz\n", "0")],
            "memo": [("4\n", "45"), ("1\n", "1")],
        }
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
                    cpp, _ = compile_program(parse(source))
                    path = Path(temporary) / f"{name}.cpp"
                    executable = Path(temporary) / name
                    path.write_text(cpp)
                    subprocess.run(
                        ["g++", "-std=c++17", "-O2", str(path), "-o", str(executable)],
                        check=True,
                        capture_output=True,
                        timeout=60,
                    )
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


if __name__ == "__main__":
    unittest.main(verbosity=2)
