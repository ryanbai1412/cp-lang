import subprocess
import contextlib
import io
import sys
import unittest
from unittest import mock
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
                self.assertTrue(step.fn or step.argv, step.title)
                if step.fn or "/" not in step.argv[0]:
                    continue
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

    def test_touched_runs_only_the_named_scripts_steps_once(self) -> None:
        seen = []
        status = ci.run_steps("touched", only={"scripts/debt-ratchet.py", "bench/sizes.py"},
                              run=lambda cmd, **kw: seen.append(cmd) or subprocess.CompletedProcess(cmd, 0, "", ""))
        self.assertEqual(status, 0)
        ran = [str(Path(c[1]).relative_to(ci.ROOT)) for c in seen]
        self.assertEqual(sorted(set(ran)), ["bench/sizes.py", "scripts/debt-ratchet.py"])
        self.assertEqual(len(ran), len(ci.touched_steps({"scripts/debt-ratchet.py", "bench/sizes.py"})))
        self.assertFalse(any(s in ci.STEPS["full"] and s not in ci.STEPS["checks"]
                             for s in ci.touched_steps(ci.step_scripts())))

    def test_full_runs_commands_scripts_and_fns_with_their_placeholders(self) -> None:
        seen, fns = [], []
        steps = (ci.Step("cmd", ("cargo", "test", "--jobs", "{nproc}"), "rerun"),
                 ci.Step("sh", ("scripts/site-check.sh", "--out", "{tmp}/w"), "rerun", env=(("CPL", "{cpl}"),)),
                 ci.Step("py", ("scripts/gen-programs.py",), "rerun"),
                 ci.Step("fn", (), "rerun", fn=lambda run, cpl: fns.append(cpl) or 0))
        with mock.patch.dict(ci.STEPS, {"full": steps}):
            status = ci.run_steps("full", cpl="/x/cpl",
                                  run=lambda cmd, **kw: seen.append((cmd, kw)) or subprocess.CompletedProcess(cmd, 0))
        self.assertEqual(status, 0)
        (cargo, _), (sh, sh_kw), (py, _) = seen
        self.assertEqual(cargo[:3], ["cargo", "test", "--jobs"])
        self.assertTrue(cargo[3].isdigit())
        self.assertEqual(sh[0], str(ci.ROOT / "scripts/site-check.sh"))
        self.assertTrue(sh[2].endswith("/w") and "{tmp}" not in sh[2])
        self.assertFalse(Path(sh[2]).parent.exists(), "the tier removes its {tmp}")
        self.assertEqual(sh_kw["env"]["CPL"], "/x/cpl")
        self.assertEqual(py[:2], [sys.executable, str(ci.ROOT / "scripts/gen-programs.py")])
        self.assertEqual(fns, ["/x/cpl"])

    def test_a_missing_command_fails_before_the_step_runs(self) -> None:
        seen = []
        step = ci.Step("needs", ("x",), "rerun", needs=("no-such-command-cpl",))
        with mock.patch.dict(ci.STEPS, {"full": (step,)}), contextlib.redirect_stderr(io.StringIO()) as err:
            status = ci.run_steps("full", run=lambda cmd, **kw: seen.append(cmd))
        self.assertEqual((status, seen), (1, []))
        self.assertIn("no-such-command-cpl` is not installed", err.getvalue())

    def test_vscode_runs_the_host_suite_only_with_a_display(self) -> None:
        for display, xvfb, host in (("1", None, ["npm", "run", "test:integration"]),
                                    ("", "/bin/xvfb-run", ["xvfb-run", "-a", "npm", "run", "test:integration"]),
                                    ("", None, None)):
            seen = []
            with mock.patch.dict(ci.os.environ, {"DISPLAY": display}), \
                    mock.patch.object(ci.shutil, "which", return_value=xvfb), contextlib.redirect_stdout(io.StringIO()):
                ci.vscode_tests(run=lambda cmd, **kw: seen.append(cmd) or subprocess.CompletedProcess(cmd, 0))
            self.assertIn(["npm", "test"], seen)
            self.assertEqual(seen[-1] if host else None, host)
            self.assertNotIn("test:integration", seen[-1] if not host else [])

    def test_check_sh_runs_no_step_itself(self) -> None:
        check = "\n".join(l for l in (Path(__file__).parent / "check.sh").read_text(encoding="utf-8").splitlines()
                          if not l.lstrip().startswith("#")) + "\n"
        self.assertNotIn("spec-gate.py ", check)
        for tier in ci.STEPS.values():
            for step in tier:
                if step.argv[:1] and step.argv[0].endswith(".py"):
                    self.assertNotIn(f"python {step.argv[0]}", check)
                elif step.argv:
                    self.assertNotIn(" ".join(step.argv), check)
        self.assertNotIn("npm test\n", check)
        for tier in ci.STEPS:
            self.assertIn(f"scripts/ci.py steps {tier}", check)


if __name__ == "__main__":
    unittest.main()
