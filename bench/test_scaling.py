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

    def test_a_bare_deg_is_a_symbol(self):
        self.assertAlmostEqual(scaling.growth(r"(n + \deg) \log^2 n", fixed=("deg",), n=16.0),
                               (64 + 1) * 36 / ((16 + 1) * 16), places=6)
        self.assertAlmostEqual(scaling.growth(r"(\deg f + m) \log m"),
                               scaling.growth(r"(f + m) \log m"), places=6)

    def test_a_binomial_grows_as_its_value(self):
        self.assertAlmostEqual(scaling.growth("C(n, k)", let={"k": "2"}, n=15.0), 1770 / 105, places=3)
        self.assertAlmostEqual(scaling.growth("C(n, k)", fixed=("k",)), 4.0, places=3)

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


class Env(unittest.TestCase):
    def test_runs_pin_the_malloc_thresholds(self):
        self.assertEqual(scaling.profile_env({"A": "1"}),
                         {"A": "1", "GLIBC_TUNABLES": scaling.MALLOC_TUNABLES})

    def test_a_callers_tunables_are_kept(self):
        env = scaling.profile_env({"GLIBC_TUNABLES": "glibc.cpu.x=1"})
        self.assertEqual(env["GLIBC_TUNABLES"], "glibc.cpu.x=1:" + scaling.MALLOC_TUNABLES)


class Runs(unittest.TestCase):
    def test_runs_alternate_between_the_sizes(self):
        import json
        import tempfile
        from unittest import mock
        order = []

        def run(cmd, **_):
            order.append(int(open(cmd[4]).read()))
            out = {"lines": [{"line": 1, "time_ns": scaling.MIN_NS * order[-1], "start_count": 1}]}
            return mock.Mock(returncode=0, stdout=json.dumps(out), stderr="")
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(scaling.subprocess, "run", run):
            best, short = scaling.per_hit("cpl", "case.cp", (1, 4), {"f": {1}}, tmp)
        self.assertEqual(order, [1, 4] * scaling.RUNS)
        self.assertEqual(best, {1: {"f": scaling.MIN_NS}, 4: {"f": 4 * scaling.MIN_NS}})
        self.assertEqual(short, {})

    def test_a_fn_under_the_time_floor_fails_alone(self):
        import json
        import tempfile
        from unittest import mock
        out = {"lines": [{"line": 1, "time_ns": scaling.MIN_NS, "start_count": 1},
                         {"line": 2, "time_ns": 2 * scaling.MIN_NS, "start_count": 1}]}
        run = mock.Mock(return_value=mock.Mock(returncode=0, stdout=json.dumps(out), stderr=""))
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(scaling.subprocess, "run", run):
            _, short = scaling.per_hit("cpl", "case.cp", (1,), {"f": {1}, "g": {2}}, tmp)
            self.assertEqual(short, {})
            _, short = scaling.per_hit("cpl", "case.cp", (1,), {"f": {1}, "g": {2}}, tmp,
                                       floor=1.5 * scaling.MIN_NS)
            self.assertEqual(list(short), ["f"])


class Merge(unittest.TestCase):
    def test_a_rerun_replaces_its_cases_and_keeps_the_others(self):
        case = lambda fn, ok=True: {"fn": fn, "ok": ok}
        old = {"cases": {"bench/scaling/dsu.cp::dsu.Dsu.find": case("dsu.Dsu.find"),
                         "bench/scaling/rooted.cp::tree.Rooted.lca": case("tree.Rooted.lca", ok=False),
                         "bench/scaling/rooted.cp::tree.Rooted.gone": case("tree.Rooted.gone"),
                         "bench/scaling/no_such_case.cp::x.y": case("x.y")}}
        results = {"bench/scaling/rooted.cp::tree.Rooted.lca": case("tree.Rooted.lca")}
        board = scaling.merge(old, results, [os.path.join(scaling.ROOT, "bench/scaling/rooted.cp")],
                              ["dsu.Dsu.find", "tree.Rooted.lca", "tree.Rooted.gone", "x.y"])
        self.assertEqual(sorted(board["cases"]), ["bench/scaling/dsu.cp::dsu.Dsu.find",
                                                  "bench/scaling/rooted.cp::tree.Rooted.lca"])
        self.assertTrue(board["cases"]["bench/scaling/rooted.cp::tree.Rooted.lca"]["ok"])
        self.assertEqual(board["unscaled"], ["tree.Rooted.gone", "x.y"])


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
