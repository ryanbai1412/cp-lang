#!/usr/bin/env python3
"""Unit tests for scripts/dream-gaps.py.

    python3 -m unittest scripts/test_dream_gaps.py
"""
import importlib.util
import os
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("dream_gaps", os.path.join(HERE, "dream-gaps.py"))
dg = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dg)


class GapTest(unittest.TestCase):
    def test_planned_syntax_is_filed_under_its_feature(self):
        self.assertEqual(
            dg.gap("E0504", "not implemented yet: a wrap line, `--- with f() ---` (see design/spec/16-state.md §16.3)", ""),
            "syntax: a wrap line, `--- with f() ---`",
        )
        self.assertEqual(
            dg.gap("E0504", "not implemented yet: `guard let … else:` (not in the design)", ""),
            "syntax: `guard let … else:`",
        )

    def test_missing_library_names_are_filed_under_the_name(self):
        self.assertEqual(dg.gap("E0201", "no `clamp` in scope, and not in the stdlib (yet)", ""), "library: clamp")
        self.assertEqual(
            dg.gap("E0204", "no type or domain named `SmallVec`, and not in the stdlib (yet)", ""),
            "library: SmallVec",
        )
        self.assertEqual(dg.gap("E0203", "Move has no field or method `all`", ""), "library: Move.all")

    def test_a_missing_import_is_named(self):
        self.assertEqual(
            dg.gap("E0204", "no type or domain named `Perm`", "`Perm` is in the standard library: add `use std.seq.{Perm}`"),
            "import: Perm",
        )

    def test_a_generic_error_names_no_gap(self):
        self.assertIsNone(dg.gap("E0107", "expected the end of the statement, found `if`", ""))

    def test_summary_ranks_gaps_by_dreams_blocked(self):
        r = {
            "dreams": 2, "with_errors": 2, "first_named": 1, "errors": 3, "errors_named": 2,
            "gaps": [{"gap": "library: clamp", "dreams": 2, "first": 1, "errors": 2},
                     {"gap": "unnamed: x", "dreams": 1, "first": 1, "errors": 1}],
        }
        lines = dg.summary_lines(r)
        self.assertIn("first error names the gap in 1/2", lines[0])
        self.assertTrue(lines[2].endswith("library: clamp"))


    def test_missing_modules_and_module_names_are_library_gaps(self):
        self.assertEqual(
            dg.gap("E0206", "no module `std.persistent`, and not in the stdlib (yet)", ""),
            "library: std.persistent",
        )
        self.assertEqual(
            dg.gap("E0201", "`std.geometry` has no `Vec`, and not in the stdlib (yet)", ""),
            "library: std.geometry.Vec",
        )

    def test_backend_gaps_and_unknown_methods_are_named(self):
        self.assertEqual(
            dg.gap("E0501", "`cpl run` and `cpl build` can't handle generic fns and types yet", ""),
            "backend: generic fns and types",
        )
        self.assertEqual(
            dg.gap("E0303", "a `Range<Int>` has no method `group` that `cpl run` knows", ""),
            "library: Range<Int>.group",
        )

    def test_missing_arguments_and_implicit_fields_are_named(self):
        self.assertEqual(dg.gap("E0310", "`count` has no argument named `where`", ""), "library: count(where:)")
        self.assertEqual(
            dg.gap("E0201", "no `z` in scope",
                   "`z` is a field of `Prefix`, and this method takes no `self`: add `self` as its first parameter"),
            "syntax: a method that uses fields without taking `self`",
        )

    def test_a_decided_rule_is_named_as_one(self):
        self.assertTrue(
            dg.gap("E0102", "a top-level `return` takes no value", "").startswith("rule: ")
        )

if __name__ == "__main__":
    unittest.main()
