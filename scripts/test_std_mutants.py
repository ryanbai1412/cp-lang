import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import std_mutants as sm  # noqa: E402


class SitesTest(unittest.TestCase):
    def test_strings_and_comments_are_not_mutated(self) -> None:
        got = [m[3] for m in sm.line_mutants('    assert a < b, "x < y" // p < q')]
        self.assertEqual(got, ['    assert a <= b, "x < y" // p < q'])

    def test_each_operator_at_each_place_once(self) -> None:
        got = sm.line_mutants("    if not a and x == y + 1: return true")
        self.assertEqual(sorted((c, w, n) for c, w, n, _ in got),
                         [(8, "not", "nothing"), (14, "and", "or"), (20, "==", "!="),
                          (25, "+", "-"), (37, "true", "false")])
        self.assertIn("    if a and x == y + 1: return true", [m[3] for m in got])

    def test_a_declaration_line_is_mutated_only_in_its_body(self) -> None:
        self.assertEqual(sm.line_mutants("pub fn f<T: A + B>(x: T) -> Int:"), [])
        got = sm.line_mutants("    pub fn g(self, x: Int = 1 + 2) -> Int = x + 1")
        self.assertEqual([m[3] for m in got], ["    pub fn g(self, x: Int = 1 + 2) -> Int = x - 1"])
        self.assertEqual(sm.line_mutants("@inline(a + b)"), [])

    def test_test_fns_are_test_code(self) -> None:
        lines = ["pub fn f() = 1 + 1", "@test", "fn t():", "    assert f() == 2", "", "pub fn g() = 2 + 2"]
        self.assertEqual(sm.test_fn_lines(lines), {1, 2, 3, 4})

    def test_the_item_is_the_enclosing_top_level_fn_or_type(self) -> None:
        lines = ["pub struct Fen<T>:", "    t: List<T>", "impl<T> Fen<T>:", "    pub fn add(self):",
                 "        x += 1", "impl<T: Add> Show for Fen<T>:", "    fn show(self) = 1", "pub fn lone() = 1"]
        self.assertEqual([sm.item_at(lines, i) for i in (1, 4, 6, 7)], ["Fen", "Fen", "Fen", "lone"])

    def test_diff_gives_the_new_sides_lines(self) -> None:
        diff = ("diff --git a/std/a.cp b/std/a.cp\n--- a/std/a.cp\n+++ b/std/a.cp\n@@ -3,0 +4,2 @@\n+x\n+y\n"
                "@@ -9 +11 @@\n-z\n+w\n--- a/std/gone.cp\n+++ /dev/null\n@@ -1 +0,0 @@\n-q\n")
        self.assertEqual(sm.parse_diff(diff), {"a.cp": {4, 5, 11}})

    def test_shards_cover_every_mutant_once(self) -> None:
        ms = list(range(10))
        self.assertEqual(sorted(x for k in range(3) for x in sm.shard(ms, f"{k}/3")), ms)
        self.assertEqual(sm.shard(ms, None), ms)

    def test_cpl_test_names_the_file_as_std_reads_it(self) -> None:
        self.assertEqual(sm.test_argv("cpl", "algebra.cp"), ["cpl", "test", "std/algebra.cp"])

    def test_cpl_test_output(self) -> None:
        out = "  ok   std/a.cp::one\n  FAIL /t/a.cp::two: assertion failed\n1 passed, 1 failed\n"
        self.assertEqual(sm.test_cases(out, 1), {"one": "pass", "two": "fail"})
        self.assertEqual(sm.test_cases("error[E0101]: no such name", 1), {"*": "error"})
        self.assertEqual(sm.test_cases("", 0), {})

    def test_a_fn_cut_by_the_time_limit_times_out(self) -> None:
        out = "  FAIL std/bag.cp::big: still running after the 2000 ms time limit\n0 passed, 1 failed\n"
        self.assertEqual(sm.test_cases(out, 1), {"big": "timeout"})


class FakeRunner(sm.Runner):
    def __init__(self, clean: dict, mutant: dict) -> None:
        super().__init__("conform", "cpl")
        self.results = {"clean": clean, "mutant": mutant}

    def slices(self, m):
        return list(self.results["clean"])

    def run(self, sl, std, limit):
        return self.results["clean" if std == sm.STD else "mutant"].get(sl)


class OutcomeTest(unittest.TestCase):
    def outcome(self, clean, mutant) -> str:
        return FakeRunner(clean, mutant).outcome({}, Path("/scratch"))

    def test_a_case_green_on_clean_std_that_fails_catches(self) -> None:
        clean = {("a",): {"x": "pass"}, ("b",): {"y": "pass"}}
        self.assertEqual(self.outcome(clean, {("a",): {"x": "pass"}, ("b",): {"y": "fail"}}), "CaughtMutant")
        self.assertEqual(self.outcome(clean, {("a",): {"x": "pass"}, ("b",): {}}), "CaughtMutant")
        self.assertEqual(self.outcome(clean, {("a",): None, ("b",): {"y": "pass"}}), "CaughtMutant")

    def test_a_spec_case_red_on_clean_std_catches_nothing(self) -> None:
        clean = {("a",): {"x": "fail"}, ("b",): {"y": "pass"}}
        self.assertEqual(self.outcome(clean, {("a",): {"x": "fail"}, ("b",): {"y": "pass"}}), "MissedMutant")

    def test_a_red_std_test_or_a_slow_or_crashed_slice_stops_the_run(self) -> None:
        green = {"y": "pass"}
        for sl, red in ((("test", "a.cp"), {"x": "fail"}), (("a",), {"x": "timeout"}),
                        (("a",), {"*": "timeout"}), (("a",), None)):
            with self.subTest(red=red), self.assertRaises(sm.BaselineRed):
                self.outcome({sl: red, ("b",): green}, {sl: red, ("b",): green})

    def test_compile_errors_alone_are_unviable(self) -> None:
        clean = {("a",): {"x": "pass"}, ("b",): {"y": "pass"}}
        self.assertEqual(self.outcome(clean, {("a",): {"*": "error"}, ("b",): {"y": "error"}}), "Unviable")
        self.assertEqual(self.outcome(clean, {("a",): {"x": "error"}, ("b",): {"y": "fail"}}), "CaughtMutant")

    def test_a_hang_times_out(self) -> None:
        clean = {("a",): {"x": "pass"}, ("b",): {"y": "pass"}}
        self.assertEqual(self.outcome(clean, {("a",): {"*": "timeout"}, ("b",): {"y": "pass"}}), "Timeout")
        self.assertEqual(self.outcome(clean, {("a",): {"x": "timeout"}, ("b",): {"y": "pass"}}), "Timeout")
        self.assertEqual(self.outcome(clean, {("a",): {"x": "timeout"}, ("b",): {"y": "fail"}}), "CaughtMutant")


if __name__ == "__main__":
    unittest.main()
