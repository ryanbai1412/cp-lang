#!/usr/bin/env python3
"""Build/link checks and native execution of every prototype-tested snippet."""

from html.parser import HTMLParser
import html as html_lib
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
        self.assertLess(path.stat().st_size, 2_600_000)
        self.assertEqual(payload["compiler"].count("const packedWasm ="), 1)
        self.assertNotIn("const packedWasm =", "".join(payload["pages"].values()))
        self.assertEqual(len(json.loads(payload["search"])), len(build.PAGES))
        self.assertIn("course", payload)
        self.assertIn("course_data", payload)

    def test_course_has_ten_sequenced_lessons_and_real_controls(self):
        self.assertEqual(len(build.LESSONS), 10)
        self.assertGreaterEqual(len(build.EXERCISES), 45)
        for lesson in build.LESSONS:
            source = (ROOT / "_build" / f'{lesson["slug"]}.html').read_text()
            page = Page(source)
            for exercise in lesson["exercises"]:
                self.assertIn(exercise["id"], page.ids)
                self.assertIn(f'id="{exercise["id"]}-feedback" role="status"', source)
                if exercise["kind"] == "output":
                    self.assertIn(
                        '<p class="exercise-sandbox-note">This sandbox is independent. '
                        "Check answer uses the original program and the inputs shown above.</p>",
                        source,
                    )
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

    def test_safe_json_escapes_script_breakers_and_round_trips_unicode(self):
        sample = {
            "close": "</ScRiPt><script>alert(1)</script>",
            "comment": "<!--<script>bad</script>-->",
            "angle": "a < b && c > d",
            "unicode": "π → λ\u2028line\u2029para",
        }
        encoded = build.safe_json(sample)
        self.assertNotIn("</", encoded)
        self.assertNotIn("<!--", encoded)
        self.assertNotIn("\u2028", encoded)
        self.assertNotIn("\u2029", encoded)
        self.assertEqual(json.loads(encoded), sample)

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
            "grid-sum": [("2 3\n", "21 21 YES")],
            "slices": [("5\n9 2 7 4 5\n", "3 7 13\n9 2 4 7 5")],
        }
        programs = {}
        for name in ("first-program", "domains", "index-slices"):
            source = (ROOT / "content" / f"course-{name}.md").read_text()
            programs.update(
                re.findall(
                    r"```cp CP Lang(?: 0\.2\.2)? · ([a-z-]+)\.cp(?: [^\n]*)?\n(.*?)```",
                    source,
                    re.S,
                )
            )
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
            "knapsack": [("3 7\n3 4\n4 5\n2 3\n", "9"), ("2 3\n4 10\n5 20\n", "0")],
            "range_array": [("6 4\n0 2 4 2 1 2\n", "1 1 3 0 1")],
            "accumulation": [("4\n0 1 2 3\n", "6"), ("1\n0\n", "0")],
            "optional": [("5\n3 7 4 1 8\n", "8"), ("3\n1 3 5\n", "-1")],
            "ascii": [("5\nabaca\n", "3"), ("1\nz\n", "0")],
            "memo": [("4\n", "45"), ("1\n", "1")],
        }
        # 0.2.2 fixtures that use Rust/WASM stdlib or syntax verified in test_compiler.cjs.
        wasm_only = {
            "early_return",
            "multitest_divider",
            "lends",
            "tropical_dp",
            "optional_coalesce",
            "memo_dp",
            "segtree_monoid",
            "lazy_action",
            "colocated_brute",
            "enum_match",
            "collections_heap",
            "sort_bsearch",
            "fenwick_sum",
            "dsu_mod",
        }
        # Course predictions are checked against actual compiled CP source,
        # not merely against a second copy of their answer key.
        for lesson in build.LESSONS:
            for exercise in lesson["exercises"]:
                if exercise["kind"] == "output" and exercise["fixture"] in fixtures:
                    fixtures[exercise["fixture"]].extend(
                        (case["input"], case["expected"]) for case in exercise["tests"])
        referenced = set()
        for path in (ROOT / "content").glob("*.md"):
            referenced.update(re.findall(r"```example (\w+)", path.read_text()))
        self.assertEqual(referenced, set(fixtures) | wasm_only)
        self.assertEqual(
            {p.stem for p in (ROOT / "examples").glob("*.cp")}, set(fixtures) | wasm_only
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

    def test_no_truncated_showcase_stubs_and_full_highlighting(self):
        index_html = (ROOT / "_build/index.html").read_text()
        self.assertNotRegex(index_html, r"//\s*\.\.\.\s*plus\s+\d+\s+lines")
        self.assertNotIn("plus 100 lines", index_html)
        self.assertIn('data-lang="cpp"', index_html)
        self.assertIn('data-lang="cp"', index_html)
        self.assertIn('class="tok-keyword"', index_html)
        self.assertIn('class="tok-type"', index_html)
        self.assertIn('class="code-expand-btn"', index_html)
        self.assertIn('aria-expanded="false"', index_html)

    def test_syntax_highlighting_for_all_supported_languages(self):
        cp_html = build.highlight("@memo\nfn dp(u: V) -> Int: // comment\n    return 0", "cp")
        self.assertIn('class="tok-decorator">@memo', cp_html)
        self.assertIn('class="tok-keyword">fn', cp_html)
        self.assertIn('class="tok-type">Int', cp_html)
        self.assertIn('class="tok-comment">// comment', cp_html)

        cpp_html = build.highlight('#include <vector>\nvector<int> a; // hi\nreturn 0;', "cpp")
        self.assertIn('class="tok-preproc">#include', cpp_html)
        self.assertIn('class="tok-type">vector', cpp_html)
        self.assertIn('class="tok-keyword">return', cpp_html)
        self.assertIn('class="tok-comment">// hi', cpp_html)

        sh_html = build.highlight('cpl build sum.cp # comment', "sh")
        self.assertIn('class="tok-keyword">cpl', sh_html)
        self.assertIn('class="tok-comment"># comment', sh_html)

        toml_html = build.highlight('[libs]\nlib = "../my-lib"', "toml")
        self.assertIn('class="tok-decorator">[libs]', toml_html)
        self.assertIn('class="tok-string">&quot;../my-lib&quot;', toml_html)

    def test_cheat_sheet_uses_single_canonical_source(self):
        self.assertFalse(
            (ROOT / "content/cheat-sheet.md").exists(),
            "site/content/cheat-sheet.md must not exist; build.py loads docs/one-page-cheat-sheet.md directly.",
        )
        canonical = (REPO / "docs/one-page-cheat-sheet.md").read_text()
        built = (ROOT / "_build/cheat-sheet.html").read_text()
        plain = html_lib.unescape(re.sub(r"<[^>]+>", "", built))
        self.assertIn("docs/one-page-cheat-sheet.md", built)
        for heading in re.findall(r"^##\s+(.+)$", canonical, re.MULTILINE):
            clean = re.sub(r"`([^`]+)`", r"\1", heading).strip()
            self.assertIn(clean.split("(")[0].strip(), plain)

    def test_no_internal_dev_jargon_in_reader_copy(self):
        forbidden = [
            r"Prototype tested",
            r"decisions\.md",
            r"plan/19-",
            r"research/language-architecture",
            r"crates/cpl-ir",
            r"\bloc\.py\b",
            r"proto/ports/",
            r"@spec\b",
            r"\bv2\.2\b",
            r"six chapters",
            r"seven chapters",
            r"//\s*\.\.\.\s*plus\s+\d+\s+lines",
        ]
        pattern = re.compile("|".join(forbidden), re.IGNORECASE)
        for path in sorted((ROOT / "_build").glob("*.html")):
            if path.name == "trace.html":
                continue
            html = path.read_text()
            visible = re.sub(r"<script\b[^>]*>.*?</script>", "", html, flags=re.S)
            visible = re.sub(r"<style\b[^>]*>.*?</style>", "", visible, flags=re.S)
            visible = re.sub(r'href="[^"]*"', "", visible)
            match = pattern.search(visible)
            self.assertIsNone(match, f"{path.name} contains internal/stale text: {match.group(0) if match else ''}")

    def test_sidebar_sections_are_not_duplicated(self):
        html = (ROOT / "_build/index.html").read_text()
        sidebar = re.search(r'<aside class="sidebar" id="sidebar">.*?</aside>', html, flags=re.S).group(0)
        headings = re.findall(r'<p class="nav-heading">([^<]+)</p>', sidebar)
        self.assertEqual(len(headings), len(set(headings)), f"Duplicate sidebar headings: {headings}")


if __name__ == "__main__":
    unittest.main(verbosity=2)
