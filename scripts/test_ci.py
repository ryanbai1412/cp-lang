import subprocess
import contextlib
import io
import json
import os
import sys
import tempfile
import unittest
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import ci  # noqa: E402

PKGS = "corpus/problems"


class GatePartTest(unittest.TestCase):
    def test_checks_and_unset(self) -> None:
        for part in ("", "checks"):
            self.assertEqual(ci.gate_part(part, "commit")["kind"], "checks")

    def test_conform_suites_and_shard(self) -> None:
        self.assertEqual(ci.gate_part("conform", "fast"),
                         {"kind": "conform", "suites": "all", "shard": "", "partition": ""})
        g = ci.gate_part("conform:conformance+lints,std@2/3", "fast")
        self.assertEqual((g["suites"], g["shard"]), ("conformance lints std", "--shard 2/3"))

    def test_fmt_and_corpus(self) -> None:
        self.assertEqual([ci.gate_part(p, "fast")["kind"] for p in ("fmt", "corpus")], ["fmt", "corpus"])

    def test_tests_partitions(self) -> None:
        self.assertEqual(ci.gate_part("tests:doc", "fast")["kind"], "doc")
        g = ci.gate_part("tests:hash:1/12", "fast")
        self.assertEqual((g["kind"], g["partition"]), ("tests", "hash:1/12"))

    def test_parts_need_fast(self) -> None:
        for part in ("fmt", "corpus", "conform", "conform:all", "tests:doc", "tests:hash:1/2"):
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

    def test_every_spec_gate_runs_somewhere(self) -> None:
        import importlib.util
        spec = importlib.util.spec_from_file_location("spec_gate", ci.SPEC_GATE)
        sg = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(sg)
        stepped = {s.argv[1] for tier in ci.STEPS.values() for s in tier
                   if s.argv and s.argv[0] == "scripts/spec-gate.py"}
        elsewhere = "".join(p.read_text() for p in [ci.ROOT / "gen.toml", ci.ROOT / "scripts/land.sh",
                                                     *(ci.ROOT / ".github/workflows").glob("*.yml")])
        for g in sg.GATES:
            if g not in stepped:
                self.assertRegex(elsewhere, rf'spec-gate\.py"?,? "?{g}\b|just spec-gate {g}\b|"{g}"\]', f"spec gate {g} is never run")

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
        for name in ("skill-gen",):
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
        # Every tier after the guards runs in `ci.py gate`; check.sh runs no cargo itself.
        self.assertIn("scripts/ci.py check", check)
        self.assertNotRegex(check, r"(?m)^\s*cargo ")
        for owned in ("cpl-conform\" all", "--test recovery", "known_traps.cp"):
            self.assertNotIn(owned, check)


class FakeProc:
    def __init__(self, code: int, running: bool = False):
        self.code, self.running, self.killed = code, running, False

    def wait(self):
        return self.code

    def poll(self):
        return None if self.running and not self.killed else self.code

    def kill(self):
        self.killed = True


class TailTest(unittest.TestCase):
    FMT = "cargo fmt --all --check"
    CLIPPY = "cargo clippy --workspace --all-targets -- -D warnings"
    BUILD = "cargo build -q -p cpl-cli -p cpl-conform"

    def _tail(self, mode="fast", part="", cached=(), codes=None, procs=None, manifest="m", since=None, sel=None,
              outs=None, **kw):
        events, codes, procs, outs = [], codes or {}, procs or {}, outs or {}

        def label(argv):
            argv = [str(a) for a in argv]
            if argv[0] == sys.executable:
                argv = argv[1:]
            return " ".join(a.replace(str(ci.ROOT) + "/", "") for a in argv)

        def run(argv, **_):
            name = label(argv)
            if name.startswith("scripts/check_cache.py"):
                verb, stage = name.split()[1], name.split()[3]
                events.append(f"{verb} {stage}")
                return subprocess.CompletedProcess(argv, 0 if verb == "save" or stage in cached else 1, "", "")
            events.append(name)
            out = "scripts/test_ci.py\n" if name.startswith("git ls-files") else ""
            out = next((o for k, o in outs.items() if name.startswith(k)), out)
            code = codes.get(name, codes.get(name.split()[0], 0))
            return subprocess.CompletedProcess(argv, code, out, "")

        def popen(argv, stdout=None, **_):
            name = label(argv)
            key = next((k for k in ("fmt_check", "unittest", "cpl-conform", "recovery", "gate.mjs") if k in name), name)
            events.append(f"spawn {key}")
            stdout.write(f"{key} log\n")
            proc = procs.get(key, FakeProc(0))
            spawned.append(proc)
            return proc
        spawned: list = []
        tail = ci.Tail(mode, part, cache_dir="d", manifest=manifest, run=run, popen=popen, which=lambda c: f"/bin/{c}", **kw)
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = tail.commit(sel) if sel is not None else tail() if since is None else tail.part(since)
        return code, events, out.getvalue(), err.getvalue(), tail, spawned

    def test_fast_overlaps_the_tests_and_saves_each_stage_after_it_passes(self) -> None:
        code, ev, out, _, tail, _ = self._tail()
        self.assertEqual(code, 0)
        spawns = [e for e in ev if e.startswith("spawn")]
        self.assertEqual(spawns, ["spawn fmt_check", "spawn unittest", "spawn cpl-conform", "spawn recovery"])
        nextest = ev.index("cargo nextest run --workspace --no-fail-fast -E " + f"not ({ci.slow_filter()})")
        self.assertLess(ev.index("spawn cpl-conform"), nextest)
        self.assertLess(nextest, ev.index("spawn recovery"))
        saves = [e.split()[1] for e in ev if e.startswith("save")]
        self.assertEqual(saves, ["clippy", "nextest_doc", "fmt_corpus", "site_gate", "scoreboard_fast", "recovery"])
        self.assertIn("fmt_check log", out)
        self.assertIn("known_traps:", out)
        self.assertIn("cpl fmt is idempotent and keeps meaning on every .cp file", [t for _, t in tail.times])

    def test_a_cached_stage_runs_nothing(self) -> None:
        stages = ("clippy", "fmt_corpus", "scoreboard_fast", "nextest_doc", "recovery", "site_gate")
        code, ev, out, _, _, _ = self._tail(cached=stages)
        self.assertEqual(code, 0)
        self.assertFalse([e for e in ev if e.startswith(("spawn", "save"))])
        self.assertEqual([e for e in ev if e.startswith("cargo")], [self.FMT, self.BUILD])
        self.assertEqual(out.count(ci.CACHED), 6)

    def test_no_manifest_runs_every_stage(self) -> None:
        code, ev, _, _, _, _ = self._tail(manifest="", cached=("fmt_corpus",))
        self.assertEqual(code, 0)
        self.assertFalse([e for e in ev if e.startswith(("check ", "save "))])
        self.assertIn("spawn fmt_check", ev)

    def test_the_checks_part_runs_no_tests_and_overlaps_the_site_gate(self) -> None:
        code, ev, out, _, _, _ = self._tail(part="checks")
        self.assertEqual(code, 0)
        self.assertEqual([e for e in ev if e.startswith("spawn")], ["spawn gate.mjs"])
        self.assertEqual([e for e in ev if e.startswith("cargo")], [self.FMT, self.CLIPPY, self.BUILD])
        self.assertEqual(out.count("CPL_GATE_PART=checks"), 5)
        self.assertFalse([e for e in ev if e.startswith("scripts/corpus.py")])

    def test_a_failed_job_names_its_rerun_and_stops_the_rest(self) -> None:
        board = FakeProc(0, running=True)
        code, ev, _, err, _, spawned = self._tail(procs={"fmt_check": FakeProc(1), "cpl-conform": board})
        self.assertEqual(code, 1)
        self.assertIn("rerun: scripts/fmt_check.py", err)
        self.assertTrue(board.killed)
        self.assertNotIn("save fmt_corpus", ev)

    def test_a_failed_foreground_step_stops_the_tier(self) -> None:
        code, ev, _, err, _, _ = self._tail(codes={"scripts/corpus.py": 1})
        self.assertEqual(code, 1)
        self.assertIn("scripts/corpus.py verify", err)
        self.assertFalse([e for e in ev if e.startswith("scripts/spec-gate.py")])

    def test_a_land_train_runs_only_what_it_reaches(self) -> None:
        code, ev, out, _, _, _ = self._tail(mode="land")
        self.assertEqual(code, 0)
        self.assertFalse([e for e in ev if e.startswith("cargo nextest") or e == "spawn gate.mjs"])
        self.assertIn("skipped: the train touches no site, docs or crate path", out)
        code, ev, _, _, _, _ = self._tail(mode="land", sel_packages="cpl-a cpl-b", sel_filter="rdeps(cpl-a)")
        self.assertIn(f"cargo nextest run --workspace --no-fail-fast -E (rdeps(cpl-a)) & not ({ci.slow_filter()})", ev)
        self.assertIn("cargo test -q -p cpl-a -p cpl-b --doc", ev)
        self.assertNotIn("save nextest_doc", ev)

    def test_a_land_train_runs_its_slices_first_and_refuses_a_slow_one(self) -> None:
        code, ev, _, _, _, _ = self._tail(mode="land", sel_packages="cpl-a", sel_filter="rdeps(cpl-a)",
                                          sel_slices="vm-trace copies", sel_slice_filter="test(=s)")
        self.assertEqual(code, 0)
        self.assertLess(ev.index("scripts/affected.py --build-slices vm-trace copies"),
                        ev.index("scripts/affected.py --run-slices vm-trace copies"))
        slices = ev.index("scripts/affected.py --run-slices vm-trace copies")
        train = ev.index(f"cargo nextest run --workspace --no-fail-fast -E (rdeps(cpl-a)) & not ({ci.slow_filter()})"
                         " & not (test(=s))")
        self.assertLess(slices, train)
        code, ev, _, err, _, _ = self._tail(mode="land", sel_slices="vm-trace", codes={"scripts/affected.py --run-slices vm-trace": 1})
        self.assertEqual(code, 1)
        self.assertIn("took over 20 s", err)
        self.assertFalse([e for e in ev if e.startswith("cargo nextest")])

    def test_full_runs_the_full_steps_after_the_rest(self) -> None:
        code, ev, out, _, _, _ = self._tail(mode="full", manifest="")
        self.assertEqual(code, 0)
        self.assertIn("the --full suites (scripts/ci.py steps full)", out)

    def test_an_interrupt_stops_the_backgrounded_jobs(self) -> None:
        board = FakeProc(0, running=True)

        def interrupted(*a, **kw):
            raise KeyboardInterrupt
        with mock.patch.object(ci, "run_steps", interrupted), self.assertRaises(KeyboardInterrupt):
            self._tail(procs={"cpl-conform": board})
        self.assertTrue(board.killed)
        self.assertIs(ci.signal.getsignal(ci.signal.SIGTERM), ci._interrupt)
        ci.signal.signal(ci.signal.SIGTERM, ci.signal.SIG_DFL)

    def _part(self, part, env=None, **kw):
        env = {"CPL_CONFORM_PREV": "", "CPL_NEXTEST_ARCHIVE": "", **(env or {})}
        with mock.patch.dict(ci.os.environ, env):
            for k in [k for k, v in env.items() if not v]:
                ci.os.environ.pop(k, None)
            return self._tail(part=part, since=0, **kw)

    def test_the_whole_conform_part_runs_the_script_tests_and_the_corpus_cases_first(self) -> None:
        code, ev, out, _, _, _ = self._part("conform")
        self.assertEqual(code, 0)
        conform = next(e for e in ev if e.split()[0].endswith("cpl-conform"))
        self.assertTrue(conform.endswith(" all --check"), conform)
        make = next(e for e in ev if e.startswith("scripts/corpus.py make") or " scripts/corpus.py make" in e)
        order = [ev.index(e) for e in ("-m unittest scripts/test_ci.py", "cargo build -q -p cpl-cli -p cpl-conform",
                                         make, conform)]
        self.assertEqual(order, sorted(order))
        self.assertIn("check: conformance part passed in", out)

    def test_a_conform_shard_runs_only_its_suites(self) -> None:
        code, ev, _, _, _, _ = self._part("conform:diff@1/3", env={"CPL_CONFORM_PREV": "/p.json"})
        self.assertEqual(code, 0)
        self.assertFalse([e for e in ev if "unittest" in e or "corpus.py make" in e])
        conform = next(e for e in ev if e.split()[0].endswith("cpl-conform"))
        self.assertTrue(conform.endswith(" diff --shard 1/3 --check --prev /p.json --save /p.json.new"), conform)

    def test_a_failed_conform_ratchet_names_its_rerun(self) -> None:
        code, _, _, err, _, _ = self._part("conform:corpus", codes={str(ci.target_dir() / "debug/cpl-conform").replace(str(ci.ROOT) + "/", ""): 1})
        self.assertEqual(code, 1)
        self.assertIn("rerun: cargo run -p cpl-conform -- corpus  --check", err)

    def test_a_test_partition_runs_from_the_archive_once_with_no_retries(self) -> None:
        code, ev, out, _, _, _ = self._part("tests:hash:3/12", env={"CPL_NEXTEST_ARCHIVE": "a.tar.zst"})
        self.assertEqual(code, 0)
        run = next(e for e in ev if e.startswith("cargo nextest run"))
        self.assertIn("--archive-file a.tar.zst --workspace-remap . --extract-to . --extract-overwrite", run)
        self.assertIn("--profile gate --no-fail-fast --partition hash:3/12", run)
        self.assertNotIn("--retries", run)
        self.assertIn("check: test partition hash:3/12 passed in", out)
        code, ev, _, err, _, _ = self._part("tests:hash:1/12", codes={"cargo": 1})
        self.assertEqual(code, 1)
        self.assertIn("--workspace --profile gate", next(e for e in ev if e.startswith("cargo nextest run")))
        self.assertIn("nextest's summary above names it", err)

    def test_the_doc_and_fmt_parts(self) -> None:
        code, ev, out, _, _, _ = self._part("tests:doc")
        self.assertEqual((code, [e for e in ev if e.startswith("cargo")]), (0, ["cargo test -q --workspace --doc"]))
        code, ev, out, _, _, _ = self._part("fmt", cached=("fmt_corpus",))
        self.assertEqual(code, 0)
        self.assertFalse([e for e in ev if e.startswith(("cargo", "scripts/fmt_check.py", "save"))])
        self.assertIn(ci.CACHED, out)
        code, ev, out, _, _, _ = self._part("fmt")
        self.assertEqual(ev[-3:], ["cargo build -q -p cpl-cli", "scripts/fmt_check.py --cpl target/debug/cpl", "save fmt_corpus"])
        self.assertIn("check: fmt part passed in", out)

    def test_the_corpus_part_runs_the_static_modes_alone(self) -> None:
        code, ev, out, _, _, _ = self._part("corpus")
        self.assertEqual(code, 0)
        self.assertEqual(ev, ["cargo build -q -p cpl-cli", "scripts/corpus.py verify --no-run --only header,check,package,lint --cpl target/debug/cpl"])
        self.assertIn("check: corpus part passed in", out)
        code, _, _, err, _, _ = self._part("corpus", codes={"scripts/corpus.py": 1})
        self.assertEqual(code, 1)
        self.assertIn("names the mode and the package", err)

    def test_the_full_part_runs_only_the_full_suites(self) -> None:
        self.assertEqual(ci.gate_part("full", "full")["kind"], "full")
        for part, mode in (("full", "fast"), ("conform", "full")):
            with self.assertRaisesRegex(ValueError, "only `full` goes with --full"):
                ci.gate_part(part, mode)
        with mock.patch.object(ci, "run_steps", return_value=0) as steps:
            code, ev, out, _, _, _ = self._part("full", mode="full")
        self.assertEqual((code, ev), (0, ["cargo build -q -p cpl-cli -p cpl-conform"]))
        self.assertEqual(steps.call_args.args[0], "full")
        self.assertIn("check: full part passed in", out)
        with mock.patch.object(ci, "run_steps", return_value=1):
            code, _, _, err, _, _ = self._part("full", mode="full")
        self.assertEqual(code, 1)
        self.assertIn("a --full suite failed", err)
        self.assertTrue(steps.call_args.kwargs["keep_going"])

    def test_keep_going_runs_every_step_and_names_each_failure(self) -> None:
        calls = []

        def run(cmd, **kw):
            line = " ".join(map(str, cmd))
            calls.append(line)
            return subprocess.CompletedProcess(cmd, int(" metamorphic" in line or " flakes" in line), "", "")
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.object(sys, "stdout", out), mock.patch.object(sys, "stderr", err), \
                mock.patch.object(ci.shutil, "which", return_value="/bin/x"):
            self.assertEqual(ci.run_steps("full", run=run, keep_going=True), 1)
        for step in ci.STEPS["full"]:
            self.assertIn(f"==> {step.title}", out.getvalue())
        self.assertIn("check: failed: spec-gate.py metamorphic", err.getvalue())
        self.assertIn("check: failed: spec-gate.py flakes --runs 2", err.getvalue())

    def test_a_part_needs_fast_and_checks_is_not_a_part(self) -> None:
        self.assertEqual(self._part("fmt", mode="land")[0], 1)
        self.assertEqual(self._part("checks")[0], 1)

    def test_counts_and_the_bare_home(self) -> None:
        self.assertEqual(ci.prelude_names("use a.{x, y} // {z}\nuse b.{\n  w,\n}\n"), 3)
        self.assertEqual(ci.open_traps("if which == 1:\n  if which == 2:\nif which == 3:\n"), 2)
        env = ci.bare_home("/h", {"HOME": "/real", "GIT_DIR": "/r/.git", "CARGO_HOME": "/c", "PATH": "/bin"})
        self.assertEqual((env["HOME"], env["CARGO_HOME"], env["RUSTUP_HOME"], env["PATH"]), ("/h", "/c", "/real/.rustup", "/bin"))
        self.assertNotIn("GIT_DIR", env)
        self.assertEqual(env["GIT_CONFIG_NOSYSTEM"], "1")


class CommitTest(unittest.TestCase):
    """The commit tier (`ci.py commit`) runs only what scripts/affected.py selected."""
    TREE = "cargo tree -q -p cpl-wasm"
    _tail = TailTest._tail

    def _commit(self, sel: dict[str, str], **kw):
        self.enterContext(mock.patch.dict(ci.os.environ))
        ci.os.environ.pop("CI", None)
        with mock.patch.object(ci, "run_steps", side_effect=lambda tier, *a, **k: self.tiers.append((tier, k.get("only"))) or 0):
            self.tiers = []
            return self._tail(mode="commit", sel=sel, **kw)

    def test_an_empty_selection_runs_the_commit_steps_and_the_staleness_note(self) -> None:
        code, ev, _, _, tail, _ = self._commit({})
        self.assertEqual(code, 0)
        self.assertEqual(self.tiers, [("commit", None)])
        self.assertEqual(ev, ["scripts/gen.py --check"])
        self.assertEqual([t for _, t in tail.times][0], "checking scripts (scripts/ci.py steps commit)")

    def test_stale_generated_files_only_note(self) -> None:
        code, _, out, _, _, _ = self._commit({}, codes={"scripts/gen.py": 1}, outs={"scripts/gen.py": "a\n3 stale\n"})
        self.assertEqual(code, 0)
        self.assertIn("3 stale\nnote: not blocking", out)

    def test_touched_crates_get_fmt_clippy_and_their_tests_and_wasm_only_when_it_reaches_them(self) -> None:
        sel = {"packages": "cpl-check cpl-cli", "filter": "rdeps(cpl-check)", "slice_filter": "test(=s)"}
        code, ev, _, _, _, _ = self._commit(sel, outs={self.TREE: "cpl-wasm v0.1.0\ncpl-check v0.1.0\n"})
        self.assertEqual(code, 0)
        ps = "-p cpl-check -p cpl-cli"
        expr = f"(rdeps(cpl-check)) & not ({ci.slow_filter()}) & not (test(=s))"
        self.assertEqual(ev[1:], [f"cargo fmt --check {ps}", f"cargo clippy -q {ps} --all-targets -- -D warnings",
                                  self.TREE + " --target wasm32-unknown-unknown -e normal --prefix none",
                                  "rustc --print target-libdir --target wasm32-unknown-unknown",
                                  f"cargo nextest run --workspace --profile commit --no-fail-fast -E {expr}"])
        code, ev, _, _, _, _ = self._commit(sel, outs={self.TREE: "cpl-wasm v0.1.0\n"})
        self.assertFalse([e for e in ev if e.startswith("rustc")])

    def test_a_touched_wasm_crate_holds_the_data_segment_to_its_budget(self) -> None:
        sel = {"packages": "cpl-check", "filter": "rdeps(cpl-check)"}
        outs = {self.TREE: "cpl-check v0.1.0\n", "rustc --print target-libdir": f"{ci.ROOT}\n"}
        code, ev, _, _, _, _ = self._commit(sel, outs=outs)
        self.assertEqual(code, 0)
        gate = [e for e in ev if e.startswith(("cargo check -q -p cpl-wasm", "scripts/build-wasm.sh", "node web/scripts/wasm-crates.mjs"))]
        self.assertEqual([g.split()[0] for g in gate], ["cargo", "scripts/build-wasm.sh", "node"])
        self.assertTrue(gate[2].endswith("wasm-data-gate/cpl.wasm --touched cpl_check"))
        code, _, _, err, _, _ = self._commit(sel, outs=outs, codes={"node": 1})
        self.assertEqual(code, 1)
        self.assertIn("cpl.wasm's data segment or a touched crate is over its budget", err)

    def test_a_failed_check_names_its_rerun_and_stops_the_tier(self) -> None:
        sel = {"packages": "cpl-fmt", "filter": "x", "site": "1"}
        code, ev, _, err, _, _ = self._commit(sel, codes={"cargo fmt --check -p cpl-fmt": 1})
        self.assertEqual(code, 1)
        self.assertIn("check: Rust files are not formatted; run: cargo fmt -p cpl-fmt", err)
        self.assertFalse([e for e in ev if "clippy" in e or "gate.mjs" in e])

    def test_every_selected_check_runs_in_order(self) -> None:
        sel = {"slices": "a b", "data_filter": "test(=d)", "check_files": "scripts/ci.py", "audit": "1",
               "idioms_catalog": "1", "steps": "scripts/x.py", "script_tests": "scripts/test_ci.py",
               "conform_cases": "c.cp", "site": "1"}
        dream = f"{PKGS}/p1/dream.cp\n"
        code, ev, _, _, _, _ = self._commit(sel, outs={"git diff --cached": dream})
        self.assertEqual(code, 0)
        self.assertEqual(self.tiers, [("commit", None), ("touched", {"scripts/x.py"})])
        firsts = ["scripts/affected.py --run-slices a b", "cargo nextest run --workspace --no-fail-fast -E (test(=d))",
                  "target/debug/cpl check scripts/ci.py", "scripts/corpus.py make --changed staged",
                  "scripts/corpus.py verify --only dream --cpl target/debug/cpl p1", "scripts/idioms-catalog.py --check",
                  "-m unittest scripts/test_ci.py", "scripts/conform_staged.py", "node web/scripts/gate.mjs"]
        order = [next(i for i, e in enumerate(ev) if e.startswith(f)) for f in firsts]
        self.assertEqual(order, sorted(order))
        self.assertEqual(ev.count("cargo build -q -p cpl-cli"), 2)
        # check.sh ran the header and package modes before anything built.
        self.assertIn("scripts/corpus.py verify --changed staged --skip dream,header --cpl target/debug/cpl", ev)


class GateTest(unittest.TestCase):
    """`ci.py gate`: the selection, the fallback to --fast, the parts and the closing lines."""

    class FakeTier:
        made: list = []

        def __init__(self, mode, part="", cpl="", cache_dir="", manifest="", packages="", filter="", site="", run=None,
                     sel_slices="", sel_slice_filter=""):
            self.mode, self.sel, self.slices = mode, (packages, filter, site), (sel_slices, sel_slice_filter)
            self.times = [(7, "the tests"), (0, "instant")]
            GateTest.FakeTier.made.append(self)

        def __call__(self):
            return 0

        def commit(self, sel):
            self.committed = sel
            return 0

        def part(self, since):
            self.parted = since
            return 0

    def _gate(self, mode, part="", affected="all=0\nreason=\npackages=cpl-fmt\nfilter=rdeps(cpl-fmt)\nsite=0\n",
              codes=None, base="abc", elapsed=12, env=None):
        events, codes = [], codes or {}
        GateTest.FakeTier.made = []

        def run(argv, **_):
            name = " ".join(str(a) for a in (argv[1:] if argv[0] == sys.executable else argv))
            events.append(name)
            out = base if name.startswith("git rev-parse") else affected if name.startswith("scripts/affected.py") else ""
            return subprocess.CompletedProcess(argv, codes.get(name.split()[0], 0), out, "boom")
        tmp = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.enterContext(mock.patch.object(ci, "target_dir", lambda: tmp))
        self.enterContext(mock.patch.dict(ci.os.environ, env or {}))
        ci.os.environ.pop("CHECK_BASE", None) if not (env or {}).get("CHECK_BASE") else None
        ci.os.environ.pop("CPL_NO_WALL_CLOCK", None)
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = ci.run_gate(mode, part, since=100.0, run=run, tail_cls=GateTest.FakeTier, clock=lambda: 100.0 + elapsed)
        return code, events, out.getvalue(), err.getvalue(), tmp

    def test_land_runs_the_train_selection_and_records_the_fast_time(self) -> None:
        code, ev, out, _, tmp = self._gate("land")
        self.assertEqual(code, 0)
        self.assertIn("scripts/affected.py --base abc", ev)
        [tier] = GateTest.FakeTier.made
        self.assertEqual((tier.mode, tier.sel), ("land", ("cpl-fmt", "rdeps(cpl-fmt)", "0")))
        self.assertEqual(tier.slices, ("", ""))
        self._gate("land", affected="all=0\npackages=\nslices=copies\nslice_filter=test(=c)\n")
        self.assertEqual(GateTest.FakeTier.made[0].slices, ("copies", "test(=c)"))
        self.assertIn("    7  the tests\n", out)
        self.assertNotIn("instant", out)
        self.assertIn("check: all land checks passed in 12s (budget <= 180s)", out)
        self.assertEqual(json.loads((tmp / "gate-fast-time.json").read_text()),
                         {"mode": "land", "elapsed_s": 12, "budget_s": 180})
        self.assertEqual(ci.os.environ.get("CPL_NO_WALL_CLOCK"), "1")

    def test_check_base_overrides_origin_main(self) -> None:
        _, ev, _, _, _ = self._gate("land", env={"CHECK_BASE": "feed"})
        self.assertIn("scripts/affected.py --base feed", ev)
        self.assertFalse([e for e in ev if e.startswith("git rev-parse")])

    def test_an_unmapped_path_or_no_base_runs_fast(self) -> None:
        code, _, out, _, _ = self._gate("land", affected="all=1\nreason=web/new.txt\npackages=x\n")
        self.assertEqual(code, 0)
        self.assertIn("check: web/new.txt is not mapped by scripts/affected.py; running --fast", out)
        self.assertEqual([(t.mode, t.sel) for t in GateTest.FakeTier.made], [("fast", ("", "", ""))])
        _, ev, out, _, _ = self._gate("land", base="")
        self.assertIn("no base to diff the train against", out)
        self.assertFalse([e for e in ev if e.startswith("scripts/affected.py")])

    def test_a_failing_selection_fails_the_gate(self) -> None:
        code, _, _, err, _ = self._gate("commit", codes={"scripts/affected.py": 1})
        self.assertEqual(code, 1)
        self.assertIn("scripts/affected.py failed; rerun: uv run scripts/affected.py\nboom", err)
        self.assertEqual(GateTest.FakeTier.made, [])

    def test_commit_audits_packages_before_anything_builds(self) -> None:
        sel = "all=0\naudit=1\npackages=\n"
        code, ev, out, _, tmp = self._gate("commit", affected=sel)
        self.assertEqual(code, 0)
        self.assertEqual(ev[-1], "scripts/corpus.py verify --changed staged --only header")
        self.assertEqual(GateTest.FakeTier.made[0].committed, {"all": "0", "audit": "1", "packages": ""})
        self.assertIn("commit-tier checks passed in 12s", out)
        self.assertFalse((tmp / "gate-fast-time.json").exists())
        code, _, _, err, _ = self._gate("commit", affected=sel, codes={"scripts/corpus.py": 1})
        self.assertEqual(code, 1)
        self.assertIn("a staged corpus package fails", err)
        self.assertEqual(GateTest.FakeTier.made, [])

    def test_a_ci_part_other_than_checks_runs_alone(self) -> None:
        code, ev, _, _, _ = self._gate("fast", part="fmt")
        self.assertEqual(code, 0)
        self.assertEqual(ev, [])
        self.assertEqual(GateTest.FakeTier.made[0].parted, 100.0)
        self.assertEqual(self._gate("fast", part="bogus")[0], 1)

    def test_a_slow_fast_gate_warns_and_full_skips_the_budget(self) -> None:
        _, _, out, _, _ = self._gate("fast", elapsed=200)
        self.assertIn("WARNING: fast gate took 200s, over the 180s (3 min) budget", out)
        self.assertEqual(ci.os.environ.get("CPL_NO_WALL_CLOCK"), "1")
        _, _, out, _, tmp = self._gate("full")
        self.assertIn("check: all full checks passed in 12s\n", out)
        self.assertNotIn("CPL_NO_WALL_CLOCK", ci.os.environ)
        self.assertFalse((tmp / "gate-fast-time.json").exists())

    def test_timing_runs_only_the_wall_clock_tests(self) -> None:
        code, ev, _, _, _ = self._gate("timing")
        self.assertEqual(code, 0)
        self.assertEqual(ev, ["cargo nextest run --workspace --no-fail-fast -E " + ci.wall_filter()
                              + " or test(=every_corpus_program_and_conformance_case_is_served)"
                              " or test(=every_port_compiles_with_every_judges_flags)",
                              "scripts/affected.py --run-slices all"])
        self.assertEqual(GateTest.FakeTier.made, [])


class GuardTest(unittest.TestCase):
    # Built at run time so this file's own added lines pass the guard.
    TOKEN = "gh" + "p_" + "a" * 36
    HOME = "/" + "home" + "/someone/"
    HOSTS = ["# comment", r"(www\.)?github\.com", ""]

    def test_scan_added_names_what_must_not_be_published(self) -> None:
        scan = lambda paths=(), lines=(), deny=(): ci.scan_added(list(paths), list(lines), self.HOSTS, list(deny))
        self.assertIsNone(scan(["src/a.rs"], ["+let x = 1;", "+see https://github.com/x"]))
        self.assertIn("build output under dist/", scan(["web/dist/app.js"]))
        self.assertIn("look like credentials", scan(["keys/server.pem"]))
        self.assertIn("look like credentials", scan([".env.local"]))
        why = scan(lines=[f"+token = {self.TOKEN}"])
        self.assertIn("ghp_aaaaaaaa...", why)
        self.assertNotIn(self.TOKEN, why)
        self.assertIn("not in scripts/public-hosts.txt", scan(lines=["+https://" + "intranet.corp/x"]))
        self.assertIsNone(scan(lines=["+https://www.github.com/x"]))
        self.assertIn("personal home paths", scan(lines=[f"+path = {self.HOME}x"]))
        self.assertIn("sensitive-words", scan(lines=["+Project NIGHTJAR"], deny=["nightjar"]))
        self.assertIsNone(scan(lines=["+anything"], deny=["", "  "]), "a blank deny line matches nothing")

    def _repo(self, files: dict[str, str], staged: dict[str, str] | None = None) -> Path:
        tmp = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.enterContext(mock.patch.dict(ci.os.environ, {"GIT_CONFIG_NOSYSTEM": "1"}))
        for k in [k for k in ci.os.environ if k.startswith("GIT_") and k != "GIT_CONFIG_NOSYSTEM"]:
            ci.os.environ.pop(k)
        git = lambda *a: subprocess.run(["git", *a], cwd=tmp, check=True, capture_output=True)
        git("init", "-q")
        git("config", "user.email", "t@example.com")
        git("config", "user.name", "t")
        (tmp / "scripts").mkdir()
        (tmp / "scripts/public-hosts.txt").write_text("\n".join(self.HOSTS) + "\n")
        for rel, text in files.items():
            (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
            (tmp / rel).write_text(text)
        git("add", "-A")
        git("commit", "-q", "-m", "base")
        for rel, text in (staged or {}).items():
            (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
            (tmp / rel).write_text(text)
            git("add", rel)
        return tmp

    def _guard(self, root: Path, mode: str = "fast") -> tuple[int, str]:
        err = io.StringIO()
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(err):
            return ci.guard(mode, root), err.getvalue()

    def test_a_clean_tree_passes(self) -> None:
        root = self._repo({"a.md": "hi\n", "web/package-lock.json": '"resolved": "https://registry.npmjs.org/x"\n'})
        self.assertEqual(self._guard(root), (0, ""))

    def test_conflict_markers_fail(self) -> None:
        root = self._repo({"a.md": "x\n<<<<<<< HEAD\ny\n=======\nz\n>>>>>>> b\n"})
        code, err = self._guard(root)
        self.assertEqual(code, 1)
        self.assertIn("merge conflict markers left in:\na.md:2", err)

    def test_a_program_needs_its_trace_and_a_commit_answers_only_for_its_own(self) -> None:
        old = "crates/cpl-cli/tests/programs/old.cp"
        root = self._repo({old: "print(1)\n"}, staged={"crates/cpl-cli/tests/programs/new.cp": "print(2)\n"})
        code, err = self._guard(root, "commit")
        self.assertEqual(code, 1)
        self.assertIn("programs/new.cp", err)
        self.assertNotIn("old.cp", err)
        self.assertIn(old, self._guard(root, "fast")[1])

    def test_a_commit_may_not_add_corpus_test_files(self) -> None:
        root = self._repo({"a.md": "x\n"}, staged={f"{PKGS}/p/tests/1.in": "1\n"})
        code, err = self._guard(root, "commit")
        self.assertEqual(code, 1)
        self.assertIn("problem-packages/R9", err)
        self.assertEqual(self._guard(root, "fast")[0], 0)

    def test_a_staged_secret_fails_and_is_not_echoed(self) -> None:
        root = self._repo({"a.md": "x\n"}, staged={"b.md": f"key {self.TOKEN}\n"})
        code, err = self._guard(root, "commit")
        self.assertEqual(code, 1)
        self.assertIn("look like secrets", err)
        self.assertNotIn(self.TOKEN, err)

    def test_a_private_npm_mirror_fails(self) -> None:
        root = self._repo({"web/package-lock.json": '"resolved": "https://' + 'npm.corp/x"\n'})
        code, err = self._guard(root)
        self.assertEqual(code, 1)
        self.assertIn("private mirror", err)

    def test_check_sh_runs_the_guards_from_ci(self) -> None:
        check = (Path(__file__).parent / "check.sh").read_text(encoding="utf-8")
        self.assertIn('exec uv run --quiet python scripts/ci.py check "$@"', check)
        self.assertNotIn("staged_guard", check)


class CheckTest(unittest.TestCase):
    """`ci.py check`, which scripts/check.sh execs: flags, caches, guards, then the tier."""

    def _check(self, argv, env=None, guard_code=0, manifest_ok=True, path=None):
        calls, gates = [], []
        with tempfile.TemporaryDirectory() as tmp:
            bin_dir = Path(tmp, "bin")
            bin_dir.mkdir()
            cargo = bin_dir / "cargo"
            cargo.write_text("#!/bin/sh\n")
            cargo.chmod(0o755)
            env = {"PATH": str(bin_dir) if path is None else path, "CHECK_NO_CXX_CACHE": "1", **(env or {})}

            def run(cmd, **kw):
                calls.append(cmd)
                Path(cmd[-1]).write_text("m")
                Path(cmd[-1] + ".reads").write_text("r")
                return subprocess.CompletedProcess(cmd, 0 if manifest_ok else 1)

            def gate(*a):
                gates.append(a)
                self.assertTrue(not a[4] or Path(a[4]).exists())
                return 0

            with mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": str(Path(tmp, "t"))}):
                Path(tmp, "t").mkdir()
                Path(tmp, "t", "gate-fast-time.json").write_text("{}")
                err = io.StringIO()
                with contextlib.redirect_stderr(err):
                    code = ci.check(argv, env=env, run=run, clock=lambda: 7.0,
                                    guard_fn=lambda mode: guard_code, gate_fn=gate)
                left = sorted(f.name for f in Path(tmp, "t").rglob("*") if f.is_file())
        return code, calls, gates, err.getvalue(), left

    def test_the_last_mode_flag_wins_and_the_tier_gets_the_stage_manifest(self) -> None:
        code, calls, gates, _, left = self._check(["--commit", "--land"], env={"CPL_GATE_PART": "x"})
        self.assertEqual(code, 0)
        self.assertEqual(calls[0][1:3], ["scripts/check_cache.py", "manifest"])
        mode, part, cpl, cache_dir, manifest, since = gates[0]
        self.assertEqual((mode, part, since), ("land", "x", 7.0))
        self.assertTrue(cpl.endswith("t/debug/cpl") and cache_dir.endswith("t/check-cache"))
        self.assertTrue(Path(manifest).name.startswith("manifest."))
        # The manifest goes when the gate ends, and the last fast time with it at the start.
        self.assertEqual(left, [])

    def test_full_no_cache_and_a_broken_manifest_turn_the_stage_cache_off(self) -> None:
        for argv, env in ((["--full"], {}), (["--no-cache"], {}), ([], {"CHECK_NO_CACHE": "1"})):
            code, calls, gates, _, _ = self._check(argv, env=env)
            self.assertEqual((code, calls, gates[0][4]), (0, [], ""), argv)
        _, calls, gates, _, _ = self._check([], manifest_ok=False)
        self.assertEqual((len(calls), gates[0][4]), (1, ""))

    def test_an_unknown_flag_a_failed_guard_or_no_cargo_stops_before_the_tier(self) -> None:
        code, _, gates, err, _ = self._check(["--fats"])
        self.assertEqual((code, gates), (2, []))
        self.assertIn("unknown argument `--fats`", err)
        code, _, gates, _, _ = self._check([], guard_code=1)
        self.assertEqual((code, gates), (1, []))
        with tempfile.TemporaryDirectory() as empty:
            code, _, gates, err, _ = self._check([], path=empty)
        self.assertEqual((code, gates), (1, []))
        self.assertIn("see https://rustup.rs", err)

    def test_help_prints_check_sh_header(self) -> None:
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            self.assertEqual(ci.check(["-h"], env={}), 0)
        self.assertTrue(out.getvalue().startswith("The checks to run before a commit or a push."))
        self.assertNotIn("set -euo", out.getvalue())

    def test_the_cxx_cache_shims_g_plus_plus_and_prunes_week_old_binaries(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            bin_dir = Path(tmp, "bin")
            bin_dir.mkdir()
            (bin_dir / "g++").write_text("#!/bin/sh\n")
            (bin_dir / "g++").chmod(0o755)
            cache = Path(tmp, "cache")
            (cache / "ab").mkdir(parents=True)
            old, new = cache / "ab" / "old", cache / "ab" / "new"
            old.write_text("o")
            new.write_text("n")
            os.utime(old, (0, 0))
            os.utime(new, (ci.CXX_CACHE_MAX_AGE_S, ci.CXX_CACHE_MAX_AGE_S))
            target = Path(tmp, "t")
            (target / "judge-probes").mkdir(parents=True)
            env = {"PATH": str(bin_dir), "CPL_CXX_CACHE_DIR": str(cache)}
            seen = []
            run = lambda cmd, **kw: seen.append((cmd, kw["env"]["PATH"])) or subprocess.CompletedProcess(cmd, 0, "k1\n")
            ci.cxx_cache(target, env, run, clock=lambda: ci.CXX_CACHE_MAX_AGE_S + 1.0)
            shim = target / "cxx-shim"
            self.assertEqual((shim / "g++").resolve(), (ci.ROOT / "scripts/cxx-cache.py").resolve())
            self.assertEqual(env["PATH"].split(os.pathsep)[0], str(shim))
            self.assertEqual(seen, [(["g++", "--cpl-system-key"], env["PATH"])])
            self.assertEqual((env["CPL_CXX_SYSTEM_KEY"], env["CPL_CXX_SHIM_DIR"]), ("k1", str(shim)))
            self.assertFalse((target / "judge-probes").exists())
            self.assertEqual((old.exists(), new.exists()), (False, True))
            ci.cxx_cache(target, env, lambda cmd, **kw: subprocess.CompletedProcess(cmd, 1, "x"))
            self.assertEqual(env["CPL_CXX_SYSTEM_KEY"], "")
            off = {"PATH": str(bin_dir), "CHECK_NO_CXX_CACHE": "1"}
            ci.cxx_cache(target, off, run)
            self.assertNotIn("CPL_CXX_SHIM_DIR", off)


if __name__ == "__main__":
    unittest.main()
