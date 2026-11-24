import subprocess
import contextlib
import io
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import ci  # noqa: E402


class GatePartTest(unittest.TestCase):
    def test_checks_and_unset(self) -> None:
        for part in ("", "checks"):
            self.assertEqual(ci.gate_part(part, "commit")["kind"], "checks")

    def test_conform_suites_and_shard(self) -> None:
        self.assertEqual(ci.gate_part("conform", "fast"),
                         {"kind": "conform", "suites": "all", "shard": "", "partition": ""})
        g = ci.gate_part("conform:conformance+lints,std@2/3", "fast")
        self.assertEqual((g["suites"], g["shard"]), ("conformance lints std", "--shard 2/3"))

    def test_tests_partitions(self) -> None:
        self.assertEqual(ci.gate_part("tests:doc", "fast")["kind"], "doc")
        g = ci.gate_part("tests:hash:1/12", "fast")
        self.assertEqual((g["kind"], g["partition"]), ("tests", "hash:1/12"))

    def test_parts_need_fast(self) -> None:
        for part in ("fmt", "conform", "conform:all", "tests:doc", "tests:hash:1/2"):
            with self.assertRaisesRegex(ValueError, "needs --fast"):
                ci.gate_part(part, "land")

    def test_unknown_and_malformed(self) -> None:
        for part in ("bogus", "tests:", "conform:all@"):
            with self.assertRaises(ValueError):
                ci.gate_part(part, "fast")


class FilterTest(unittest.TestCase):
    def test_blocking_tiers_skip_every_wall_clock_test(self) -> None:
        slow, wall = ci.slow_filter(), ci.wall_filter()
        for name in ci.WALL_CLOCK_TESTS:
            clause = f"test(/(^|::){name}$/)"
            self.assertIn(clause, slow)
            self.assertIn(clause, wall)
        for name in ci.SLOW_TESTS:
            self.assertIn(f"test(/(^|::){name}$/)", slow)
        self.assertTrue(slow.startswith("binary(budget)"))

    def test_plan_round_trips_through_the_shell(self) -> None:
        out = subprocess.run(
            ["bash", "-c", 'eval "$(python3 scripts/ci.py plan --mode fast --gate-part conform:a+b@1/2)"; '
             'printf "%s|%s|%s\\n" "$gate_kind" "$conform_suites" "$conform_shard"; printf "%s" "$slow_filter"'],
            cwd=Path(__file__).parent.parent, capture_output=True, text=True, check=True).stdout
        first, slow = out.split("\n", 1)
        self.assertEqual(first, "conform|a b|--shard 1/2")
        self.assertEqual(slow, ci.slow_filter())

    def test_plan_rejects_a_bad_part(self) -> None:
        self.assertEqual(ci.main(["plan", "--mode", "fast", "--gate-part", "bogus"]), 1)



class StepsTest(unittest.TestCase):
    @staticmethod
    def _name(cmd):
        script = Path(cmd[1])
        return cmd[2] if script == ci.SPEC_GATE else script.stem

    def _run(self, codes, healed=False):
        calls = []

        def run(cmd, **kw):
            calls.append(self._name(cmd))
            return subprocess.CompletedProcess(cmd, codes.get(self._name(cmd), 0), "out\n", "")
        return ci.run_steps("checks", run=run, healed=healed), calls

    def test_every_step_runs_a_script_that_exists(self) -> None:
        import importlib.util
        spec = importlib.util.spec_from_file_location("spec_gate", ci.SPEC_GATE)
        sg = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(sg)
        for tier in ci.STEPS.values():
            for step in tier:
                self.assertTrue((ci.ROOT / step.argv[0]).is_file(), step.argv[0])
                if step.argv[0] == "scripts/spec-gate.py":
                    self.assertIn(step.argv[1], sg.GATES)

    def test_every_failure_says_how_to_fix_it(self) -> None:
        from test_check_cache import FIX_HINT
        for tier in ci.STEPS.values():
            for step in tier:
                self.assertRegex(step.fail, FIX_HINT, "say the command to rerun or the fix (ci/R31)")

    def test_stops_at_the_first_failure(self) -> None:
        status, calls = self._run({"matrix": 1})
        self.assertEqual(status, 1)
        self.assertEqual(calls[-1], "matrix")
        self.assertNotIn("widths", calls)

    def test_a_derived_step_only_warns_once_healed(self) -> None:
        for name in ("coverage", "skill-gen"):
            self.assertEqual(self._run({name: 1})[0], 1)
            status, calls = self._run({name: 1}, healed=True)
            self.assertEqual(status, 0)
            self.assertEqual(len(calls), len(ci.STEPS["checks"]))

    def test_the_cpl_binary_is_passed_through(self) -> None:
        seen = []
        ci.run_steps("checks", cpl="/x/cpl", run=lambda cmd, **kw: seen.append(cmd) or subprocess.CompletedProcess(cmd, 0, "", ""))
        self.assertIn(["--cpl", "/x/cpl"], [c[-2:] for c in seen])
        self.assertFalse(any("{cpl}" in a for c in seen for a in c))

    def test_dropped_lines_stay_hidden(self) -> None:
        sizes = next(s for s in ci.STEPS["checks"] if s.drop)
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            ci.run_steps("checks", run=lambda cmd, **kw: subprocess.CompletedProcess(
                cmd, 0, f"a.cpp: warning: big\nok {sizes.argv[0]}\n" if Path(cmd[1]).name == "sizes.py" else "", ""))
        self.assertNotIn("warning", out.getvalue())
        self.assertIn("ok bench/sizes.py", out.getvalue())

    def test_check_sh_runs_no_step_itself(self) -> None:
        check = (Path(__file__).parent / "check.sh").read_text(encoding="utf-8")
        self.assertNotIn("spec-gate.py ", check)
        for tier in ci.STEPS.values():
            for step in tier:
                self.assertNotIn(f"python {step.argv[0]}", check)
        for tier in ci.STEPS:
            self.assertIn(f"scripts/ci.py steps {tier}", check)


if __name__ == "__main__":
    unittest.main()
