#!/usr/bin/env python3
"""Unit tests for scripts/check-doc-links.py."""
import importlib.util
import os
import tempfile
import unittest

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "check-doc-links.py")
SPEC = importlib.util.spec_from_file_location("check_doc_links", SCRIPT)
assert SPEC and SPEC.loader
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class CheckDocLinksTest(unittest.TestCase):
    def test_accepts_existing_and_flags_missing(self):
        with tempfile.TemporaryDirectory() as tmp:
            open(os.path.join(tmp, "exists.md"), "w").close()
            md = (
                "See [good](exists.md#sec), [url](https://example.com), "
                "[anchor](#top), [bad](missing.md).\n"
                "```\n[ignored](also_missing.md)\n```\n"
            )
            errs = MOD.check_markdown("README.md", md, tmp)
            self.assertEqual(len(errs), 1)
            self.assertIn("missing.md", errs[0])


if __name__ == "__main__":
    unittest.main()
