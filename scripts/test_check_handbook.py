#!/usr/bin/env python3
"""Unit tests for scripts/check-handbook.py."""
import importlib.util
import os
import unittest

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "check-handbook.py")
SPEC = importlib.util.spec_from_file_location("check_handbook", SCRIPT)
assert SPEC and SPEC.loader
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class CheckHandbookTest(unittest.TestCase):
    def test_flags_context_phrases_and_hashes(self):
        md = (
            "As discussed, the gate moved.\n"
            "Fixed in ccf30acf4 last night.\n"
            "Widths up to 2026112 bytes and the word deadbeef are fine.\n"
            "Quoting `as discussed` in a code span is fine.\n"
            "```\nas discussed ccf30acf4\n```\n"
        )
        errs = MOD.check_markdown("handbook/x.md", md)
        self.assertEqual(len(errs), 3, errs)
        self.assertIn(":1:", errs[0])
        self.assertTrue(all(":2:" in e for e in errs[1:]))

    def test_repo_handbook_is_clean(self):
        self.assertEqual(MOD.main([]), 0)


if __name__ == "__main__":
    unittest.main()
