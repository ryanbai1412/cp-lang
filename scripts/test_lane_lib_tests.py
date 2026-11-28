"""Tests for scripts/lane-lib-tests.py's reading of a run and its verdict."""

import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "lane-lib-tests.py"
SPEC = importlib.util.spec_from_file_location("lane_lib_tests", SCRIPT)
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class LaneLibTests(unittest.TestCase):
    def test_a_run_names_its_failed_tests_only(self):
        out = ("test tests::a ... ok\ntest tests::b ... FAILED\n---- tests::b stdout ----\n"
               "test tests::c ... FAILED\nfailures:\n    tests::b\n")
        self.assertEqual(MOD.reds_of(out), {"tests::b", "tests::c"})

    def test_a_quiet_run_names_its_reds_by_their_stdout_blocks(self) -> None:
        out = ("running 3 tests\n.F.\nfailures:\n\n---- tests::b stdout ----\npanicked\n\n"
               "failures:\n    tests::b\n\ntest result: FAILED. 2 passed; 1 failed\n")
        self.assertEqual(MOD.reds_of(out), {"tests::b"})

    def test_a_run_of_many_test_binaries_names_the_reds_of_each(self) -> None:
        out = ("running 2 tests\n.F\nfailures:\n\n---- tests::b stdout ----\npanicked\n\n"
               "test result: FAILED. 1 passed; 1 failed\n\nrunning 1 test\nF\nfailures:\n\n"
               "---- std_unchecked_is_what_checking_every_std_fn_finds stdout ----\npanicked\n")
        self.assertEqual(MOD.reds_of(out), {"tests::b", "std_unchecked_is_what_checking_every_std_fn_finds"})

    def test_the_gate_runs_every_cpl_next_test_binary_but_the_lists_test(self) -> None:
        cmd = MOD.command("cpl-next")
        self.assertIn("--tests", cmd)
        self.assertIn("--lib", cmd)
        tail = cmd[cmd.index("--"):]
        self.assertIn("--exact", tail)
        self.assertEqual([t for a, t in zip(tail, tail[1:]) if a == "--skip"], ["every_case_is_where_the_lists_say"])
        self.assertTrue((MOD.ROOT / "crates/cpl-next/tests/std_modules.rs").is_file())
        self.assertIn("--lib", MOD.command("cpl-doc"))

    def test_the_list_reads_rows_and_skips_comments(self):
        text = "# header\ncpl-next tests::b  # H, abc\n\ncpl-doc tests::d\n"
        self.assertEqual(MOD.listed(text), {("cpl-next", "tests::b"), ("cpl-doc", "tests::d")})

    def test_a_row_without_its_crate_is_refused(self):
        with self.assertRaises(SystemExit):
            MOD.listed("tests::b\n")

    def test_an_unlisted_red_refuses(self):
        why = MOD.verdict({("cpl-next", "tests::b")}, set(), {"cpl-next"})
        self.assertEqual(len(why), 1)
        self.assertIn("NEW red", why[0])

    def test_a_listed_red_passes(self):
        self.assertEqual(MOD.verdict({("cpl-next", "tests::b")}, {("cpl-next", "tests::b")}, {"cpl-next"}), [])

    def test_a_listed_test_that_passes_refuses_until_its_row_goes(self):
        why = MOD.verdict(set(), {("cpl-doc", "tests::d")}, {"cpl-next", "cpl-doc"})
        self.assertEqual(len(why), 1)
        self.assertIn("drop its row", why[0])


if __name__ == "__main__":
    unittest.main()
