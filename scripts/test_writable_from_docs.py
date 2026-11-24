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
        self.assertIn("web/content/docs/guides/cheatsheet.mdx", bundle)
        self.assertIn("web/content/docs/guides/stdlib.mdx", bundle)
        self.assertIn("web/content/docs/guides/cpp-guide.mdx", bundle)
        self.assertIn("web/content/docs/guides/idioms.mdx", bundle)

    def test_manifest_has_50_plus_unported_problems(self):
        import json

        manifest = json.loads((ROOT / "research/writable-from-docs-set.json").read_text())
        probs = manifest["problems"]
        self.assertGreaterEqual(len(probs), 50)
        self.assertEqual(len(probs), len(set(probs)))
        ported = wfd.ported_ids()
        overlap = set(probs) & ported
        self.assertEqual(overlap, set(), f"manifest overlaps ported problems: {overlap}")

    def test_ratchet_check_passes_and_catches_regression(self):
        ratchet = {"min_compile_rate": 0.80, "min_pass_rate": 0.30}
        ok_summary = {"compile_rate": 0.85, "pass_rate": 0.35}
        self.assertEqual(wfd.check_ratchet_data(ratchet, ok_summary), [])

        bad_summary = {"compile_rate": 0.72, "pass_rate": 0.25}
        errs = wfd.check_ratchet_data(ratchet, bad_summary)
        self.assertEqual(len(errs), 2)
        self.assertIn("compile_rate", errs[0])
        self.assertIn("pass_rate", errs[1])

    def test_outputs_match_float_tolerance(self):
        self.assertTrue(wfd.outputs_match("3.2475579530543808", "3.2475579530543811"))
        self.assertFalse(wfd.outputs_match("3.24", "3.25"))
        self.assertTrue(wfd.outputs_match("YES\n42", "YES\n42"))
        self.assertFalse(wfd.outputs_match("YES", "NO"))

    def test_ported_ids_excludes_dream(self):
        ids = wfd.ported_ids()
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
        self.assertIn("**`cpl check` pass rate:** 1/1 (100.0%)", md)
        self.assertIn("| `abc999a` | PASS | PASS | 2/2 | PASS |", md)


if __name__ == "__main__":
    unittest.main()
