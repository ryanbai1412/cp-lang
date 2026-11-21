#!/usr/bin/env python3
"""Unit tests for scripts/writable-from-docs.py (offline, no API calls)."""

import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location("wfd", ROOT / "scripts/writable-from-docs.py")
wfd = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(wfd)


class TestWritableFromDocs(unittest.TestCase):
    def test_parse_atcoder_samples(self):
        md = """
Sample Input 1
```
3
1 2 3
```
Sample Output 1
```
6
```
Sample Input 2
```
1
42
```
Sample Output 2
```
42
```
"""
        self.assertEqual(
            wfd.parse_samples(md),
            [("3\n1 2 3\n", "6"), ("1\n42\n", "42")],
        )

    def test_parse_codeforces_samples(self):
        md = """
Examples
Input
```
2
3 4
```
Output
```
7
```
"""
        self.assertEqual(wfd.parse_samples(md), [("2\n3 4\n", "7")])

    def test_extract_cp_code(self):
        resp = "Here is the solution:\n```cp\nlet n: Int = read()\nprint(n)\n```\nDOCS_NOTES: none"
        self.assertEqual(wfd.extract_cp_code(resp), "let n: Int = read()\nprint(n)\n")
        self.assertIsNone(wfd.extract_cp_code("no fenced code here"))

    def test_load_docs_bundle_nonempty(self):
        bundle = wfd.load_docs_bundle(ROOT)
        self.assertIn("One-Page Contest Cheat Sheet", bundle)
        self.assertIn("site/content/stdlib.md", bundle)

    def test_ported_ids_excludes_dream(self):
        ids = wfd.ported_ids(ROOT)
        self.assertGreater(len(ids), 200)

    def test_format_markdown_report(self):
        md = wfd.format_markdown_report(
            [
                {
                    "id": "abc999a",
                    "samples_total": 2,
                    "samples_passed": 2,
                    "check_ok": True,
                    "build_ok": True,
                    "passed": True,
                    "error_codes": [],
                    "error_summary": "",
                }
            ],
            "test-model",
        )
        self.assertIn("Compile pass: **1/1**", md)
        self.assertIn("| `abc999a` | ok | ok | 2/2 | — | PASS |", md)


if __name__ == "__main__":
    unittest.main()
