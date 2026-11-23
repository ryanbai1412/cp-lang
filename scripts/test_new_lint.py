#!/usr/bin/env python3
"""Unit tests for scripts/new-lint.py."""
import importlib.util
import os
import unittest

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "new-lint.py")
SPEC = importlib.util.spec_from_file_location("new_lint", SCRIPT)
assert SPEC and SPEC.loader
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class NewLintTest(unittest.TestCase):
    def test_reads_codes_from_records_and_the_diagnostics_table(self):
        text = 'code = "L0562"\nlevel = "warn"\n| L0564 | read_loop | x |\n| E0503 | y |\n'
        self.assertEqual(MOD.codes_in(text), {562, 564})

    def test_next_code_is_max_plus_one_within_its_series(self):
        used = {201, 211, 562, 564, 609}
        self.assertEqual(MOD.next_code(used, 5), 565)
        self.assertEqual(MOD.next_code(used, 2), 212)
        self.assertEqual(MOD.next_code(used, 3), 301)

    def test_a_full_series_is_an_error(self):
        with self.assertRaises(SystemExit):
            MOD.next_code({599}, 5)


if __name__ == "__main__":
    unittest.main()
