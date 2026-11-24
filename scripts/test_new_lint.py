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

    def test_the_default_series_moves_on_when_one_is_full(self):
        self.assertEqual(MOD.first_free({598, 599, 610}), 611)
        self.assertEqual(MOD.first_free({562}), 563)

    def test_next_needs_no_rule_but_a_record_does(self):
        self.assertEqual(MOD.main(["loop_sum", "--no-fetch", "--series", "6"]), 2)

    def test_a_catalog_row_is_claimed_with_its_family_and_text(self):
        catalog = "### S.seq: reductions\n\n| Lint | Catches | Write instead | Tier | P | Engine |\n|---|---|---|---|---|---|\n| loop_product | `acc *= f(x)` loop | `xs.map(f).product()` | fix | #2 | loop |\n| loop_xor_fold | `acc ^= x` loop | `xs.fold(Xor)` | fix | #2 | loop |\n"
        rest, row = MOD.claim_row(catalog, "loop_product")
        self.assertNotIn("loop_product", rest)
        self.assertIn("loop_xor_fold", rest)
        self.assertEqual(row, {"family": "S.seq: reductions", "catches": "`acc *= f(x)` loop",
                               "write": "`xs.map(f).product()`", "practice": "#2"})
        self.assertIsNone(MOD.claim_row(catalog, "loop_sum"))

    def test_a_toml_string_escapes_quotes(self):
        self.assertEqual(MOD.toml_str('`"a"`'), '"`\\"a\\"`"')


if __name__ == "__main__":
    unittest.main()
