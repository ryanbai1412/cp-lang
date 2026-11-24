import importlib.util
import math
import os
import unittest

spec = importlib.util.spec_from_file_location("scaling", os.path.join(os.path.dirname(__file__), "scaling.py"))
scaling = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scaling)


class Growth(unittest.TestCase):
    def test_the_documented_classes_grow_as_their_formulas(self):
        for formula, want in [("1", 1.0), (r"\log n", 16 / 14), ("n", 4.0), (r"n \log n", 4 * 16 / 14),
                              (r"\lvert D \rvert", 4.0), (r"\mathit{rows} \cdot \mathit{cols}", 16.0),
                              (r"\sqrt{n}", 2.0), (r"\alpha(n)", 1.0), ("n^2", 16.0), (r"E \sqrt V", 8.0)]:
            self.assertAlmostEqual(scaling.growth(formula), want, places=3, msg=formula)

    def test_a_fixed_symbol_does_not_grow(self):
        self.assertEqual(scaling.growth("K / 64", fixed=("K",)), 1.0)

    def test_a_let_symbol_grows_as_its_formula_in_n(self):
        self.assertAlmostEqual(scaling.growth("k 2^k", let={"k": r"\log n"}), 4 * 16 / 14, places=3)
        self.assertAlmostEqual(scaling.growth(r"\min(k, n - k)", let={"k": "n / 2"}), 4.0, places=3)

    def test_function_arguments_logs_and_bases(self):
        for formula, want in [(r"\log \max(\lvert x \rvert, \lvert y \rvert)", 16 / 14),
                              (r"(n + m) \log(n + m)", 4 * 17 / 15), (r"\log \log n", math.log2(16) / math.log2(14)),
                              (r"\mathit{log\_P} n", 16 / 14), (r"\min(\mathit{nodes}(\mathit{self}), m)", 4.0),
                              (r"\lvert \mathit{from} \rvert", 4.0)]:
            self.assertAlmostEqual(scaling.growth(formula, fixed=("P",)), want, places=3, msg=formula)

    def test_a_sum_grows_as_its_term_times_its_bound(self):
        self.assertEqual(scaling.growth(r"\sum_i p_i^{e_i}", fixed=("p_i", "e_i")), 1.0)
        self.assertAlmostEqual(scaling.growth(r"\sum_{i=1}^{k} \log n", fixed=("k",)), 16 / 14, places=3)
        self.assertAlmostEqual(scaling.growth(r"\sum_{i=1}^{k} 1"), 4.0, places=3)

    def test_an_unread_formula_is_an_error(self):
        with self.assertRaises(ValueError):
            scaling.growth(r"\binom{n}{k}")
        with self.assertRaises(ValueError):
            scaling.growth("k 2^k")

    def test_the_first_big_o_of_a_cost_line(self):
        self.assertEqual(scaling.big_o(r"\(O(\log n)\) expected, \(O(n)\) memory"), r"\log n")
        self.assertIsNone(scaling.big_o("as `SortedBag.join`"))


class Judge(unittest.TestCase):
    def test_a_linear_call_documented_constant_fails(self):
        self.assertFalse(scaling.judge(measured=4.0, documented=1.0))

    def test_a_quadratic_call_documented_linear_fails(self):
        self.assertFalse(scaling.judge(measured=16.0, documented=4.0))

    def test_docs_that_overstate_the_cost_fail(self):
        self.assertFalse(scaling.judge(measured=1.0, documented=16.0))

    def test_noise_around_the_documented_growth_passes(self):
        for measured, documented in [(0.8, 1.0), (1.5, 1.14), (5.0, 4.0), (3.1, 4.0)]:
            self.assertTrue(scaling.judge(measured, documented))


class Alias(unittest.TestCase):
    def test_a_cost_as_another_fn_reads_its_cost(self):
        costs = {"m.T.f": r"\(O(\log n)\)", "m.T.g": "as `f`", "m.U.h": "as [`T.g`](T.g), plus one call",
                 "m.T.k": "as `nope`"}
        self.assertEqual(scaling.alias("m.T.g", costs["m.T.g"], costs), costs["m.T.f"])
        self.assertEqual(scaling.alias("m.U.h", costs["m.U.h"], costs), costs["m.T.f"])
        self.assertEqual(scaling.alias("m.T.k", costs["m.T.k"], costs), "as `nope`")


class Header(unittest.TestCase):
    def test_the_header_ends_at_the_first_other_line(self):
        text = "// scaling: bag.SortedBag.add\n// fixed: K\n// n: 512\nlet n: Int = read()\n// n: 9\n"
        self.assertEqual(scaling.header(text), {"scaling": "bag.SortedBag.add", "fixed": "K", "n": "512"})

    def test_each_timed_line_names_its_fn_or_takes_the_headers(self):
        text = "// scaling: a.f\nx // timed\ny\nz // timed: b.g\nw // timed: b.g\n"
        self.assertEqual(scaling.timed_lines(text, "a.f"), {"a.f": {2}, "b.g": {4, 5}})

    def test_a_within_line_names_the_fn_its_time_counts_to(self):
        text = "let f = x => x // within: a.f\nx // timed: a.f\n// within: a.f is prose\n"
        self.assertEqual(scaling.within_lines(text), {"a.f": {1}})

    def test_every_case_times_a_named_fn(self):
        for f in sorted(os.listdir(scaling.CASES)):
            text = open(os.path.join(scaling.CASES, f)).read()
            groups = scaling.timed_lines(text, scaling.header(text).get("scaling"))
            self.assertTrue(groups, f)
            self.assertNotIn(None, groups, f)


if __name__ == "__main__":
    unittest.main()
