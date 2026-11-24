import importlib.util
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

    def test_an_unread_formula_is_an_error(self):
        with self.assertRaises(ValueError):
            scaling.growth(r"\binom{n}{k}")

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


class Header(unittest.TestCase):
    def test_the_header_ends_at_the_first_other_line(self):
        text = "// scaling: bag.SortedBag.add\n// fixed: K\n// n: 512\nlet n: Int = read()\n// n: 9\n"
        self.assertEqual(scaling.header(text), {"scaling": "bag.SortedBag.add", "fixed": "K", "n": "512"})

    def test_every_case_names_its_fn_and_a_timed_line(self):
        for f in sorted(os.listdir(scaling.CASES)):
            text = open(os.path.join(scaling.CASES, f)).read()
            self.assertIn("scaling", scaling.header(text), f)
            self.assertIn("// timed", text, f)


if __name__ == "__main__":
    unittest.main()
