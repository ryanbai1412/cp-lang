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

    def test_a_deferred_form_is_filed_as_deferred(self):
        self.assertEqual(
            dg.gap("E0504", "not implemented yet, and deferred: a context, `with io(base: 1)` (see design/spec/17-contexts.md §17.1)", ""),
            "deferred: a context, `with io(base: 1)`",
        )

    def test_a_member_of_a_generic_type_is_filed_without_its_arguments(self):
        self.assertEqual(dg.gap("E0203", "Graph<_, _> has no field or method `hops`", ""), "library: Graph.hops")

    def test_a_broken_checker_rule_is_named_by_the_rule(self):
        index = "rule: an index is a position of the array's own domain"
        self.assertEqual(dg.gap("E0301", "`rays` is indexed by Cnt, but `z` is an Int", ""), index)
        self.assertEqual(dg.gap("E0301", "this is indexed by `V`, but the index is a `Int`", ""), index)
        self.assertEqual(dg.gap("E0301", "`p - 1` is a ?Pos: it may step outside Pos", ""), index)
        self.assertEqual(
            dg.gap("E0305", "`small` takes 2 indices (V, 0..=cap), but 1 was given", ""),
            "rule: an array is indexed in every dimension at once",
        )
        self.assertEqual(
            dg.gap("E0402", "`raise` changes `f`, so the argument must be passed as `mut set[kept]`", ""),
            "rule: an argument the fn changes is passed with `mut`",
        )
        self.assertIsNone(dg.gap("E0302", "expected `Bool`, found `Int`", ""))

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
            dg.gap("E0310", "`MinHeap<Seg>` has no parameter named `by`", ""), "library: MinHeap<Seg>(by:)"
        )
        self.assertEqual(
            dg.gap("E0201", "no `z` in scope",
                   "`z` is a field of `Prefix`, and this method takes no `self`: add `self` as its first parameter"),
            "syntax: a method that uses fields without taking `self`",
        )

    def test_a_decided_rule_is_named_as_one(self):
        self.assertTrue(
            dg.gap("E0102", "a top-level `return` takes no value", "").startswith("rule: ")
        )

    def test_spellings_from_other_languages_are_named(self):
        self.assertEqual(dg.gap("E0107", "`..<` is `..` in cp", ""), "spelling: `..<` (write `..`)")
        self.assertEqual(
            dg.gap("E0004", "CP Lang concatenates with `+`", ""), "spelling: concatenates (write `+`)"
        )

    def test_a_domain_position_used_as_an_int_is_a_rule(self):
        rule = "rule: a position of a domain and an Int are different types"
        for msg in [
            "expected `Int`, found `Pos`",
            "expected `Int`, found `(Row, Col)`",
            "expected `I.bounds`, found `Int`",
            "can't compare a `V` with a `Int`",
            "`+` can't be applied to `Col.bounds` and `Int`",
        ]:
            self.assertEqual(dg.gap("E0302", msg, ""), rule, msg)
        self.assertIsNone(dg.gap("E0302", "expected `Str`, found `Int`", ""))
        self.assertIsNone(dg.gap("E0302", "expected `Int`, found `List<Int>`", ""))

    def test_an_array_used_as_a_list_is_a_rule(self):
        rule = "rule: an array over a domain and a List are different types"
        self.assertEqual(dg.gap("E0302", "expected `List<Mod<998244353>>`, found `[Mask]Mod<998244353>`", ""), rule)
        self.assertEqual(dg.gap("E0302", "`side` is a [Shut]Mod<_>, but `[s => s]` is a List<_>", ""), rule)

    def test_a_mut_let_condition_is_a_rule(self):
        self.assertEqual(
            dg.gap("E0105", "a binding in a `let` condition can't be `mut`", ""),
            "rule: a binding in a `let` condition can't change",
        )

    def test_a_question_mark_inside_a_branch_is_a_rule(self):
        msg = "`f(x)?` is inside a branch, a lambda or a comprehension, and can't leave the fn from there"
        self.assertTrue(dg.gap("E0309", msg, "").startswith("rule: "))

    def test_unknown_members_and_sibling_modules_are_named(self):
        self.assertEqual(
            dg.gap("E0203", "a `[Int]Int` has no member `positions` that `cpl run` knows", ""),
            "library: [Int]Int.positions",
        )
        self.assertEqual(dg.gap("E0206", "no module `cf1a`", ""), "import: module cf1a")

if __name__ == "__main__":
    unittest.main()
