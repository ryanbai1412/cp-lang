"""Tests for scripts/land.sh against a throwaway remote."""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
GEN = "spec/conformance/widths/int.cp"
RATCHET = "scripts/debt-ratchet.json"


def sized(gz, fns=None):
    """A --sizes record standing in for a wasm build: crate -> B gz, and `fns` (crate ->
    fn -> normalized B), by default each crate's code the same B in one fn."""
    return json.dumps({"crates": gz, "sorts": 0, "fns": {c: {"f": n} for c, n in gz.items()} | (fns or {})})
INDEX = "corpus/index.json"


def git(cwd, *args):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True).stdout


class LandTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp)
        remote = self.tmp / "remote.git"
        git(self.tmp, "init", "-q", "--bare", "-b", "main", str(remote))
        seed = self.clone("seed")
        (seed / "scripts").mkdir()
        for name in ("land.sh", "wasm-warm.sh", "retry.sh", "ratchet_merge.py", "ratchet_remeasure.py", "gen.py",
                     "new_reds.py", "culprit.py", "domain_of.py"):
            shutil.copy(HERE / name, seed / "scripts" / name)
        (seed / "gen.toml").write_text('[[gen]]\nname = "widths"\nfiles = ["spec/conformance/widths/*.cp"]\n')
        # A regen.sh that dirties a file the rebase never saw: land must not run it.
        (seed / "scripts" / "regen.sh").write_text("#!/bin/sh\necho x >> other.txt\necho other.txt\n")
        (seed / "scripts" / "regen.sh").chmod(0o755)
        # A corpus.py whose build counts the packages, as the real index does.
        (seed / "scripts" / "corpus.py").write_text(
            "import glob, json, os\n"
            "n = len(glob.glob(os.path.join('corpus', 'pkgs', '*', 'meta.json')))\n"
            "open('corpus/index.json', 'w').write(json.dumps({'count': n}) + '\\n')\n")
        (seed / "corpus" / "pkgs" / "a").mkdir(parents=True)
        (seed / "corpus" / "pkgs" / "a" / "meta.json").write_text("{}\n")
        (seed / INDEX).write_text('{"count": 1}\n')
        (seed / GEN).parent.mkdir(parents=True)
        (seed / GEN).write_text("base\n")
        (seed / RATCHET).write_text(json.dumps({"a": 5, "b": 5, "names": ["x", "y"]}, indent=2) + "\n")
        self.commit(seed, "seed")
        git(seed, "push", "-q", "origin", "HEAD:main")

    def clone(self, name):
        path = self.tmp / name
        git(self.tmp, "clone", "-q", str(self.tmp / "remote.git"), str(path))
        git(path, "config", "user.email", "t@example.com")
        git(path, "config", "user.name", "t")
        return path

    def commit(self, repo, msg):
        git(repo, "add", "-A")
        git(repo, "commit", "-q", "--no-verify", "-m", msg, "-m", "Domain: infra\nLane: G")

    def land(self, repo, **extra):
        env = {**{k: v for k, v in os.environ.items() if not k.startswith("CPL_LAND")}, "CPL_LAND_FULL": "1", "CPL_LAND_QUEUE": "1", "CPL_LAND_TRIES": "1", "CPL_LAND_LOCK_POLL": "1", **extra}
        return subprocess.run(["bash", "scripts/land.sh"], cwd=repo, env=env, capture_output=True, text=True)

    def test_a_default_land_runs_no_suite_and_a_full_one_does(self):
        # Lanes push after focused tests; CI and the infra lane run the suites.
        repo = self.clone("lane")
        (repo / "scripts" / "conform_staged.py").write_text("import sys\nsys.exit(1)\n")
        (repo / "x.txt").write_text("x\n")
        self.commit(repo, "x")
        refused = self.land(repo)
        self.assertNotEqual(refused.returncode, 0, refused.stderr)
        self.assertIn("conformance case that passes on main fails", refused.stderr)
        fast = self.land(repo, CPL_LAND_FULL="")
        self.assertEqual(fast.returncode, 0, fast.stderr)
        git(repo, "fetch", "-q", "origin")
        self.assertEqual(git(repo, "log", "-1", "--format=%s", "origin/main").strip(), "x")

    def test_a_std_that_collides_with_mains_is_refused_even_on_a_fast_land(self):
        # Each change passed alone; together std has two impls and no program builds.
        lane = self.clone("lane")
        other = self.clone("other")
        (other / "std").mkdir()
        (other / "std" / "a.cp").write_text("impl\n")
        self.commit(other, "std a")
        self.assertEqual(self.land(other, CPL_LAND_FULL="").returncode, 0)
        with open(lane / ".git" / "info" / "exclude", "a") as f:
            f.write("target/\n")
        (lane / "target" / "debug").mkdir(parents=True)
        fake = lane / "target" / "debug" / "cpl-next"
        fake.write_text('#!/bin/sh\n[ "$(cat "$2"/*.cp)" != "$(printf \'impl\\nimpl\')" ] || '
                        '{ echo "check: E0202: a second impl"; exit 1; }\n')
        fake.chmod(0o755)
        (lane / "std").mkdir()
        (lane / "std" / "b.cp").write_text("impl\n")
        self.commit(lane, "std b")
        refused = self.land(lane, CPL_LAND_FULL="")
        self.assertNotEqual(refused.returncode, 0, refused.stderr)
        self.assertIn("E0202: a second impl", refused.stderr)
        self.assertIn("cpl-next rejects the rebased std", refused.stderr)
        (lane / "std" / "b.cp").write_text("other\n")
        self.commit(lane, "std b, not a second impl")
        landed = self.land(lane, CPL_LAND_FULL="")
        self.assertEqual(landed.returncode, 0, landed.stderr)

    def test_two_lands_from_worktrees_of_one_clone_both_push(self):
        # 29411aa2 sat behind its own land: a second land of one clone shares its .git, so
        # the lock, rebase and base-rerun state must each be per worktree (or locked).
        one = self.clone("one")
        two = self.tmp / "two"
        git(one, "worktree", "add", "-q", "-b", "two", str(two), "origin/main")
        (one / "one.txt").write_text("1\n")
        self.commit(one, "one")
        (two / "two.txt").write_text("2\n")
        self.commit(two, "two")
        env = {**{k: v for k, v in os.environ.items() if not k.startswith("CPL_LAND")},
               "CPL_LAND_FULL": "1", "CPL_LAND_TRIES": "3", "CPL_LAND_LOCK_POLL": "1"}
        runs = [subprocess.Popen(["bash", "scripts/land.sh"], cwd=r, env=env, text=True,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE) for r in (one, two)]
        outs = [p.communicate(timeout=300) for p in runs]
        for p, (_, err) in zip(runs, outs):
            self.assertEqual(p.returncode, 0, err)
        git(one, "fetch", "-q", "origin")
        landed = git(one, "log", "--format=%s", "origin/main")
        self.assertIn("one", landed.split())
        self.assertIn("two", landed.split())

    def test_a_generated_file_conflict_takes_mains_copy_and_lands(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / GEN).write_text("main's\n")
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / GEN).write_text("mine\n")
        (ours / "src.txt").write_text("source\n")
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual((ours / GEN).read_text(), "main's\n")
        self.assertFalse((ours / "other.txt").exists())
        self.assertEqual(git(ours, "status", "--porcelain"), "")
        self.assertIn("src.txt", git(ours, "show", "--name-only", "--format=", "origin/main"))

    def test_end_state_is_regenerated_on_the_rebased_tree(self):
        # Every lane's land moves the ratchet END-STATE.md is generated from; a land that
        # kept its own copy failed the ratchet's stale check and had to run again.
        seed = self.clone("seed2")
        with open(seed / "gen.toml", "a") as f:
            f.write('[[gen]]\nname = "end-state"\nfiles = ["plan/END-STATE.md"]\n')
        (seed / "scripts" / "end-state.py").write_text(
            "import os\nos.makedirs('plan', exist_ok=True)\n"
            f"open('plan/END-STATE.md', 'w').write(open('{RATCHET}').read())\n")
        subprocess.run([sys.executable, "scripts/end-state.py"], cwd=seed, check=True)
        self.commit(seed, "end state")
        git(seed, "push", "-q", "origin", "HEAD:main")
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / RATCHET).write_text(json.dumps({"a": 4, "b": 5, "names": ["x", "y"]}, indent=2) + "\n")
        subprocess.run([sys.executable, "scripts/end-state.py"], cwd=theirs, check=True)
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / RATCHET).write_text(json.dumps({"a": 5, "b": 4, "names": ["x", "y"]}, indent=2) + "\n")
        subprocess.run([sys.executable, "scripts/end-state.py"], cwd=ours, check=True)
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        landed = git(ours, "show", "origin/main:plan/END-STATE.md")
        self.assertEqual(landed, git(ours, "show", f"origin/main:{RATCHET}"))
        self.assertEqual(git(ours, "status", "--porcelain"), "")

    def test_the_case_lists_do_not_merge_by_union(self):
        # merge=union wrote duplicate ids into passing.txt (bdd60b6e5): a list
        # conflict stops the land, and the lists are regenerated from a run.
        rules = [l for l in (HERE.parent / ".gitattributes").read_text().splitlines()
                 if l.startswith("crates/cpl-next/") and "merge=union" in l]
        self.assertEqual(rules, [])

    def test_a_conflict_in_a_hand_written_case_stops_instead_of_taking_mains(self):
        case = "spec/conformance/core-01/x.cp"
        theirs, ours = self.clone("theirs"), self.clone("ours")
        for repo, text in ((theirs, "main's\n"), (ours, "mine\n")):
            (repo / case).parent.mkdir(parents=True)
            (repo / case).write_text(text)
            self.commit(repo, text)
            if repo == theirs:
                git(theirs, "push", "-q", "origin", "HEAD:main")
        r = self.land(ours)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn(f"rebase conflict in {case}", r.stderr)

    def test_an_autostash_that_conflicts_stops_and_names_the_stash(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / RATCHET).write_text(json.dumps({"a": 4, "b": 5, "names": ["x", "y"]}, indent=2) + "\n")
        self.commit(theirs, "tighten a")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "src.txt").write_text("source\n")
        self.commit(ours, "ours")
        (ours / RATCHET).write_text(json.dumps({"a": 5, "b": 3, "names": ["x", "y"]}, indent=2) + "\n")
        r = self.land(ours)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn(f"conflict with main in: {RATCHET}", r.stderr)
        self.assertIn("stash@{0}", r.stderr)
        self.assertNotIn("src.txt", git(ours, "show", "--name-only", "--format=", "origin/main"))

    def test_a_checkout_behind_main_lands_with_mains_land_sh(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        land = theirs / "scripts" / "land.sh"
        land.write_text(land.read_text().replace('cd "$(git rev-parse --show-toplevel)"\n',
                                                 'cd "$(git rev-parse --show-toplevel)"\necho "land: main\'s copy" >&2\n', 1))
        self.commit(theirs, "land")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "src.txt").write_text("source\n")
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stderr.count("land: main's copy"), 1, r.stderr)
        self.assertIn("src.txt", git(ours, "show", "--name-only", "--format=", "origin/main"))

    def _hook(self, repo, body):
        hook = repo / ".githooks" / "pre-commit"
        hook.parent.mkdir(exist_ok=True)
        hook.write_text("#!/bin/sh\n" + body)
        hook.chmod(0o755)

    def test_a_re_exec_keeps_the_once_per_land_gate_rerun(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        self._hook(theirs, "echo gate: ran\n")
        self.commit(theirs, "gate")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        # What a re-exec after a rerun passes on: 4a53bbd4's land reran the tier after each
        # pull that changed land.sh (20+ min).
        r = self.land(ours, CPL_LAND_RECHECKED="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("rerunning the commit tier", r.stderr)
        self.assertIn("main changed the gates again", r.stderr)
        exec_env = Path("scripts/land.sh").read_text().split("reexec_if_land_changed() {")[1].split("\n}")[0]
        self.assertIn("CPL_LAND_RECHECKED=${rechecked:-}", exec_env)
        self.assertIn("CPL_LAND_STEPS_RUNS=${steps_runs:-0}", exec_env)

    def test_main_changing_only_lands_own_scripts_reruns_nothing(self):
        theirs = self.clone("theirs")
        self._hook(theirs, "echo gate: ran\n")
        self.commit(theirs, "gate")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        ours = self.clone("ours")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        (theirs / "scripts" / "test_new.py").write_text("pass\n")
        self.commit(theirs, "a script test")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("main changed only land's own scripts and tests (scripts/test_new.py); no rerun", r.stderr)
        self.assertNotIn("gate: ran", r.stderr)

    def test_a_rebase_past_a_gate_change_reruns_the_commit_tier_on_the_rebased_tip(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        # main's new gate: no staged file may be named bad.txt.
        self._hook(theirs, "! git diff --cached --name-only | grep -qx bad.txt || { echo gate: bad.txt; exit 1; }\n")
        self.commit(theirs, "gate")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "bad.txt").write_text("x\n")
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("rerunning the commit tier", r.stderr)
        self.assertIn("gate: bad.txt", r.stderr)
        self.assertIn("bad.txt", git(ours, "show", "--name-only", "--format=", "HEAD"))
        self.assertNotIn("bad.txt", git(ours, "show", "--name-only", "--format=", "origin/main"))

    def test_the_commit_tier_refuses_only_the_reds_a_change_brings(self):
        # Perf C's fix was refused for 4 tests red on clean main: a test red at
        # the base is main's to fix, one that passes there is the change's.
        theirs, ours, next_ = self.clone("theirs"), self.clone("ours"), self.clone("next")
        (theirs / "scripts" / "fake-tests.sh").write_text(
            "#!/bin/sh\ncat reds*.txt 2>/dev/null | while read -r b t; do echo \"        FAIL [   0.010s] $b $t\"; done\n"
            "! cat reds*.txt 2>/dev/null | grep -q . || exit 100\n")
        (theirs / "reds.txt").write_text("cpl-codegen tests::red_on_main\n")
        (theirs / "crates" / "cpl-codegen" / "src").mkdir(parents=True)
        (theirs / "crates" / "cpl-codegen" / "src" / "lib.rs").write_text("fn red_on_main() {}\n")
        self._hook(theirs, "exec python3 scripts/new_reds.py -- sh scripts/fake-tests.sh -E all\n")
        self.commit(theirs, "main is red")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "x.txt").write_text("x\n")
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("pre-existing red, owner infra: cpl-codegen tests::red_on_main", r.stderr)
        self.assertIn("x.txt", git(ours, "show", "--name-only", "--format=", "origin/main"))
        (next_ / "reds-new.txt").write_text("cpl-codegen tests::new_red\n")
        self.commit(next_, "next")
        r = self.land(next_)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("NEW red: cpl-codegen tests::new_red", r.stderr)
        self.assertIn("pre-existing red, owner infra: cpl-codegen tests::red_on_main", r.stderr)
        self.assertNotIn("reds-new.txt", git(next_, "show", "--name-only", "--format=", "origin/main"))
        # The base's results are cached by its tree: asking again reruns nothing there.
        r = subprocess.run([sys.executable, "scripts/new_reds.py", "--base", "origin/main", "--", "sh", "scripts/fake-tests.sh"],
                           cwd=next_, capture_output=True, text=True)
        self.assertEqual(r.returncode, 1)
        self.assertIn("NEW red: cpl-codegen tests::new_red", r.stderr)
        self.assertNotIn("new-reds: rerunning", r.stderr)

    def test_an_aggregate_red_at_the_base_still_refuses_a_row_it_grows(self):
        # b23f483bf grew ccc20s1/port.cp 0 -> 12 silent unknowns while
        # silent_unknowns_only_go_down was red at the base on cause:branch, and new_reds
        # passed it as main's red: one test id covers every row.
        theirs, same, grows = self.clone("theirs"), self.clone("same"), self.clone("grows")
        (theirs / "scripts" / "fake-tests.sh").write_text(
            "#!/bin/sh\nrows=$(cat rows*.txt 2>/dev/null)\n[ -n \"$rows\" ] || exit 0\n"
            "echo '        FAIL [   0.010s] cpl-conform::silent_unknowns silent_unknowns_only_go_down'\n"
            "echo \"$rows\" | sed 's/^/cpl-finding: silent_unknowns_only_go_down /'\nexit 100\n")
        (theirs / "rows.txt").write_text("cause:branch +6\n")
        self.commit(theirs, "main is red on one row")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        cmd = [sys.executable, "scripts/new_reds.py", "--base", "origin/main", "--", "sh", "scripts/fake-tests.sh"]
        for repo in (same, grows):
            git(repo, "pull", "-q", "origin", "main")
        (same / "x.txt").write_text("x\n")
        r = subprocess.run(cmd, cwd=same, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("pre-existing red", r.stderr)
        (grows / "rows-ours.txt").write_text("ccc20s1/port.cp +12\n")
        r = subprocess.run(cmd, cwd=grows, capture_output=True, text=True)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("NEW rows in cpl-conform::silent_unknowns silent_unknowns_only_go_down", r.stderr)
        self.assertIn("ccc20s1/port.cp (+0 there, +12 here)", r.stderr)
        self.assertNotIn("cause:branch (", r.stderr)
        # The land that brings the rows: the base's test prints none, so its id decides.
        (theirs / "scripts" / "fake-tests.sh").write_text(
            "#!/bin/sh\n[ -f rows.txt ] || exit 0\n"
            "echo '        FAIL [   0.010s] cpl-conform::silent_unknowns silent_unknowns_only_go_down'\nexit 100\n")
        self.commit(theirs, "the test before it printed rows")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        first = self.clone("first")
        (first / "scripts" / "fake-tests.sh").write_text((same / "scripts" / "fake-tests.sh").read_text())
        r = subprocess.run(cmd, cwd=first, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("pre-existing red", r.stderr)

    def test_a_red_at_the_base_with_one_failing_case_refuses_a_second(self):
        # 7a8dcef2's std-wide E0606 passed as main's red: std_own_tests_pass was red at
        # the base on one file, and the change failed another inside the same test id.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "scripts" / "fake-tests.sh").write_text(
            "#!/bin/sh\necho '        FAIL [   0.010s] cpl-cli::std_checks std_own_tests_pass'\n"
            "cat cases*.txt | sed 's/^/cpl-finding: std_own_tests_pass /'\nexit 100\n")
        (theirs / "cases.txt").write_text("std/a.cp:first +1\n")
        self.commit(theirs, "main is red on one case")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "origin", "main")
        (ours / "cases-ours.txt").write_text("std/b.cp:second +1\n")
        cmd = [sys.executable, "scripts/new_reds.py", "--base", "origin/main", "--", "sh", "scripts/fake-tests.sh"]
        r = subprocess.run(cmd, cwd=ours, capture_output=True, text=True)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("std/b.cp:second (+0 there, +1 here)", r.stderr)
        self.assertNotIn("pre-existing red", r.stderr)

    def test_a_base_with_a_stale_build_is_rebuilt_clean_then_judged(self):
        # d2fc481a: the base's cpl-interp built against a stale cpl-rt (E0599 on a variant the
        # base had), and main's own reds refused the land as its.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "scripts" / "fake-tests.sh").write_text(
            "#!/bin/sh\n[ ! -f \"$CARGO_TARGET_DIR/stale\" ] || exit 101\n"
            "echo '        FAIL [   0.010s] cpl-cli::trace_goldens red_on_main'\nexit 100\n")
        self.commit(theirs, "main is red")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "origin", "main")
        common = Path(git(ours, "rev-parse", "--path-format=absolute", "--git-common-dir").strip())
        (common / "cpl-base-target").mkdir()
        (common / "cpl-base-target" / "stale").write_text("old cpl-rt\n")
        cmd = [sys.executable, "scripts/new_reds.py", "--base", "origin/main", "--", "sh", "scripts/fake-tests.sh"]
        env = {k: v for k, v in os.environ.items() if k != "CPL_BASE_TARGET"}
        r = subprocess.run(cmd, cwd=ours, env=env, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("rebuilding its worktree and target", r.stderr)
        self.assertIn("pre-existing red", r.stderr)
        self.assertNotIn("NEW", r.stderr)

    def base_red(self, fake, files=None):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "scripts" / "fake-tests.sh").write_text(
            fake + "echo '        FAIL [   0.010s] cpl-cli::trace_goldens red_on_main'\nexit 100\n")
        for name, text in (files or {}).items():
            (theirs / name).write_text(text)
        self.commit(theirs, "main is red")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "origin", "main")
        return ours

    def run_new_reds(self, ours, **env):
        cmd = [sys.executable, "scripts/new_reds.py", "--base", "origin/main", "--", "sh", "scripts/fake-tests.sh"]
        env = {**{k: v for k, v in os.environ.items() if k != "CPL_BASE_TARGET"}, **env}
        return subprocess.run(cmd, cwd=ours, env=env, capture_output=True, text=True)

    def test_a_base_run_of_a_cpl_built_for_another_checkout_is_rebuilt_then_judged(self):
        # Tools at 82ccb3f98: trace_goldens failed at the base on a stale dev cpl from the
        # shared base worktree, and the red passed as main's.
        stale = "echo 'cpl: this dev `cpl` reads std from /gone/std, but runs from the checkout at /x'\n"
        ours = self.base_red(f'#!/bin/sh\n[ ! -f "$CARGO_TARGET_DIR/stale" ] || {stale}')
        common = Path(git(ours, "rev-parse", "--path-format=absolute", "--git-common-dir").strip())
        (common / "cpl-base-target").mkdir()
        (common / "cpl-base-target" / "stale").write_text("built in another checkout\n")
        r = self.run_new_reds(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("ran a cpl built for another checkout; rebuilding its worktree and target", r.stderr)
        self.assertIn("pre-existing red", r.stderr)

    def test_a_base_run_whose_cpl_stays_stale_refuses_and_is_never_main_red(self):
        stale = "echo 'cpl: this dev `cpl` reads std from /gone/std, but runs from the checkout at /x'\n"
        ours = self.base_red(f'#!/bin/sh\n[ -z "$CARGO_TARGET_DIR" ] || {stale}')
        r = self.run_new_reds(ours)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("ran a cpl built for another checkout in", r.stderr)
        self.assertIn("can't be compared", r.stderr)
        self.assertNotIn("pre-existing red", r.stderr)

    def test_a_base_tree_that_moves_during_its_run_is_rerun_before_it_is_cached(self):
        # Tools' and L2's silent_unknowns rows read cause:call +8 "there" where a clean run
        # at the same base gives +18: the shared worktree had another tree under the run.
        mark = self.tmp / "moved-once"
        ours = self.base_red('#!/bin/sh\n[ -z "$CARGO_TARGET_DIR" ] || [ -f "$MARK" ] || '
                             '{ touch "$MARK"; echo another > tree.txt; }\n', {"tree.txt": "base\n"})
        r = self.run_new_reds(ours, MARK=str(mark))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("moved to another tree during the run; rebuilding", r.stderr)
        self.assertIn("pre-existing red", r.stderr)
        common = Path(git(ours, "rev-parse", "--path-format=absolute", "--git-common-dir").strip())
        tree = git(ours, "rev-parse", "origin/main^{tree}").strip()
        cache = json.loads((common / "cpl-base-tests" / f"{tree}.json").read_text())
        self.assertEqual(cache["cpl-cli::trace_goldens red_on_main"], "fail")

    def test_a_base_that_doesnt_build_is_rebuilt_before_its_reds_count_as_new(self):
        # perf's ord: the base worktree lacked a test file ("couldn't find file"), the
        # base run failed to build, and two reds main had too refused the land.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "scripts" / "fake-tests.sh").write_text(
            "#!/bin/sh\n[ ! -f broken.txt ] || exit 101\n"
            "echo '        FAIL [   0.010s] cpl-cli::trace_goldens red_on_main'\nexit 100\n")
        self.commit(theirs, "main is red")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "origin", "main")
        common = Path(git(ours, "rev-parse", "--path-format=absolute", "--git-common-dir").strip())
        git(ours, "worktree", "add", "-q", "--detach", str(common / "cpl-base-wt"), "HEAD")
        (common / "cpl-base-wt" / "broken.txt").write_text("stale\n")
        cmd = [sys.executable, "scripts/new_reds.py", "--base", "origin/main", "--", "sh", "scripts/fake-tests.sh"]
        r = subprocess.run(cmd, cwd=ours, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("didn't build; rebuilding its worktree", r.stderr)
        self.assertIn("pre-existing red", r.stderr)

    def test_every_aggregate_rust_test_reports_its_rows(self):
        # An `*_only_go_down` test is one id over many rows; without cpl-finding lines a
        # red at the base hides every row a land grows.
        root = Path(__file__).resolve().parent.parent
        silent = []
        for f in (root / "crates").rglob("*.rs"):
            src = f.read_text()
            for name in re.findall(r"fn (\w+_only_go_down|std_errors_never_grow|std_own_tests_pass|every_program_runs_traced_and_untraced)\(", src):
                if f"cpl-finding: {name} " not in src:
                    silent.append(f"{f.relative_to(root)}: {name}")
        self.assertEqual(silent, [], "print `cpl-finding: <test> <row> +<excess>` per grown row")

    def test_a_new_red_that_passes_alone_at_the_tip_is_flaky_not_refused(self):
        # cpl-wasm parity's transcript order raced under the full run only, and
        # refused d2fc481a's unrelated fix as a new red.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "scripts" / "fake-tests.sh").write_text(
            "#!/bin/sh\n"
            # Alone (-E names one test) the racy test passes; the full run fails it.
            'case "$*" in *"test(=tests::racy)"*) [ -f solid.txt ] || exit 0 ;; esac\n'
            "if [ -f racy.txt ]; then echo '        FAIL [   0.010s] cpl-wasm tests::racy'; exit 100; fi\n"
            "exit 0\n")
        self.commit(theirs, "fake tests")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "origin", "main")
        cmd = [sys.executable, "scripts/new_reds.py", "--base", "origin/main", "--", "sh", "scripts/fake-tests.sh"]
        (ours / "racy.txt").write_text("x\n")
        r = subprocess.run(cmd, cwd=ours, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("cpl-wasm tests::racy failed in the full run, passes alone here and at the base; not refusing", r.stderr)
        self.assertNotIn("NEW red", r.stderr)
        # One that fails alone too is the change's.
        (ours / "solid.txt").write_text("x\n")
        r = subprocess.run(cmd, cwd=ours, capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("NEW red: cpl-wasm tests::racy", r.stderr)

    def test_a_test_missing_or_unbuilt_at_the_base_is_a_new_red(self):
        # 617dc8f83 landed with its own new test red: a test the base doesn't have
        # (nextest exits 4, no tests) or can't build (exit 101) is the change's red.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "scripts" / "fake-tests.sh").write_text(
            "#!/bin/sh\n"
            "if [ -f new-red.txt ]; then echo '        FAIL [   0.010s] cpl-codegen tests::brand_new'; exit 100; fi\n"
            "[ -f new-pass.txt ] && exit 0\n"
            "[ -f base-broken.txt ] && exit 101\n"
            "exit 4\n")
        self.commit(theirs, "fake tests")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "origin", "main")
        cmd = [sys.executable, "scripts/new_reds.py", "--base", "origin/main", "--", "sh", "scripts/fake-tests.sh"]
        (ours / "new-pass.txt").write_text("x\n")
        r = subprocess.run(cmd, cwd=ours, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        (ours / "new-pass.txt").unlink()
        (ours / "new-red.txt").write_text("x\n")
        r = subprocess.run(cmd, cwd=ours, capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("NEW red: cpl-codegen tests::brand_new", r.stderr)
        self.assertNotIn("pre-existing red", r.stderr)
        (theirs / "base-broken.txt").write_text("x\n")
        self.commit(theirs, "base doesn't build")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "fetch", "-q", "origin")
        r = subprocess.run(cmd, cwd=ours, capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("NEW red: cpl-codegen tests::brand_new", r.stderr)
        self.assertNotIn("pre-existing red", r.stderr)

    def test_the_ratchet_measures_with_a_cpl_built_at_the_rebased_tip(self):
        # corpus_fmt_dirty refuses a cpl older than the sources the rebase brought in.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "Cargo.toml").write_text("[workspace]\n")
        (theirs / "scripts" / "debt-ratchet.py").write_text(
            "import os, sys\nsys.exit(0 if os.path.exists('built') else 'cpl is stale')\n")
        self.commit(theirs, "ratchet")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        bin_dir = self.tmp / "bin"
        bin_dir.mkdir()
        (bin_dir / "cargo").write_text('#!/bin/sh\ncase "$*" in *"build -q -p cpl-cli"*) touch built;; esac\n')
        (bin_dir / "cargo").chmod(0o755)
        (ours / ".gitignore").write_text("built\n")
        self.commit(ours, "ours")
        r = self.land(ours, PATH=f"{bin_dir}{os.pathsep}{os.environ['PATH']}")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue((ours / "built").exists())

    def test_the_rerun_hands_the_hook_no_stale_cpl(self):
        # A cpl copied from before the rebase escapes the dep-info staleness check
        # and measures main's old sources; the rerun leaves the hook to build its own.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        self._hook(theirs, 'echo "rerun cpl=${CPL:-unset}"\n')
        self.commit(theirs, "gate")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "target" / "release").mkdir(parents=True)
        (ours / "target" / "release" / "cpl").write_text("#!/bin/sh\n")
        (ours / "target" / "release" / "cpl").chmod(0o755)
        (ours / ".gitignore").write_text("target/\n")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("rerun cpl=unset", r.stderr)

    def test_slices_the_hooks_cut_skipped_run_before_the_push(self):
        ours = self.clone("ours")
        (ours / "scripts" / "affected.py").write_text(
            "import sys\nprint('slices', *sys.argv[1:], file=sys.stderr)\n"
            "sys.exit(1 if sys.argv[1] == '--run-slices' and sys.argv[-1] == 'copies' else 0)\n")
        self.commit(ours, "a slice runner")
        git(ours, "push", "-q", "origin", "HEAD:main")
        for slices, code in (("copies\n", 1), ("vm-trace\nvm-trace\n", 0)):
            pending = ours / ".git" / "cpl-pending-slices"
            pending.write_text(slices)
            (ours / "pkg.txt").write_text(slices)
            self.commit(ours, "ours")
            r = self.land(ours)
            self.assertEqual(r.returncode, code, r.stderr)
            self.assertRegex(r.stderr, r"slices --run-slices --base [0-9a-f]{40} " + slices.split()[0])
            self.assertEqual(pending.exists(), code != 0)
            git(ours, "fetch", "-q")
            self.assertEqual(git(ours, "show", "origin/main:pkg.txt") if code == 0 else
                             git(ours, "ls-tree", "origin/main", "pkg.txt"), "" if code else slices)

    def test_the_traits_gate_refuses_only_problem_lines_main_doesnt_have(self):
        ours = self.clone("ours")
        (ours / "scripts" / "spec-gate.py").write_text(
            "import pathlib, sys\n"
            "names = (pathlib.Path(__file__).resolve().parents[1] / 'std' / 'gaps.txt').read_text().split()\n"
            "for n in names: print(f'spec-gate traits: traits.toml: no case: {n} (write one)')\n"
            "sys.exit(1 if names else 0)\n")
        (ours / "std").mkdir()
        (ours / "std" / "gaps.txt").write_text("Main/A/a\n")
        self.commit(ours, "a traits gate main fails")
        git(ours, "push", "-q", "origin", "HEAD:main")
        # Swapping main's problem for another still refuses: lines are compared, not counts.
        for gaps, code in (("Mine/B/b\n", 1), ("Main/A/a\nMine/B/b\n", 1), ("Main/A/a\n", 0)):
            git(ours, "reset", "-q", "--hard", "origin/main")
            (ours / "std" / "gaps.txt").write_text(gaps)
            (ours / "std" / "x.cp").write_text(gaps)
            self.commit(ours, "ours")
            r = self.land(ours)
            self.assertEqual(r.returncode, code, r.stderr)
            self.assertNotIn("no case: Main/A/a", r.stderr)
            if code:
                self.assertIn("no case: Mine/B/b", r.stderr)
            else:
                self.assertIn("1 problems, all already on main", r.stderr)

    def test_land_runs_the_slices_the_rebased_tips_affected_py_selects(self):
        # fc73818ec was committed before main had the silent-unknowns slice, so its hook
        # never chose it and land ran nothing: land selects again at the rebased tip.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "scripts" / "affected.py").write_text(
            "import sys\nprint('slices', *sys.argv[1:], file=sys.stderr)\n"
            "if sys.argv[1:] == ['--base', 'FETCH_HEAD']: print('slices=silent-unknowns')\n"
            "sys.exit(1 if sys.argv[1:2] == ['--run-slices'] and 'silent-unknowns' in sys.argv else 0)\n")
        self.commit(theirs, "a slice main added")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "std").mkdir()
        (ours / "std" / "x.cp").write_text("ours\n")
        self.commit(ours, "ours")
        self.assertFalse((ours / ".git" / "cpl-pending-slices").exists())
        r = self.land(ours)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertRegex(r.stderr, r"slices --run-slices --base [0-9a-f]{40} silent-unknowns")
        self.assertIn("land: a touched-path slice fails at the rebased tip", r.stderr)
        self.assertNotIn("ours", git(ours, "log", "--format=%s", "origin/main"))

    def test_conformance_cases_the_hooks_cut_skipped_run_before_the_push(self):
        ours = self.clone("ours")
        (ours / "scripts" / "conform_staged.py").write_text(
            "import sys\nprint('cases', *sys.argv[1:], file=sys.stderr)\nsys.exit(1 if 'bad.cp' in sys.argv else 0)\n")
        self.commit(ours, "a case runner")
        git(ours, "push", "-q", "origin", "HEAD:main")
        pending = ours / ".git" / "cpl-pending-cases"
        for case, code in (("bad.cp", 1), ("good.cp", 0)):
            (ours / case).write_text("x\n")
            self.commit(ours, case)
            pending.write_text(case + "\n")
            r = self.land(ours)
            self.assertEqual(r.returncode, code, r.stderr)
            self.assertRegex(r.stderr, rf"cases --base [0-9a-f]{{40}} --build {case}")  # held to the reds it brings
            self.assertEqual(pending.exists(), code != 0)
            if code:
                git(ours, "rm", "-q", case)
                self.commit(ours, "drop " + case)

    def _steps_runner(self, repo, check_steps="", sleep=0):
        """main's hook and a check.sh that lists `check_steps` as unrun and takes
        `sleep` s, and a ci.py whose `commit --only` refuses wasm with a warning."""
        hooks = repo / ".githooks"
        hooks.mkdir(exist_ok=True)
        shutil.copy(HERE.parent / ".githooks" / "pre-commit", hooks / "pre-commit")
        check = repo / "scripts" / "check.sh"
        check.write_text(f'#!/bin/sh\nprintf "{check_steps}" >"$CPL_COMMIT_PROGRESS"\n'
                         '[ -z "$LOCKS" ] || git ls-remote origin refs/land/lock-v2 | cut -f1 | grep . >>"$LOCKS" || echo free >>"$LOCKS"\n'
                         f'echo "==> cargo check -p cpl-wasm"\nexec sleep {sleep}\n')
        check.chmod(0o755)
        (repo / "scripts" / "ci.py").write_text(
            "import os, subprocess, sys\n"
            "runs = os.environ['CI_RUNS']\n"
            "first = not os.path.exists(runs)\n"
            "open(runs, 'a').write(' '.join(sys.argv[1:]) + '\\n')\n"
            "if (first or len(open(runs).read().splitlines()) <= int(os.environ.get('RACES', 1))) and os.environ.get('RACE'):\n"
            "    if not first:\n"
            "        subprocess.run(['git', '-c', 'user.name=t', '-c', 'user.email=t@t', 'commit', '-q', '--allow-empty', '-m', 'race'], cwd=os.environ['RACE'], check=True)\n"
            "    subprocess.run(['git', 'push', '-q', 'origin', 'HEAD:main'], cwd=os.environ['RACE'], check=True)\n"
            "if os.environ.get('LOCKS'):\n"
            "    held = subprocess.run(['git', 'ls-remote', 'origin', 'refs/land/lock-v2'], capture_output=True, text=True).stdout\n"
            "    open(os.environ['LOCKS'], 'a').write((held.split() or ['free'])[0] + '\\n')\n"
            "sys.exit('warning: field `0` is never read' if 'wasm' in sys.argv[-1] else 0)\n")
        self.commit(repo, "a commit tier")
        git(repo, "push", "-q", "origin", "HEAD:main")
        return repo / ".git" / "cpl-pending-steps", self.tmp / "ci.runs"

    def _hooked_commit(self, repo, msg):
        env = {**os.environ, "CPL_COMMIT_TIMEOUT": "1"}
        git(repo, "add", "-A")
        subprocess.run(["git", "-c", "core.hooksPath=.githooks", "commit", "-q", "-m", msg, "-m", "Domain: infra\nLane: G"],
                       cwd=repo, env=env, check=True, capture_output=True, text=True)

    def test_a_cut_before_the_wasm_check_leaves_it_to_land_which_refuses_its_warning(self):
        ours = self.clone("ours")
        pending, runs = self._steps_runner(ours, "wasm\\nnextest\\n", sleep=5)
        (ours / "pkg.txt").write_text("x\n")
        self._hooked_commit(ours, "ours")
        self.assertEqual(pending.read_text().split(), ["wasm", "nextest"])
        r = self.land(ours, CI_RUNS=str(runs))
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("warning: field `0` is never read", r.stderr)
        self.assertIn("uv run scripts/ci.py commit --only nextest,wasm", r.stderr)
        self.assertEqual(runs.read_text(), "commit --only nextest,wasm\n")
        self.assertTrue(pending.exists())
        git(ours, "fetch", "-q")
        self.assertEqual(git(ours, "ls-tree", "origin/main", "pkg.txt"), "")

    def test_a_hook_that_got_through_leaves_no_step_pending(self):
        ours = self.clone("ours")
        pending, runs = self._steps_runner(ours, "wasm\\n")
        (ours / "pkg.txt").write_text("x\n")
        self._hooked_commit(ours, "ours")
        self.assertFalse(pending.exists())
        r = self.land(ours, CI_RUNS=str(runs))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertFalse(runs.exists())

    def test_after_a_lost_push_race_the_pending_steps_dont_run_again(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        git(theirs, "pull", "-q", "origin", "main")
        (theirs / "theirs.txt").write_text("x\n")
        self.commit(theirs, "theirs")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("nextest\n")
        r = self.land(ours, CI_RUNS=str(runs), RACE=str(theirs), CPL_LAND_TRIES="2")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(runs.read_text(), "commit --only nextest\n")
        self.assertIn("landing on the skipped steps' rerun", r.stderr)
        self.assertIn("pushed", r.stdout)
        self.assertFalse(pending.exists())

    def lock(self, repo, age=0):
        """Hold refs/land/lock-v2 on the remote as a lane that took it `age` s ago."""
        empty = git(repo, "hash-object", "-t", "tree", "/dev/null").strip()
        sha = git(repo, "commit-tree", empty, "-m", f"land-lock other 1 {int(time.time()) - age} infra").strip()
        git(repo, "push", "-q", "origin", f"{sha}:refs/land/lock-v2")
        return sha

    def note_on_remote(self, repo):
        sha = self.lock_on_remote().split()[0]
        git(repo, "fetch", "-q", "origin", "refs/land/lock-v2")
        return git(repo, "log", "-1", "--format=%s", sha)

    def lock_on_remote(self):
        return git(self.tmp, "ls-remote", str(self.tmp / "remote.git"), "refs/land/lock-v2").strip()

    def test_a_waiter_names_the_holders_branch_step_and_last_beat(self):
        repo = self.clone("lane")
        empty = git(repo, "hash-object", "-t", "tree", "/dev/null").strip()
        note = f"land-lock rival 7 {int(time.time()) - 5} infra feat@abc1234 pushing"
        git(repo, "push", "-q", "origin", git(repo, "commit-tree", empty, "-m", note).strip() + ":refs/land/lock-v2")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_WAIT="2")
        self.assertNotEqual(r.returncode, 0)
        self.assertRegex(r.stderr, r"waiting for the land lock \(land-lock rival 7 \d+ infra feat@abc1234 pushing; beat [5-9]s ago\)")

    def test_a_holders_lock_names_its_branch_head_and_step(self):
        repo = self.clone("lane")
        notes = self.tmp / "notes"
        hook = self.tmp / "remote.git" / "hooks" / "post-receive"
        hook.write_text("#!/bin/sh\nwhile read old new ref; do\n"
                        f'  [ "$ref" != refs/land/lock-v2 ] || git log -1 --format=%s "$new" >>"{notes}" 2>/dev/null\n'
                        "done\n")
        hook.chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo)
        self.assertEqual(r.returncode, 0, r.stderr)
        where = git(repo, "rev-parse", "--abbrev-ref", "HEAD").strip() + "@" + git(repo, "rev-parse", "--short", "HEAD").strip()
        self.assertIn(f" infra {where} pushing", notes.read_text())

    def test_a_land_takes_the_land_lock_and_drops_it(self):
        repo = self.clone("lane")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_held_land_lock_makes_land_wait_and_leaves_it_alone(self):
        repo = self.clone("lane")
        held = self.lock(repo)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_WAIT="2")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("waiting for the land lock (land-lock other", r.stderr)
        self.assertIn("no land lock after", r.stderr)
        self.assertTrue(self.lock_on_remote().startswith(held))
        self.assertNotEqual(git(repo, "rev-parse", "HEAD"), git(repo, "ls-remote", "origin", "main").split()[0] + "\n")

    def test_a_dead_lanes_stale_land_lock_is_taken_over(self):
        repo = self.clone("lane")
        self.lock(repo, age=1000)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_BEAT="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("taking over a land lock", r.stderr)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_remote_that_refuses_the_lock_refuses_the_land(self):
        repo = self.clone("lane")
        hook = self.tmp / "remote.git" / "hooks" / "pre-receive"
        hook.write_text("#!/bin/sh\nwhile read old new ref; do\n"
                        '  [ "$ref" != refs/land/lock-v2 ] || { echo "HTTP 500" >&2; exit 1; }\n'
                        "done\n")
        hook.chmod(0o755)
        main = git(repo, "rev-parse", "origin/main").strip()
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo)
        self.assertNotEqual(r.returncode, 0, r.stderr)
        self.assertIn("refuses refs/land/lock-v2", r.stderr)
        self.assertIn("land again", r.stderr)
        self.assertNotIn("without the land lock", r.stderr)
        self.assertEqual(git(repo, "ls-remote", "origin", "refs/heads/main").split()[0], main)

    def test_a_lock_taken_after_a_wait_is_stamped_when_it_is_taken(self):
        repo, other = self.clone("lane"), self.clone("other")
        self.lock(repo)
        stamps = self.tmp / "stamps"
        hook = self.tmp / "remote.git" / "hooks" / "pre-receive"
        hook.write_text("#!/bin/sh\nwhile read old new ref; do\n"
                        f'  [ "$ref" != refs/land/lock-v2 ] || git log -1 --format=%s "$new" >>"{stamps}" 2>/dev/null || :\n'
                        "done\n")
        hook.chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        freed = []
        timer = threading.Timer(3, lambda: (freed.append(time.time()),
                                            git(other, "push", "-q", "origin", ":refs/land/lock-v2")))
        timer.start()
        r = self.land(repo)
        timer.join()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("took the land lock after", r.stderr)
        taken = [int(s.split()[3]) for s in stamps.read_text().splitlines() if " other " not in f" {s} "]
        self.assertTrue(taken and taken[0] >= int(freed[0]), (taken, freed))

    def test_two_worktrees_landing_at_once_each_run_their_own_copy_of_mains_land_sh(self):
        a = self.clone("a")
        b = self.tmp / "b"
        git(a, "worktree", "add", "-q", "-b", "b", str(b), "HEAD")
        (a / "a.txt").write_text("a\n")
        self.commit(a, "a")
        (b / "b.txt").write_text("b\n")
        self.commit(b, "b")
        theirs = self.clone("theirs")
        land = theirs / "scripts" / "land.sh"
        # v1 stops a's land in its copy until b has landed with v2 (once: a re-execs
        # into v2 after its pull).
        v1 = land.read_text().replace("set -euo pipefail\n", 'set -euo pipefail\n[ -z "${CPL_TEST_GO:-}" ] || '
                                      '{ echo >"$CPL_TEST_READY"; read -r _ <"$CPL_TEST_GO"; unset CPL_TEST_GO; }\n', 1)
        land.write_text(v1)
        self.commit(theirs, "main's land.sh v1")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        ready, go = self.tmp / "ready", self.tmp / "go"
        os.mkfifo(ready)
        os.mkfifo(go)
        env = {**{k: v for k, v in os.environ.items() if not k.startswith("CPL_LAND")}, "CPL_LAND_FULL": "1", "CPL_LAND_TRIES": "3",
               "CPL_TEST_READY": str(ready), "CPL_TEST_GO": str(go)}
        first = subprocess.Popen(["bash", "scripts/land.sh"], cwd=a, env=env, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, text=True)
        ready.read_text()
        # main's land.sh changes while a's land waits in its copy of v1; b's land copies v2,
        # whose long first line puts a's read offset mid-line in a shared copy.
        shebang, rest = v1.split("\n", 1)
        land.write_text(f"{shebang}\n#{'x' * (len(v1) // 4)}\n{rest}")
        self.commit(theirs, "main's land.sh v2")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        second = self.land(b, CPL_LAND_TRIES="3")
        go.write_text("\n")
        out, err = first.communicate(timeout=120)
        self.assertEqual(first.returncode, 0, err)
        self.assertEqual(second.returncode, 0, second.stderr)
        git(theirs, "fetch", "-q")
        log = git(theirs, "log", "--format=%s", "origin/main")
        self.assertIn("a", log.split())
        self.assertIn("b", log.split())

    def test_a_change_to_land_sh_lands_with_its_own_copy(self):
        ours = self.clone("ours")
        land = ours / "scripts" / "land.sh"
        land.write_text(land.read_text().replace("set -euo pipefail\n",
                                                 "set -euo pipefail\necho 'land: the new copy runs' >&2\n", 1))
        self.commit(ours, "a new land.sh")
        # The land starts on main's copy; the first pull puts this change's copy in
        # the tree, and re-execs before any lock is held.
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stderr.count("landing with the new copy"), 1, r.stderr)
        # Once before it hands over to main's copy, once after the re-exec.
        self.assertEqual(r.stderr.count("the new copy runs"), 2, r.stderr)
        self.assertIn("pushed", r.stdout)
        self.assertEqual(self.lock_on_remote(), "")

    def test_every_pull_that_changes_land_sh_re_execs(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        git(theirs, "pull", "-q", "origin", "main")
        land = theirs / "scripts" / "land.sh"
        land.write_text(land.read_text() + "# main's later fix\n")
        self.commit(theirs, "main's land.sh")
        mine = ours / "scripts" / "land.sh"
        mine.write_text(mine.read_text().replace("set -euo pipefail\n",
                                                 "set -euo pipefail\necho 'land: the new copy runs' >&2\n", 1))
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("nextest\n")
        # Ours's own copy at the first pull, then main's fix that lands mid-run.
        r = self.land(ours, CI_RUNS=str(runs), RACE=str(theirs), CPL_LAND_TRIES="3")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stderr.count("landing with the new copy"), 2, r.stderr)
        self.assertIn("pushed", r.stdout)

    def test_a_pull_that_changes_land_sh_lands_with_the_new_copy_once(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        git(theirs, "pull", "-q", "origin", "main")
        land = theirs / "scripts" / "land.sh"
        land.write_text(land.read_text().replace("set -euo pipefail\n",
                                                 "set -euo pipefail\necho 'land: the new copy runs' >&2\n", 1))
        self.commit(theirs, "a new land.sh")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("nextest\n")
        # theirs lands the new land.sh while ours runs its pending steps, so ours's push loses the race.
        r = self.land(ours, CI_RUNS=str(runs), RACE=str(theirs), CPL_LAND_TRIES="3")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stderr.count("landing with the new copy"), 1, r.stderr)
        self.assertEqual(r.stderr.count("the new copy runs"), 1, r.stderr)
        self.assertIn("pushed", r.stdout)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_land_that_loses_the_race_reruns_without_the_land_lock(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        git(theirs, "pull", "-q", "origin", "main")
        # A gate change: the lost race reruns the commit tier, too slow to hold the lock.
        (theirs / "scripts" / "theirs.txt").write_text("x\n")
        self.commit(theirs, "theirs")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("nextest\n")
        locks = self.tmp / "locks"
        r = self.land(ours, CI_RUNS=str(runs), RACE=str(theirs), LOCKS=str(locks), CPL_LAND_TRIES="2")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("pushed", r.stdout)
        # The skipped steps, then the lost race's commit-tier rerun, ran while another
        # lane could take the lock.
        self.assertEqual(locks.read_text(), "free\nfree\n")
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_lock_held_past_120s_is_stale_by_default(self):
        repo = self.clone("lane")
        self.lock(repo, age=150)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_BEAT="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertRegex(r.stderr, r"taking over a land lock last beat 1[5-9]\ds ago \(holder other 1, no beat for over 120s\)")

    def beat_as_rival(self, repo, age):
        """Restamp refs/land/lock-v2 every 0.3 s as live holder `rival 7`, `age` s behind this clock; stop() ends it."""
        empty = git(repo, "hash-object", "-t", "tree", "/dev/null").strip()
        stop = threading.Event()

        def beat():
            n = 0
            while not stop.is_set():
                n += 1
                note = f"land-lock rival 7 {int(time.time()) - age} infra feat@abc1234 pushing {n}"
                git(repo, "push", "-q", "-f", "origin", git(repo, "commit-tree", empty, "-m", note).strip() + ":refs/land/lock-v2")
                stop.wait(0.3)

        git(repo, "push", "-q", "origin", git(repo, "commit-tree", empty, "-m", f"land-lock rival 7 {int(time.time()) - 115} infra").strip() + ":refs/land/lock-v2")
        t = threading.Thread(target=beat)
        t.start()
        return lambda: (stop.set(), t.join())

    def test_a_holder_beating_while_a_waiter_waits_past_120s_is_never_taken_over(self):
        # Perf C took lint-fmt's live lock: a holder first seen 115 s old, beating since.
        repo = self.clone("lane")
        stop = self.beat_as_rival(self.clone("rival"), age=0)
        try:
            (repo / "pkg.txt").write_text("x\n")
            self.commit(repo, "ours")
            r = self.land(repo, CPL_LAND_LOCK_WAIT="8", CPL_LAND_LOCK_BEAT="1")
        finally:
            stop()
        self.assertNotEqual(r.returncode, 0)
        self.assertNotIn("taking over", r.stderr)
        self.assertIn("no land lock after", r.stderr)

    def test_a_holder_beating_on_a_clock_behind_the_waiters_is_never_taken_over(self):
        repo = self.clone("lane")
        stop = self.beat_as_rival(self.clone("rival"), age=150)
        try:
            (repo / "pkg.txt").write_text("x\n")
            self.commit(repo, "ours")
            r = self.land(repo, CPL_LAND_LOCK_WAIT="8", CPL_LAND_LOCK_BEAT="1")
        finally:
            stop()
        self.assertNotEqual(r.returncode, 0)
        self.assertNotIn("taking over", r.stderr)
        self.assertIn("rival 7", git(repo, "ls-remote", "origin", "refs/land/lock-v2") and self.note_on_remote(repo))

    def held_note(self, repo, age, head):
        empty = git(repo, "hash-object", "-t", "tree", "/dev/null").strip()
        note = f"land-lock rival 7 {int(time.time()) - age} infra feat@{head} pushing"
        git(repo, "push", "-q", "-f", "origin", git(repo, "commit-tree", empty, "-m", note).strip() + ":refs/land/lock-v2")

    def test_a_lock_whose_holders_head_is_on_main_is_taken_once_a_beat_is_missed(self):
        # lint-fmt and compiler pushed, then died before deleting: each waiter paid 120 s.
        repo = self.clone("lane")
        self.held_note(repo, 30, git(repo, "rev-parse", "--short", "origin/main").strip())
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_WAIT="5")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertRegex(r.stderr, r"taking over a land lock last beat 3\ds ago \(holder rival 7, [0-9a-f]+ is on main\)")

    def test_a_lock_whose_holders_head_is_not_on_main_waits_out_its_beat(self):
        repo = self.clone("lane")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        self.held_note(repo, 30, git(repo, "rev-parse", "--short", "HEAD").strip())
        r = self.land(repo, CPL_LAND_LOCK_WAIT="3")
        self.assertNotEqual(r.returncode, 0)
        self.assertNotIn("taking over", r.stderr)

    def slow_beats_and_timed_refs(self, fail_delete=False):
        """A git that takes 3 s over each lock beat (and fails lock deletes with `fail_delete`),
        and a remote that logs `<time> <ref> <new>` per update; returns (PATH, log)."""
        log = self.tmp / "refs.log"
        hook = self.tmp / "remote.git" / "hooks" / "post-receive"
        hook.write_text(f'#!/bin/sh\nwhile read old new ref; do echo "$(date +%s.%N) $ref $new" >>"{log}"; done\n')
        hook.chmod(0o755)
        shim = self.tmp / "bin"
        shim.mkdir()
        real = shutil.which("git")
        fail = 'case "$*" in *" :refs/land/lock-v2"*) exit 1 ;; esac\n' if fail_delete else ""
        (shim / "git").write_text(
            "#!/bin/sh\n" + fail +
            'case "$*" in *"--force-with-lease=refs/land/lock-v2:"[0-9a-f]*" "[0-9a-f]*":refs/land/lock-v2"*) sleep 3 ;; esac\n'
            f'exec "{real}" "$@"\n')
        (shim / "git").chmod(0o755)
        return f"{shim}:{os.environ['PATH']}", log

    def test_a_holder_deletes_its_lock_right_after_its_push_not_after_its_beat(self):
        repo = self.clone("lane")
        path, log = self.slow_beats_and_timed_refs()
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=path, CPL_LAND_LOCK_BEAT="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        rows = [line.split() for line in log.read_text().splitlines()]
        pushed = next(float(t) for t, ref, _ in rows if ref == "refs/heads/main")
        deleted = next(float(t) for t, ref, new in rows if ref == "refs/land/lock-v2" and set(new) == {"0"})
        self.assertLess(deleted - pushed, 2.5, rows)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_lock_delete_that_keeps_failing_is_logged(self):
        repo = self.clone("lane")
        path, _ = self.slow_beats_and_timed_refs(fail_delete=True)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=path)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("could not delete the land lock after 5 tries; a waiter takes it once its beat is 20s old", r.stderr)

    def test_a_stale_threshold_under_120s_is_raised_to_120(self):
        # CPL_LAND_LOCK_STALE=30, read as minutes, let consumers' land take 4 live lands' locks.
        repo = self.clone("lane")
        held = self.lock(repo, age=60)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_STALE="30", CPL_LAND_LOCK_WAIT="2")
        self.assertNotEqual(r.returncode, 0, r.stderr)
        self.assertIn("CPL_LAND_LOCK_STALE=30 is under 120 s", r.stderr)
        self.assertNotIn("taking over", r.stderr)
        self.assertTrue(self.lock_on_remote().startswith(held), self.lock_on_remote())

    def test_a_beat_whose_reply_is_lost_keeps_the_lock_beating(self):
        # The first beat lands but answers an error, as a 504 after the update does.
        repo = self.clone("lane")
        shim, log, lost = self.tmp / "bin", self.tmp / "beats", self.tmp / "lost"
        shim.mkdir()
        real = shutil.which("git")
        (shim / "git").write_text(
            "#!/bin/sh\n"
            'case "$*" in *" HEAD:main"*) sleep 5 ;; esac\n'
            'case "$*" in *"--force-with-lease=refs/land/lock-v2:"[0-9a-f]*" "[0-9a-f]*":refs/land/lock-v2"*)\n'
            f'  "{real}" "$@"; rc=$?\n'
            f'  [ -e "{lost}" ] || {{ : >"{lost}"; exit 1; }}\n'
            f'  echo "$rc" >>"{log}"; exit $rc ;;\n'
            "esac\n"
            f'exec "{real}" "$@"\n')
        (shim / "git").chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=f"{shim}:{os.environ['PATH']}", CPL_LAND_LOCK_BEAT="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue(lost.exists())
        self.assertIn("0", log.read_text().split() if log.exists() else [], r.stderr)
        self.assertNotIn("heartbeat failed", r.stderr)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_holder_restamps_its_lock_while_it_holds_it(self):
        # A live holder's lock never ages past the threshold, however long its push takes.
        repo = self.clone("lane")
        shim, log = self.tmp / "bin", self.tmp / "beats"
        shim.mkdir()
        real = shutil.which("git")
        (shim / "git").write_text(
            "#!/bin/sh\n"
            'case "$*" in *" HEAD:main"*) sleep 4 ;; esac\n'
            'case "$*" in *"--force-with-lease=refs/land/lock-v2:"[0-9a-f]*" "[0-9a-f]*":refs/land/lock-v2"*) '
            f'echo beat >>"{log}" ;; esac\n'
            f'exec "{real}" "$@"\n')
        (shim / "git").chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=f"{shim}:{os.environ['PATH']}", CPL_LAND_LOCK_BEAT="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertGreaterEqual(len(log.read_text().split()) if log.exists() else 0, 2)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_current_land_parks_an_old_copys_lock_and_old_copies_wait_on_it(self):
        # Copies from before the heartbeat read only refs/land/lock and take any lock
        # stamped over 120 s ago; parked with a stamp in 2100, it is never theirs.
        repo, old_lane = self.clone("lane"), self.clone("old")
        empty = git(repo, "hash-object", "-t", "tree", "/dev/null").strip()
        old = git(repo, "commit-tree", empty, "-m", f"land-lock devin-box 1 {int(time.time())} compiler").strip()
        git(repo, "push", "-q", "origin", f"{old}:refs/land/lock")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("a land.sh from before c457bd860 holds refs/land/lock (land-lock devin-box 1 ", r.stderr)
        parked = git(repo, "ls-remote", "origin", "refs/land/lock").split()[0]
        git(repo, "fetch", "-q", "origin", "refs/land/lock")
        self.assertEqual(git(repo, "log", "-1", "--format=%s", parked).strip(),
                         "land-lock retired-use-lock-v2 0 4102444800 factory")
        # c457bd860's parent is the last copy without the heartbeat; CI's shallow clone fetches it.
        pre_beat = "524026262e6d18249bbdb447c2674f27d32fe0a9"
        if subprocess.run(["git", "cat-file", "-e", f"{pre_beat}^{{commit}}"], cwd=HERE).returncode:
            subprocess.run(["git", "fetch", "-q", "--depth=1", "origin", pre_beat], cwd=HERE, check=True)
        (old_lane / "scripts" / "land.sh").write_bytes(subprocess.run(
            ["git", "show", f"{pre_beat}:scripts/land.sh"], cwd=HERE, capture_output=True, check=True).stdout)
        (old_lane / "pkg2.txt").write_text("y\n")
        self.commit(old_lane, "old copy's land")
        main = git(repo, "ls-remote", "origin", "main").split()[0]
        r = self.land(old_lane, CPL_LAND_LOCK_STALE="1", CPL_LAND_LOCK_WAIT="3")
        self.assertNotEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("taking over", r.stderr)
        self.assertIn("waiting for the land lock (land-lock retired-use-lock-v2", r.stderr)
        self.assertEqual(git(repo, "ls-remote", "origin", "main").split()[0], main)
        self.assertEqual(git(repo, "ls-remote", "origin", "refs/land/lock").split()[0], parked)
        # A second current land sees it parked and says nothing.
        git(repo, "pull", "-q", "--rebase", "origin", "main")
        (repo / "pkg3.txt").write_text("z\n")
        self.commit(repo, "ours again")
        r = self.land(repo)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("before c457bd860", r.stderr)

    def beat_shim(self, body):
        shim = self.tmp / "bin"
        shim.mkdir()
        (shim / "git").write_text(
            "#!/bin/sh\n"
            f'case "$*" in *--force-with-lease=refs/land/lock-v2:\\ *) touch "{shim}/locked" ;; esac\n'
            f'case "$*" in "fetch -q origin main") [ ! -e "{shim}/locked" ] || sleep 5 ;; esac\n'
            'case "$*" in *"--force-with-lease=refs/land/lock-v2:"[0-9a-f]*" "[0-9a-f]*":refs/land/lock-v2"*)\n'
            f"{body}\n"
            ";; esac\n"
            f'exec "{shutil.which("git")}" "$@"\n')
        (shim / "git").chmod(0o755)
        return shim

    def test_a_beat_the_proxy_refuses_retries_in_seconds_and_lands(self):
        repo = self.clone("lane")
        shim = self.beat_shim('[ -e "$0.failed" ] || { touch "$0.failed"; echo "error: HTTP 429" >&2; exit 1; }')
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=f"{shim}:{os.environ['PATH']}", CPL_LAND_LOCK_BEAT="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("a land lock heartbeat failed (error: HTTP 429); retrying", r.stderr,
                      r.stdout + str(sorted(p.name for p in shim.iterdir())))
        self.assertNotIn("lost the land lock", r.stderr)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_beat_that_finds_another_holder_never_pushes_main(self):
        repo, rival = self.clone("lane"), self.clone("rival")
        empty = git(rival, "hash-object", "-t", "tree", "/dev/null").strip()
        theirs = git(rival, "commit-tree", empty, "-m", f"land-lock rival 1 {int(time.time())} infra").strip()
        real = shutil.which("git")
        shim = self.beat_shim(f'"{real}" -C "{rival}" push -q -f origin "{theirs}:refs/land/lock-v2" >>"{self.tmp}/rival.log" 2>&1; '
                              'echo "error: HTTP 429" >&2; exit 1')
        main = git(repo, "ls-remote", "origin", "main").split()[0]
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=f"{shim}:{os.environ['PATH']}", CPL_LAND_LOCK_BEAT="1", CPL_LAND_LOCK_WAIT="2")
        self.assertNotEqual(r.returncode, 0, r.stderr + (self.tmp / "rival.log").read_text())
        self.assertIn(f"lost the land lock (now {theirs})", r.stderr)
        self.assertEqual(git(repo, "ls-remote", "origin", "main").split()[0], main)
        self.assertTrue(self.lock_on_remote().startswith(theirs), self.lock_on_remote())

    def ticket(self, repo, name, age=0):
        empty = git(repo, "hash-object", "-t", "tree", "/dev/null").strip()
        env = dict(os.environ)
        stamp = subprocess.run(["date", "+%s"], capture_output=True, text=True).stdout.strip()
        env["GIT_COMMITTER_DATE"] = f"{int(stamp) - age} +0000"
        sha = subprocess.run(["git", "commit-tree", empty, "-m", "land ticket other"], cwd=repo, env=env,
                             capture_output=True, text=True, check=True).stdout.strip()
        git(repo, "push", "-q", "origin", f"+{sha}:refs/locks/queue/{name}")

    def tickets_on_remote(self, repo):
        out = git(repo, "ls-remote", "origin", "refs/locks/queue/*")
        return [line.split("/")[-1] for line in out.splitlines()]

    def test_an_earlier_live_ticket_takes_its_turn_first(self):
        repo = self.clone("lane")
        self.ticket(repo, "1000000000000000000-1-other")
        self.ticket(repo, "9000000000000000000-1-later")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        # The lock is free, but an earlier waiter is in line: ours waits its turn.
        r = self.land(repo, CPL_LAND_LOCK_WAIT="2")
        self.assertNotEqual(r.returncode, 0, r.stderr)
        self.assertIn("waiting in line behind 1000000000000000000-1-other", r.stderr)
        self.assertEqual(self.lock_on_remote(), "")
        self.assertEqual(self.tickets_on_remote(repo), ["1000000000000000000-1-other", "9000000000000000000-1-later"])
        # Once the earlier waiter has landed, ours goes ahead of the later one.
        git(repo, "push", "-q", "origin", ":refs/locks/queue/1000000000000000000-1-other")
        r = self.land(repo, CPL_LAND_LOCK_WAIT="2")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("pushed", r.stdout)
        self.assertEqual(self.tickets_on_remote(repo), ["9000000000000000000-1-later"])
        self.assertEqual(self.lock_on_remote(), "")

    def test_losing_the_lock_to_churn_is_not_a_refusal(self):
        # A lane that takes and drops the lock between our push and our look
        # leaves no holder; my land read 2 of those as "origin refuses" and
        # pushed without the lock.
        repo = self.clone("lane")
        shim = self.tmp / "shim"
        shim.mkdir()
        (shim / "git").write_text(f"""#!/bin/sh
case "$*" in *--force-with-lease=refs/land/lock-v2:\\ *|*--force-with-lease=refs/land/lock-v2:) 
    n=$(cat "{shim}/n" 2>/dev/null || echo 0)
    if [ "$n" -lt 6 ]; then
        echo $((n + 1)) >"{shim}/n"
        echo " ! [rejected]        x -> refs/land/lock-v2 (stale info)" >&2
        exit 1
    fi ;;
esac
exec {shutil.which("git")} "$@"
""")
        (shim / "git").chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=f"{shim}:{os.environ['PATH']}")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual((shim / "n").read_text(), "6\n")
        self.assertNotIn("refuses", r.stderr)

    def test_a_dead_waiters_ticket_expires(self):
        # Dead means its sha stopped changing (no heartbeat), by the waiter's clock.
        repo = self.clone("lane")
        self.ticket(repo, "1000000000000000000-1-dead")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_TICKET_STALE="2")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("dropping queue ticket 1000000000000000000-1-dead: no heartbeat for over 2s", r.stderr)
        self.assertEqual(self.tickets_on_remote(repo), [])

    def test_a_malformed_ticket_is_dropped(self):
        repo = self.clone("lane")
        self.ticket(repo, "1795545766843135466-380144-")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("dropping queue ticket 1795545766843135466-380144-: malformed name", r.stderr)
        self.assertEqual(self.tickets_on_remote(repo), [])

    def test_a_live_waiter_in_a_long_line_never_times_out(self):
        repo = self.clone("lane")
        self.ticket(repo, "1000000000000000000-1-other")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        env = {**{k: v for k, v in os.environ.items() if not k.startswith("CPL_LAND")},
               "CPL_LAND_QUEUE": "1", "CPL_LAND_TRIES": "1", "CPL_LAND_HEARTBEAT": "2"}
        land = subprocess.Popen(["bash", "scripts/land.sh"], cwd=repo, env=env,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        for line in land.stderr:
            if "waiting in line behind 1000000000000000000-1-other" in line:
                break
        # Ours heartbeats its ticket (every CPL_LAND_HEARTBEAT s) and keeps waiting, past what
        # lock_wait would have allowed, until the earlier waiter is gone.
        ours = lambda: [l.split() for l in git(repo, "ls-remote", "origin", "refs/locks/queue/*").splitlines()
                        if "-other" not in l]
        first = ours()
        while ours() == first:
            self.assertIsNone(land.poll())
        self.ticket(repo, "1000000000000000000-1-other")  # the earlier waiter's heartbeat
        git(repo, "push", "-q", "origin", ":refs/locks/queue/1000000000000000000-1-other")
        out, err = land.communicate(timeout=120)
        self.assertEqual(land.returncode, 0, err)
        self.assertIn("pushed", out)
        self.assertNotIn("no land lock", err)
        self.assertNotIn("ahead in line", err)
        self.assertEqual(self.tickets_on_remote(repo), [])

    def test_without_cpl_land_queue_a_land_ignores_the_line(self):
        repo = self.clone("lane")
        self.ticket(repo, "1000000000000000000-1-other")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_QUEUE="")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("in line", r.stderr)
        self.assertEqual(self.tickets_on_remote(repo), ["1000000000000000000-1-other"])

    def test_a_live_head_that_leaves_the_lock_free_loses_its_place(self):
        repo = self.clone("lane")
        stuck = "1000000000000000000-1-stuck"
        self.ticket(repo, stuck)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        env = {**{k: v for k, v in os.environ.items() if not k.startswith("CPL_LAND")},
               "CPL_LAND_QUEUE": "1", "CPL_LAND_TRIES": "1", "CPL_LAND_TICKET_STALE": "3"}
        land = subprocess.Popen(["bash", "scripts/land.sh"], cwd=repo, env=env,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        err = []
        reader = threading.Thread(target=lambda: err.extend(land.stderr))
        reader.start()
        # The stuck lane heartbeats (so it never expires as dead) but never locks.
        while land.poll() is None and not any("left the land lock free" in l for l in err):
            self.ticket(repo, stuck)
        out = land.stdout.read()
        land.wait(timeout=120)
        reader.join()
        self.assertEqual(land.returncode, 0, "".join(err))
        self.assertIn(f"land: dropping queue ticket {stuck}: first in line but left the land lock free", "".join(err))
        self.assertIn("pushed", out)

    def test_a_waiter_far_back_in_line_polls_less(self):
        # Compiler's 58-waiter queue at 2 s polls drew GitHub 504s.
        repo = self.clone("lane")
        for i in range(3):
            self.ticket(repo, f"100000000000000000{i}-1-other")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_WAIT="2", CPL_LAND_QUEUE_POLL="1")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("land: 3 ahead in line; looking every 1s", r.stderr)

    def test_a_rejected_push_tries_again_holding_the_lock_and_its_place(self):
        repo = self.clone("lane")
        rejected = self.tmp / "rejected"
        hook = self.tmp / "remote.git" / "hooks" / "pre-receive"
        hook.write_text("#!/bin/sh\nwhile read old new ref; do\n"
                        f'  [ "$ref" != refs/heads/main ] || [ -e "{rejected}" ] || '
                        f'{{ touch "{rejected}"; echo failed >&2; exit 1; }}\n'
                        "done\n")
        hook.chmod(0o755)
        self.ticket(repo, "9000000000000000000-1-later")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_WAIT="2")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue(rejected.exists())
        self.assertIn("the push failed; trying again holding the land lock", r.stderr)
        self.assertIn("(try 1)", r.stdout)
        self.assertEqual(self.lock_on_remote(), "")
        self.assertEqual(self.tickets_on_remote(repo), ["9000000000000000000-1-later"])

    def test_a_land_whose_lock_delete_failed_takes_its_own_lock_back(self):
        # idioms' main push got three HTTP 500s, then its lock delete did too: it waited
        # 159 s on its own stale lock with nobody holding it.
        repo = self.clone("lane")
        n = self.tmp / "n"
        hook = self.tmp / "remote.git" / "hooks" / "pre-receive"
        hook.write_text("#!/bin/sh\nwhile read old new ref; do\n"
                        f'  k="{n}.$(echo "$ref" | tr / _)"; echo x >>"$k"\n'
                        f'  case "$ref $new" in refs/heads/main*) [ "$(wc -l <"$k")" -gt 3 ] && touch "$k.ok" || '
                        "{ echo 'Internal Server Error' >&2; exit 1; } ;;\n"
                        f'  "refs/land/lock-v2 0000000000000000000000000000000000000000") [ -e "{n}.refs_heads_main.ok" ] || '
                        "{ echo 'Internal Server Error' >&2; exit 1; } ;; esac\n"
                        "done\n")
        hook.chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_TRIES="2", CPL_LAND_LOCK_WAIT="5")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("took back this land's own lock", r.stderr)
        self.assertNotIn("no land lock", r.stderr)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_lock_whose_holder_cant_be_read_is_never_taken_over(self):
        repo = self.clone("lane")
        empty = git(repo, "hash-object", "-t", "tree", "/dev/null").strip()
        held = git(repo, "commit-tree", empty, "-m", "land-lock other 1 unread infra").strip()
        git(repo, "push", "-q", "origin", f"{held}:refs/land/lock-v2")
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, CPL_LAND_LOCK_STALE="1", CPL_LAND_LOCK_WAIT="2")
        self.assertNotEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("taking over", r.stderr)
        self.assertIn("waiting for the land lock (land-lock other 1 unread infra; no readable stamp)", r.stderr)
        self.assertIn("no land lock after", r.stderr)
        self.assertTrue(self.lock_on_remote().startswith(held), self.lock_on_remote())

    def test_two_lanes_cant_both_take_over_one_stale_lock(self):
        repo, rival = self.clone("lane"), self.clone("rival")
        self.lock(repo, age=1000)
        empty = git(rival, "hash-object", "-t", "tree", "/dev/null").strip()
        theirs = git(rival, "commit-tree", empty, "-m", f"land-lock rival 2 {int(time.time())} infra").strip()
        # The rival takes the stale lock over between this lane's look and its takeover.
        shim = self.tmp / "bin"
        shim.mkdir()
        real = shutil.which("git")
        (shim / "git").write_text(
            "#!/bin/sh\n"
            'case "$*" in *"--force-with-lease=refs/land/lock-v2:"[0-9a-f]*)\n'
            f'  [ -e "{self.tmp}/rival.done" ] || {{ touch "{self.tmp}/rival.done"; '
            f'"{real}" -C "{rival}" push -q -f origin "{theirs}:refs/land/lock-v2"; }} ;;\n'
            "esac\n"
            f'exec "{real}" "$@"\n')
        (shim / "git").chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=f"{shim}:{os.environ['PATH']}", CPL_LAND_LOCK_WAIT="2", CPL_LAND_LOCK_BEAT="1")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("taking over a land lock", r.stderr)
        self.assertIn("waiting for the land lock (land-lock rival", r.stderr)
        self.assertTrue(self.lock_on_remote().startswith(theirs))

    def test_lost_lock_swaps_on_the_server_are_races_not_refusals(self):
        # Six times another lane makes the lock between our look and our write,
        # then drops it before our next look: the server's swap fails.
        repo = self.clone("lane")
        shim, count = self.tmp / "bin", self.tmp / "swaps"
        shim.mkdir()
        (shim / "git").write_text(
            "#!/bin/sh\n"
            'case "$*" in *"--force-with-lease=refs/land/lock-v2: "*)\n'
            f'  if [ "$(wc -c <"{count}" 2>/dev/null || echo 0)" -lt 6 ]; then\n'
            f'    printf x >>"{count}"\n'
            "    echo \"error: cannot lock ref 'refs/land/lock-v2': reference already exists\" >&2\n"
            "    exit 1\n"
            "  fi ;;\n"
            "esac\n"
            f'exec "{shutil.which("git")}" "$@"\n')
        (shim / "git").chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=f"{shim}:{os.environ['PATH']}")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("refuses", r.stderr)
        self.assertEqual(count.read_text(), "xxxxxx")

    def test_a_refused_land_drops_the_land_lock(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("wasm\n")
        r = self.land(ours, CI_RUNS=str(runs))
        self.assertNotEqual(r.returncode, 0)
        self.assertEqual(self.lock_on_remote(), "")

    def test_the_pending_steps_run_once_through_lost_races_then_land(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        git(theirs, "pull", "-q", "origin", "main")
        (theirs / "theirs.txt").write_text("x\n")
        self.commit(theirs, "theirs")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("nextest\n")
        r = self.land(ours, CI_RUNS=str(runs), RACE=str(theirs), RACES="2", CPL_LAND_TRIES="3")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(runs.read_text(), "commit --only nextest\n" )
        self.assertIn("landing on the skipped steps' rerun", r.stderr)

    def test_lost_races_rebase_and_recheck_without_the_lock(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        locks = self.tmp / "locks"
        git(theirs, "pull", "-q", "origin", "main")
        (theirs / "theirs.txt").write_text("x\n")
        self.commit(theirs, "theirs")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("nextest\n")
        r = self.land(ours, CI_RUNS=str(runs), RACE=str(theirs), RACES="2", CPL_LAND_TRIES="3", LOCKS=str(locks))
        self.assertEqual(r.returncode, 0, r.stderr)
        # Every rerun ran with the lock free; the land took it only to push.
        self.assertEqual(set(locks.read_text().split()), {"free"})
        self.assertNotIn("holding the land lock", r.stderr)
        self.assertIn("(try 1)", r.stdout)
        self.assertEqual(self.lock_on_remote(), "")
        self.assertIn("theirs", git(ours, "log", "--format=%s", "origin/main"))

    def test_main_moving_during_the_lock_wait_rebases_holding_the_lock(self):
        # Compiler, idioms and consumers each lost 5 of 5 tries: whoever held the lock
        # pushed, so main had moved by the time theirs came, and they let it go.
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "theirs.txt").write_text("x\n")
        self.commit(theirs, "theirs")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        self.lock(theirs)

        # The holder pushes and lets go once ours is in line, from the remote's hook.
        hook = self.tmp / "remote.git" / "hooks" / "post-receive"
        hook.write_text(f"""#!/bin/sh
unset GIT_DIR
while read old new ref; do
    case "$ref" in refs/locks/queue/*)
        mkdir "{self.tmp}/held" 2>/dev/null || continue
        (cd "{theirs}" && git push -q origin HEAD:main && git push -q origin :refs/land/lock-v2) </dev/null >/dev/null 2>&1 & ;;
    esac
done
""")
        hook.chmod(0o755)
        r = self.land(ours, CPL_LAND_LOCK_WAIT="60")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("rebasing holding it", r.stderr)
        self.assertIn("(try 1)", r.stdout)
        self.assertEqual(self.lock_on_remote(), "")
        self.assertIn("theirs", git(ours, "log", "--format=%s", "origin/main"))

    def test_a_lanes_land_leaves_another_worktrees_pending_steps_alone(self):
        a = self.clone("ours")
        _, runs = self._steps_runner(a)
        b = self.tmp / "b"
        git(a, "worktree", "add", "-q", "-b", "b", str(b), "HEAD")
        pending_b = Path(git(b, "rev-parse", "--absolute-git-dir").strip()) / "cpl-pending-steps"
        pending_b.write_text("nextest\n")
        (a / "a.txt").write_text("x\n")
        self.commit(a, "a")
        r = self.land(a, CI_RUNS=str(runs))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertFalse(runs.exists())
        self.assertTrue(pending_b.exists())
        (b / "b.txt").write_text("x\n")
        self.commit(b, "b")
        r = self.land(b, CI_RUNS=str(runs))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(runs.read_text(), "commit --only nextest\n")
        self.assertFalse(pending_b.exists())

    def test_a_std_change_runs_the_traits_gate_and_the_signature_callers(self):
        ours = self.clone("ours")
        (ours / "scripts" / "spec-gate.py").write_text(
            "import sys, pathlib\nsys.exit(1 if 'Bad' in pathlib.Path('std/a.cp').read_text() else 0)\n")
        (ours / "scripts" / "affected.py").write_text(
            "print('spec/conformance/c.cp')\nprint('corpus/p.cp')\n")
        (ours / "scripts" / "conform_staged.py").write_text(
            "import sys\nprint('cases', *sys.argv[1:], file=sys.stderr)\n")
        (ours / "spec" / "conformance").mkdir(parents=True, exist_ok=True)
        (ours / "spec" / "conformance" / "c.cp").write_text("x\n")
        self.commit(ours, "gates")
        git(ours, "push", "-q", "origin", "HEAD:main")
        (ours / "std").mkdir(exist_ok=True)
        (ours / "std" / "a.cp").write_text("impl Bad for X:\n")
        self.commit(ours, "a std impl with no case")
        r = self.land(ours)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("no spec/conformance/traits.toml case", r.stderr)
        (ours / "std" / "a.cp").write_text("pub fn f(mut x: Int):\n")
        self.commit(ours, "a signature")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertRegex(r.stderr, r"cases --base [0-9a-f]{40} --build spec/conformance/c.cp")
        self.assertRegex(r.stderr, r"cases --base [0-9a-f]{40} --check corpus/p.cp")

    def test_a_fetch_mid_land_leaves_the_std_diff_on_the_pinned_base(self):
        # VM-expr's interp-only series ran the std signature callers: a fetch in its
        # worktree mid-land moved FETCH_HEAD past main's newer std commits.
        ours, theirs = self.clone("ours"), self.clone("theirs")
        (ours / "scripts" / "affected.py").write_text(
            "import os, subprocess, sys\n"
            "if sys.argv[1] == '--build-slices':\n"
            f"    t = {str(theirs)!r}\n"
            "    subprocess.run(['git', 'pull', '-q', 'origin', 'main'], cwd=t, check=True)\n"
            "    os.makedirs(t + '/std', exist_ok=True)\n"
            "    open(t + '/std/b.cp', 'w').write('pub fn g(x: Int):\\n')\n"
            "    subprocess.run(['git', 'add', '-A'], cwd=t, check=True)\n"
            "    subprocess.run(['git', 'commit', '-q', '--no-verify', '-m', 'std', '-m', 'Domain: infra\\nLane: G'],\n"
            "                   cwd=t, check=True)\n"
            "    subprocess.run(['git', 'push', '-q', 'origin', 'HEAD:main'], cwd=t, check=True)\n"
            "    subprocess.run(['git', 'fetch', '-q', 'origin', 'main'], check=True)\n"
            "elif sys.argv[1] == '--sig-callers':\n"
            "    print('corpus/p.cp')\n")
        (ours / "scripts" / "conform_staged.py").write_text(
            "import sys\nprint('cases', *sys.argv[1:], file=sys.stderr)\n")
        self.commit(ours, "an interp-only change")
        (ours / ".git" / "cpl-pending-slices").write_text("x\n")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("touched-path slices", r.stderr)
        self.assertNotIn("changed std signature", r.stderr)
        git(ours, "fetch", "-q")
        self.assertIn("std/b.cp", git(ours, "ls-tree", "-r", "--name-only", "origin/main"))

    def test_pending_cases_refuse_uncommitted_compiler_changes(self):
        ours = self.clone("ours")
        (ours / "crates").mkdir()
        (ours / "crates" / "a.rs").write_text("fn a() {}\n")
        (ours / "c.cp").write_text("x\n")
        self.commit(ours, "a crate and a case")
        (ours / "crates" / "a.rs").write_text("fn b() {}\n")
        (ours / ".git" / "cpl-pending-cases").write_text("c.cp\n")
        r = self.land(ours)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("uncommitted changes under crates/ or std/", r.stderr)
        git(ours, "fetch", "-q")
        self.assertEqual(git(ours, "ls-tree", "origin/main", "c.cp"), "")

    def test_a_rebase_past_no_gate_change_lands_without_rerunning_it(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        self._hook(theirs, "echo gate ran; exit 1\n")
        self.commit(theirs, "gate")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "--rebase", "origin", "main")
        (theirs / "other.txt").write_text("y\n")
        self.commit(theirs, "not a gate")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "src.txt").write_text("source\n")
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("gate ran", r.stderr + r.stdout)

    def test_a_hook_failure_in_a_crate_main_changed_says_to_rebase(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        lib = Path("crates/cpl-x/src/lib.rs")
        (theirs / lib).parent.mkdir(parents=True)
        (theirs / lib).write_text("fixed\n")
        self.commit(theirs, "fix x")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "fetch", "-q", "origin")
        (ours / ".githooks").mkdir()
        shutil.copy(HERE.parent / ".githooks" / "pre-commit", ours / ".githooks" / "pre-commit")
        (ours / "scripts" / "check.sh").write_text("#!/bin/sh\necho 'error: unused  --> crates/cpl-x/src/lib.rs:1:1'\nexit 1\n")
        (ours / "scripts" / "check.sh").chmod(0o755)
        r = subprocess.run([str(ours / ".githooks" / "pre-commit")], cwd=ours, capture_output=True, text=True)
        self.assertEqual(r.returncode, 1)
        self.assertIn("behind origin/main in crates/cpl-x", r.stderr)

    def test_a_ratchet_failure_on_a_branch_behind_main_says_to_rebase(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "later.txt").write_text("x\n")
        self.commit(theirs, "later")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "fetch", "-q", "origin")
        (ours / ".githooks").mkdir()
        shutil.copy(HERE.parent / ".githooks" / "pre-commit", ours / ".githooks" / "pre-commit")
        (ours / "scripts" / "check.sh").write_text("#!/bin/sh\necho 'debt-ratchet: ERROR a: increased from 5 to 6'\nexit 1\n")
        (ours / "scripts" / "check.sh").chmod(0o755)
        r = subprocess.run([str(ours / ".githooks" / "pre-commit")], cwd=ours, capture_output=True, text=True)
        self.assertEqual(r.returncode, 1)
        self.assertIn("1 commits behind origin/main", r.stderr)

    def test_a_ratchet_conflict_keeps_both_sides_tightening(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / RATCHET).write_text(json.dumps({"a": 3, "b": 5, "names": ["x"]}, indent=2) + "\n")
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / RATCHET).write_text(json.dumps({"a": 5, "b": 4, "names": ["x", "y"]}, indent=2) + "\n")
        (ours / "src.txt").write_text("source\n")
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(json.loads((ours / RATCHET).read_text()), {"a": 3, "b": 4, "names": ["x"]})
        self.assertEqual(git(ours, "status", "--porcelain"), "")
        self.assertIn("src.txt", git(ours, "show", "--name-only", "--format=", "origin/main"))

    def rule(self, glue):
        """A debt-ratchet.py whose `lines` counter counts src.txt, and glue.txt with `glue`."""
        files = "('src.txt', 'glue.txt')" if glue else "('src.txt',)"
        return ("import sys\n"
                f"FILES = {files}\n"
                "def lines(root):\n"
                "    return [l for f in FILES for l in (root / f).read_text().splitlines() if l]\n"
                "COUNTERS = {'lines': ('', lines, True)}\n"
                "if __name__ == '__main__':\n    sys.exit(0)\n")

    def race(self, ours_changes_the_rule):
        """One side counts glue.txt too (5 src + 3 glue lines: 8); the other deletes 2 src lines (3)."""
        seed = self.clone("rule-seed")
        (seed / "src.txt").write_text("1\n2\n3\n4\n5\n")
        (seed / "glue.txt").write_text("a\nb\nc\n")
        (seed / "scripts" / "debt-ratchet.py").write_text(self.rule(glue=False))
        (seed / RATCHET).write_text(json.dumps({"a": 5, "lines": 5}, indent=2) + "\n")
        self.commit(seed, "rule seed")
        git(seed, "push", "-q", "origin", "HEAD:main")
        theirs, ours = self.clone("theirs"), self.clone("ours")
        rule_side, delete_side = (ours, theirs) if ours_changes_the_rule else (theirs, ours)
        (rule_side / "scripts" / "debt-ratchet.py").write_text(self.rule(glue=True))
        (rule_side / RATCHET).write_text(json.dumps({"a": 5, "lines": 8}, indent=2) + "\n")
        self.commit(rule_side, "count glue")
        (delete_side / "src.txt").write_text("1\n2\n3\n")
        (delete_side / RATCHET).write_text(json.dumps({"a": 5, "lines": 3}, indent=2) + "\n")
        self.commit(delete_side, "delete src lines")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        return ours, r

    def test_a_rebase_onto_a_rule_change_restates_the_count_by_the_new_rule(self):
        ours, r = self.race(ours_changes_the_rule=False)
        # The lower of 8 (new rule) and 3 (old rule) would be 3; the tip measures 6, which the
        # merge driver already gave (main's 8 less the lane's 2), so nothing is restated.
        self.assertEqual(json.loads((ours / RATCHET).read_text())["lines"], 6, r.stderr)
        self.assertEqual(json.loads(git(ours, "show", f"origin/main:{RATCHET}"))["lines"], 6)
        self.assertEqual(git(ours, "status", "--porcelain"), "")

    def test_a_rule_change_rebased_onto_a_deletion_restates_mains_count(self):
        ours, _ = self.race(ours_changes_the_rule=True)
        self.assertEqual(json.loads(git(ours, "show", f"origin/main:{RATCHET}"))["lines"], 6)

    def test_a_rebase_with_no_rule_change_keeps_the_merged_count(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        for repo in (theirs, ours):
            (repo / "scripts" / "debt-ratchet.py").write_text(self.rule(glue=False))
        (theirs / "src.txt").write_text("1\n")
        (theirs / RATCHET).write_text(json.dumps({"a": 4, "b": 5, "names": ["x", "y"]}, indent=2) + "\n")
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "src.txt").write_text("1\n")
        (ours / RATCHET).write_text(json.dumps({"a": 3, "b": 5, "names": ["x", "y"]}, indent=2) + "\n")
        self.commit(ours, "ours")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("restated", r.stderr)
        self.assertEqual(json.loads((ours / RATCHET).read_text())["a"], 3)

    def test_an_uncommitted_ratchet_edit_stays_out_of_mains_commits(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "src.txt").write_text("1\n")
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        tip = git(theirs, "rev-parse", "HEAD").strip()
        edited = json.dumps({"a": 4, "b": 5, "names": ["x", "y"]}, indent=2) + "\n"
        (ours / RATCHET).write_text(edited)
        self.land(ours)
        # With no commit of its own, a land rewrites none of main's.
        self.assertEqual(git(ours, "ls-remote", "origin", "main").split()[0], tip)
        self.assertEqual((ours / RATCHET).read_text(), edited)
        (ours / "other.txt").write_text("other\n")
        git(ours, "add", "other.txt")
        git(ours, "commit", "-q", "--no-verify", "-m", "ours", "-m", "Domain: infra\nLane: G")
        (theirs / "src.txt").write_text("2\n")
        self.commit(theirs, "theirs again")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(git(ours, "show", "--name-only", "--format=", "origin/main").split(), ["other.txt"])
        self.assertEqual((ours / RATCHET).read_text(), edited)

    def test_a_corpus_index_conflict_is_rebuilt_from_both_sides_packages(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        for repo, pkg in ((theirs, "b"), (ours, "c")):
            (repo / "corpus" / "pkgs" / pkg).mkdir()
            (repo / "corpus" / "pkgs" / pkg / "meta.json").write_text("{}\n")
            (repo / INDEX).write_text(json.dumps({"count": 2, "by": pkg}) + "\n")
            self.commit(repo, pkg)
            if repo == theirs:
                git(theirs, "push", "-q", "origin", "HEAD:main")
        r = self.land(ours)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(json.loads((ours / INDEX).read_text()), {"count": 3})
        self.assertEqual(git(ours, "status", "--porcelain"), "")

    def test_the_ratchet_merge_never_loosens(self):
        sys.path.insert(0, str(HERE))
        from ratchet_merge import merge
        base = {"n": 5, "allow": {"f.rs": 2400, "g.rs": 2100}, "edges": ["a", "b"]}
        ours = {"n": 4, "allow": {"f.rs": 2300, "g.rs": 2100}, "edges": ["a", "b", "c"]}
        theirs = {"n": 3, "allow": {"f.rs": 2400}, "edges": ["a"], "new": 1}
        self.assertEqual(merge(base, ours, theirs),
                         {"n": 3, "allow": {"f.rs": 2300}, "edges": ["a", "c"], "new": 1})
        raised = {"n": 9, "allow": {"f.rs": 2500, "g.rs": 2100}, "edges": ["a", "b"]}
        self.assertEqual(merge(base, base, raised), base)

    def test_the_ratchet_merge_keeps_a_number_main_raised(self):
        sys.path.insert(0, str(HERE))
        from ratchet_merge import merge
        base = {"old": 179917, "n": 5}
        self.assertEqual(merge(base, {"old": 213697, "n": 5}, {"old": 179917, "n": 4}), {"old": 213697, "n": 4})
        self.assertEqual(merge(base, {"old": 213697, "n": 5}, {"old": 179907, "n": 5}), {"old": 213687, "n": 5})

    def test_a_rebase_merges_the_ratchet_through_its_driver(self):
        repo = self.tmp / "drv"
        repo.mkdir()
        git(repo, "init", "-q", "-b", "main")
        git(repo, "config", "user.email", "t@example.com")
        git(repo, "config", "user.name", "t")
        (repo / "scripts").mkdir()
        for name in ("ratchet_merge.py", "merge-drivers.sh"):
            shutil.copy(HERE / name, repo / "scripts" / name)
        shutil.copy(HERE.parent / ".gitattributes", repo / ".gitattributes")
        subprocess.run(["bash", "scripts/merge-drivers.sh"], cwd=repo, check=True)
        ratchet = repo / "scripts" / "debt-ratchet.json"
        ratchet.write_text(json.dumps({"a": 5, "b": 5}, indent=2) + "\n")
        git(repo, "add", "-A")
        git(repo, "commit", "-qm", "base")
        git(repo, "checkout", "-qb", "lane")
        ratchet.write_text(json.dumps({"a": 3, "b": 5}, indent=2) + "\n")
        git(repo, "commit", "-qam", "lane tightens a")
        git(repo, "checkout", "-q", "main")
        ratchet.write_text(json.dumps({"a": 5, "b": 4}, indent=2) + "\n")
        git(repo, "commit", "-qam", "main tightens b")
        git(repo, "checkout", "-q", "lane")
        git(repo, "rebase", "-q", "main")
        self.assertEqual(json.loads(ratchet.read_text()), {"a": 3, "b": 4})

    def test_the_ratchet_merge_keeps_both_moves_of_not_passing(self):
        sys.path.insert(0, str(HERE))
        from ratchet_merge import merge
        # One side adds 11 cases the new pipeline refuses, the other passes 30.
        self.assertEqual(merge({"next_not_passing": 4146}, {"next_not_passing": 4116}, {"next_not_passing": 4157}),
                         {"next_not_passing": 4127})

    def _wasm(self, ours, node_exit):
        """A fake node that logs its arguments, a cargo whose fmt passes, and a target dir."""
        bin_dir = self.tmp / "bin"
        bin_dir.mkdir(exist_ok=True)
        (bin_dir / "node").write_text(f'#!/bin/sh\necho "node $*" >> "{self.tmp}/node.log"\nexit {node_exit}\n')
        (bin_dir / "node").chmod(0o755)
        (bin_dir / "cargo").write_text('#!/bin/sh\n[ "$1" != metadata ] || echo \'{"packages": []}\'\n')
        (bin_dir / "cargo").chmod(0o755)
        return {"PATH": f"{bin_dir}{os.pathsep}{os.environ['PATH']}", "CARGO_TARGET_DIR": str(self.tmp / "tgt")}

    def _built(self, ours, parent=True):
        """A warm wasm build of ours' tree rebased on main's tip, and the sizes of main's tip."""
        out = self.tmp / "tgt" / "wasm-warm"
        out.mkdir(parents=True)
        (out / "cpl.wasm").write_text("wasm")
        git(ours, "fetch", "-q", "origin")
        git(ours, "rebase", "-q", "origin/main")
        (out / "tree").write_text(git(ours, "rev-parse", "HEAD^{tree}"))
        if parent:
            (out / "sizes").mkdir()
            (out / "sizes" / f'{git(ours, "rev-parse", "origin/main^{tree}").strip()}.json').write_text('{"fns": {}}')
        return out

    def _crate_change(self, theirs_crate):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        (theirs / "crates" / theirs_crate).mkdir(parents=True)
        (theirs / "crates" / theirs_crate / "main.rs").write_text("theirs\n")
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "crates" / "cpl-x").mkdir(parents=True)
        (ours / "crates" / "cpl-x" / "lib.rs").write_text("ours\n")
        self.commit(ours, "ours")
        return ours

    def test_a_touched_crate_over_its_wasm_budget_in_the_warm_build_stops_the_land(self):
        ours = self._crate_change("cpl-y")
        out = self._built(ours)
        r = self.land(ours, **self._wasm(ours, 1))
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("grew past main while over its budget, or crossed its budget", r.stderr)
        self.assertIn(f"node web/scripts/wasm-crates.mjs --data {out}/cpl.wasm --parent {out}/sizes/",
                      (self.tmp / "node.log").read_text())
        self.assertNotIn("ours", git(ours, "log", "--format=%s", "origin/main"))

    def test_without_main_tips_wasm_sizes_land_builds_them_or_refuses(self):
        ours = self._crate_change("cpl-y")
        self._built(ours, parent=False)
        r = self.land(ours, **self._wasm(ours, 1))
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("could not build", r.stderr)
        self.assertNotIn("ours", git(ours, "log", "--format=%s", "origin/main"))

    def test_a_warm_build_within_budget_lands(self):
        ours = self._crate_change("cpl-y")
        self._built(ours)
        r = self.land(ours, **self._wasm(ours, 0))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("--parent", (self.tmp / "node.log").read_text())

    def _ratchet(self, ours_gz, parent_gz, budgets, ours_budgets=None, ours_fns=None, parent_fns=None):
        """A land of a cpl-x change through the real wasm-crates.mjs: a warm build measuring
        `ours_gz` and main's tip `parent_gz` (crate -> B gz, normalized code `*_fns`), under
        main's `budgets`, which the change sets to `ours_budgets`."""
        data = "data (std source, strings)"
        budgets, ours_gz, parent_gz = {data: 100, **budgets}, {data: 90, **ours_gz}, {data: 90, **parent_gz}
        theirs, ours = self.clone("theirs"), self.clone("ours")
        web = theirs / "web" / "scripts"
        web.mkdir(parents=True)
        shutil.copy(HERE.parent / "web" / "scripts" / "wasm-crates.mjs", web)
        (web / "wasm-crate-budgets.json").write_text(json.dumps(budgets))
        (web / "wasm-sort-copies.json").write_text('{"max": 0}\n')
        shutil.copytree(HERE.parent / "handbook" / "domains", theirs / "handbook" / "domains")
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        (ours / "crates" / "cpl-x").mkdir(parents=True)
        (ours / "crates" / "cpl-x" / "lib.rs").write_text("ours\n")
        if ours_budgets is not None:
            git(ours, "pull", "-q", "--rebase", "origin", "main")
            (ours / "web" / "scripts" / "wasm-crate-budgets.json").write_text(json.dumps({data: 100, **ours_budgets}))
        self.commit(ours, "ours")
        out = self._built(ours)
        (out / "cpl.wasm").write_text(sized(ours_gz, ours_fns))
        (next((out / "sizes").iterdir())).write_text(sized(parent_gz, parent_fns))
        env = self._wasm(ours, 0)
        (self.tmp / "bin" / "node").unlink()
        return self.land(ours, **env)

    def test_the_wasm_ratchet_refuses_growth_of_a_crate_the_change_never_touched(self):
        # ecd92f8e6..b434e93d3 grew cpl_check and cpl_syntax past their budgets in lands
        # that touched other crates, which only warned.
        r = self._ratchet({"cpl_x": 50, "cpl_y": 151}, {"cpl_x": 50, "cpl_y": 150}, {"cpl_x": 100, "cpl_y": 100})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("cpl_y is 151 B gzipped, over its budget of 100", r.stderr)
        self.assertIn("it grew 1 B against its parent's 150 B", r.stderr)

    def test_the_wasm_ratchet_refuses_a_touched_crate_over_budget_that_grew_a_byte(self):
        r = self._ratchet({"cpl_x": 151}, {"cpl_x": 150}, {"cpl_x": 100})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("it grew 1 B against its parent's 150 B", r.stderr)

    def test_the_wasm_ratchet_lands_a_crate_over_budget_that_shrank(self):
        r = self._ratchet({"cpl_x": 149, "cpl_y": 150}, {"cpl_x": 150, "cpl_y": 150}, {"cpl_x": 100, "cpl_y": 100})
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("cpl_x is 149 B gzipped, over its budget of 100", r.stderr)
        self.assertIn("but its normalized code is no larger than its parent's", r.stderr)

    def test_the_wasm_ratchet_lands_a_crate_over_budget_that_held_its_size(self):
        r = self._ratchet({"cpl_x": 150}, {"cpl_x": 150}, {"cpl_x": 100})
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("but its normalized code is no larger than its parent's", r.stderr)

    def test_the_wasm_ratchet_keeps_a_crate_within_budget_within_it(self):
        r = self._ratchet({"cpl_x": 50, "cpl_y": 101}, {"cpl_x": 50, "cpl_y": 90}, {"cpl_x": 100, "cpl_y": 100})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("cpl_y is 101 B gzipped, over its budget of 100", r.stderr)
        self.assertNotIn("ours", git(self.tmp / "remote.git", "log", "--format=%s", "main"))

    def _transfer(self, ours_budgets):
        # VM-memory's narrow storage moved IntWidth's 60 B of code from cpl_x to cpl_y,
        # which is over its budget (decisions.md 634).
        return self._ratchet({"cpl_x": 40, "cpl_y": 140}, {"cpl_x": 90, "cpl_y": 120}, {"cpl_x": 100, "cpl_y": 100},
                             ours_budgets=ours_budgets, ours_fns={"cpl_x": {"f": 30}, "cpl_y": {"f": 120, "w": 60}},
                             parent_fns={"cpl_x": {"f": 30, "w": 60}, "cpl_y": {"f": 120}})

    def test_a_budget_transferred_with_the_code_that_moved_lands(self):
        r = self._transfer({"cpl_x": 50, "cpl_y": 150})
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_without_its_budget_transfer_a_move_into_an_over_budget_crate_refuses(self):
        r = self._transfer({"cpl_x": 100, "cpl_y": 100})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("cpl_y is 140 B gzipped, over its budget of 100", r.stderr)
        self.assertIn("grew most: w +60", r.stderr)

    def test_a_budget_raise_with_no_matching_cut_refuses(self):
        r = self._transfer({"cpl_x": 100, "cpl_y": 150})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("raises cpl_y +50 (50 B) but lowers others by only 0 B", r.stderr)

    def test_a_budget_transfer_bigger_than_the_code_that_moved_refuses(self):
        r = self._transfer({"cpl_x": 20, "cpl_y": 180})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("normalized code dropped only 60 B of it (cpl_x -80 budget, -60 B code)", r.stderr)

    def test_a_budget_raise_no_wasm_build_measured_refuses(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        web = theirs / "web" / "scripts"
        web.mkdir(parents=True)
        shutil.copy(HERE.parent / "web" / "scripts" / "wasm-crates.mjs", web)
        (web / "wasm-crate-budgets.json").write_text('{"cpl_x": 100, "cpl_y": 100}')
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "--rebase", "origin", "main")
        (ours / "web" / "scripts" / "wasm-crate-budgets.json").write_text('{"cpl_x": 50, "cpl_y": 150}')
        self.commit(ours, "ours")
        env = self._wasm(ours, 0)
        (self.tmp / "bin" / "node").unlink()
        r = self.land(ours, **env)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("no wasm build measured the moved code", r.stderr)
        self.assertNotIn("ours", git(self.tmp / "remote.git", "log", "--format=%s", "main"))

    def _raise(self, trailer):
        """A land raising cpl_y's wasm budget 100 -> 150 with no build measuring a transfer,
        its commit carrying `trailer`."""
        theirs, ours = self.clone("theirs"), self.clone("ours")
        web = theirs / "web" / "scripts"
        web.mkdir(parents=True)
        shutil.copy(HERE.parent / "web" / "scripts" / "wasm-crates.mjs", web)
        (web / "wasm-crate-budgets.json").write_text('{"cpl_x": 100, "cpl_y": 100}')
        self.commit(theirs, "theirs")
        git(theirs, "push", "-q", "origin", "HEAD:main")
        git(ours, "pull", "-q", "--rebase", "origin", "main")
        (ours / "web" / "scripts" / "wasm-crate-budgets.json").write_text('{"cpl_x": 100, "cpl_y": 150}')
        git(ours, "add", "-A")
        git(ours, "commit", "-q", "--no-verify", "-m", "ours", "-m", f"Domain: infra\nLane: G\n{trailer}")
        env = self._wasm(ours, 0)
        (self.tmp / "bin" / "node").unlink()
        return self.land(ours, **env)

    def test_a_budget_raise_the_coordinator_signed_lands(self):
        r = self._raise("Budget-raise: cpl_y 100->150, signed off by coordinator")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("ours", git(self.tmp / "remote.git", "log", "--format=%s", "main"))

    def test_a_budget_raise_without_the_coordinators_sign_off_refuses(self):
        r = self._raise("Budget-raise: cpl_y 100->150")
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("no wasm build measured the moved code", r.stderr)

    def test_a_signed_budget_raise_to_another_number_refuses(self):
        r = self._raise("Budget-raise: cpl_y 100->140, signed off by coordinator")
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("no wasm build measured the moved code", r.stderr)

    def _check(self, ours_gz, budgets, parent_gz=None, built_gz=None):
        """The commit tier's wasm step (ci.py wasm_data runs scripts/wasm-warm.sh --check)
        through the real wasm-crates.mjs: a build measuring `ours_gz` (crate -> B gz)
        against HEAD's recorded sizes `parent_gz`, or none, under `budgets`; with none,
        a build of HEAD's tree measures `built_gz`, or fails when that is None."""
        data = "data (std source, strings)"
        repo = self.clone("ours")
        web = repo / "web" / "scripts"
        web.mkdir(parents=True)
        shutil.copy(HERE.parent / "web" / "scripts" / "wasm-crates.mjs", web)
        (web / "wasm-crate-budgets.json").write_text(json.dumps({data: 100, **budgets}))
        (web / "wasm-sort-copies.json").write_text('{"max": 0}\n')
        shutil.copytree(HERE.parent / "handbook" / "domains", repo / "handbook" / "domains")
        (repo / "crates" / "cpl-x").mkdir(parents=True)
        (repo / "crates" / "cpl-x" / "lib.rs").write_text("x\n")
        built = self.tmp / "built.wasm"
        if built_gz is not None:
            built.write_text(sized({data: 90, **built_gz}))
        (repo / "scripts" / "build-wasm.sh").write_text(f'#!/bin/sh\nmkdir -p "$2"\ncp "{built}" "$2/cpl.wasm"\n')
        (repo / "scripts" / "build-wasm.sh").chmod(0o755)
        self.commit(repo, "gate")
        wasm = self.tmp / "gate.wasm"
        wasm.write_text(sized({data: 90, **ours_gz}))
        if parent_gz is not None:
            sizes = self.tmp / "tgt" / "wasm-warm" / "sizes"
            sizes.mkdir(parents=True)
            tree = git(repo, "rev-parse", "HEAD^{tree}").strip()
            (sizes / f"{tree}.json").write_text(sized({data: 90, **parent_gz}))
        env = {**os.environ, "CARGO_TARGET_DIR": str(self.tmp / "tgt")}
        return subprocess.run(["bash", "scripts/wasm-warm.sh", "--check", str(wasm), "HEAD", "cpl-x"],
                              cwd=repo, env=env, capture_output=True, text=True)

    def test_the_commit_tier_passes_a_crate_over_budget_that_shrank(self):
        # D8's follow-up cut cpl_check 277 B and the commit tier still refused it.
        r = self._check({"cpl_x": 149}, {"cpl_x": 100}, parent_gz={"cpl_x": 150})
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("but its normalized code is no larger than its parent's", r.stderr)

    def test_the_commit_tier_refuses_a_crate_over_budget_that_grew(self):
        r = self._check({"cpl_x": 151}, {"cpl_x": 100}, parent_gz={"cpl_x": 150})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("it grew 1 B against its parent's 150 B", r.stderr)

    def test_an_unrecorded_base_is_built_and_holds_an_over_budget_crate_at_net_0(self):
        # VM-trace's c9d4caff8: main's newest tip had no sizes, so cpl_ir at net 0 met
        # its absolute budget and was refused twice.
        r = self._check({"cpl_x": 150}, {"cpl_x": 100}, built_gz={"cpl_x": 150})
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("building it to hold the crates to it", r.stderr)
        self.assertEqual(len(list((self.tmp / "tgt" / "wasm-warm" / "sizes").glob("*.json"))), 1)

    def test_growth_against_a_built_base_refuses(self):
        r = self._check({"cpl_x": 151}, {"cpl_x": 100}, built_gz={"cpl_x": 150})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("it grew 1 B against its parent's 150 B", r.stderr)

    def test_a_base_that_cant_be_built_refuses_instead_of_using_budgets(self):
        r = self._check({"cpl_x": 90}, {"cpl_x": 100})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("could not build HEAD's wasm", r.stderr)

    def test_land_refuses_when_it_cant_build_the_wasm_it_lacks(self):
        # wasm-cut: "no current wasm build; CI checks it" let six cpl_check growths land.
        ours = self._crate_change("cpl-y")
        r = self.land(ours, **self._wasm(ours, 1))
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("land: no cpl.wasm of the rebased tip", r.stderr)
        self.assertNotIn("CI checks it", r.stderr)
        self.assertNotIn("ours", git(ours, "log", "--format=%s", "origin/main"))

    def test_a_stale_warm_build_is_rebuilt_at_the_rebased_tip_and_held_to_main(self):
        ours = self._crate_change("cpl-x")
        self._built(ours)
        build = ours / "scripts" / "build-wasm.sh"
        build.write_text('#!/bin/sh\nmkdir -p "$2"\necho built > "$2/cpl.wasm"\n')
        build.chmod(0o755)
        self.commit(ours, "build")
        env = self._wasm(ours, 0)
        node = self.tmp / "bin" / "node"
        node.write_text(node.read_text().replace("exit 0", '[ "$2" != --sizes ] || echo \'{"fns": {}}\' > "$4"\nexit 0'))
        r = self.land(ours, **env)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("land: building the rebased tip's cpl.wasm", r.stderr)
        self.assertIn("--data", (self.tmp / "node.log").read_text())

    def test_wasm_warm_records_the_tree_it_built_and_only_a_committed_one(self):
        ours = self.clone("ours")
        (ours / "scripts" / "build-wasm.sh").write_text('#!/bin/sh\nmkdir -p "$2"\necho built >> "$2/cpl.wasm"\n')
        (ours / "scripts" / "build-wasm.sh").chmod(0o755)
        self.commit(ours, "build")
        git(ours, "push", "-q", "origin", "HEAD:main")
        main_tree = git(ours, "rev-parse", "HEAD^{tree}").strip()
        (ours / "crates" / "cpl-x").mkdir(parents=True)
        (ours / "crates" / "cpl-x" / "lib.rs").write_text("ours\n")
        self.commit(ours, "ours")
        env = {**os.environ, **self._wasm(ours, 0)}
        node = self.tmp / "bin" / "node"
        node.write_text(node.read_text().replace("exit 0", '[ "$2" != --sizes ] || echo \'{"fns": {}}\' > "$4"\nexit 0'))
        warm = lambda *a: subprocess.run(["bash", "scripts/wasm-warm.sh", *a], cwd=ours, env=env,
                                         capture_output=True, text=True)
        (ours / "crates" / "cpl-x" / "lib.rs").write_text("edited\n")
        self.assertEqual(warm("--wait").returncode, 0)
        self.assertEqual(warm("--current", "cpl-x").returncode, 1)
        git(ours, "checkout", "-q", "--", "crates")
        r = warm("--wait")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        r = warm("--current", "cpl-x")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stdout.strip(), str(self.tmp / "tgt" / "wasm-warm" / "cpl.wasm"))
        self.assertIn("is current", warm("--wait").stdout)
        out = self.tmp / "tgt" / "wasm-warm"
        self.assertEqual((out / "cpl.wasm").read_text(), "built\n")
        self.assertTrue((out / "sizes" / f'{git(ours, "rev-parse", "HEAD^{tree}").strip()}.json').exists())
        self.assertTrue((out / "sizes" / f"{main_tree}.json").exists())
        self.assertIn(f"node web/scripts/wasm-crates.mjs --sizes {out}/base/cpl.wasm", (self.tmp / "node.log").read_text())
        r = warm("--parent", "origin/main", "cpl-x")
        self.assertEqual((r.returncode, r.stdout.strip()), (0, str(out / "sizes" / f"{main_tree}.json")))
        self.assertEqual(warm("--parent", "HEAD", "cpl_nope").returncode, 2)
        self.assertEqual(warm("--current", "cpl-nope").returncode, 2)
        (ours / "crates" / "cpl-w").mkdir(parents=True)
        (ours / "crates" / "cpl-w" / "lib.rs").write_text("another lane's crate\n")
        self.commit(ours, "another lane")
        self.assertEqual(warm("--current", "cpl-x").returncode, 1)
        git(ours, "reset", "-q", "--hard", "HEAD~1")
        self.assertEqual(warm("--parent", "HEAD", "cpl-x").stdout.strip(), str(out / "sizes" / f'{git(ours, "rev-parse", "HEAD^{tree}").strip()}.json'))
        (ours / "crates" / "cpl-y").mkdir(parents=True)
        (ours / "crates" / "cpl-y" / "lib.rs").write_text("other crate\n")
        self.commit(ours, "other crate")
        newer = out / "sizes" / f'{git(ours, "rev-parse", "HEAD^{tree}").strip()}.json'
        newer.write_text("{}")
        os.utime(newer, (time.time() + 60, time.time() + 60))
        r = warm("--parent", "origin/main", "cpl-x")
        self.assertEqual(r.stdout.strip(), str(out / "sizes" / f"{main_tree}.json"))

    def test_wasm_warm_parent_reads_a_crate_and_std_as_its_inputs(self):
        ours = self.clone("ours")
        out = self.tmp / "tgt" / "wasm-warm"
        (out / "sizes").mkdir(parents=True)
        (ours / "crates" / "cpl-x").mkdir(parents=True)
        (ours / "crates" / "cpl-x" / "lib.rs").write_text("one\n")
        self.commit(ours, "one")
        old = git(ours, "rev-parse", "HEAD^{tree}").strip()
        (out / "sizes" / f"{old}.json").write_text('{"fns": {}}')
        warm = lambda *a: subprocess.run(["bash", "scripts/wasm-warm.sh", *a], cwd=ours, env={**os.environ, **self._wasm(ours, 0)},
                                         capture_output=True, text=True)
        for edit in (ours / "crates" / "cpl-x" / "lib.rs", ours / "std" / "seq.cp"):
            edit.parent.mkdir(parents=True, exist_ok=True)
            edit.write_text("two\n")
            self.commit(ours, "two")
            self.assertEqual(warm("--parent", "HEAD", "cpl-x").returncode, 1, edit)
            git(ours, "reset", "-q", "--hard", "HEAD~1")
        (ours / "README.md").write_text("elsewhere\n")
        self.commit(ours, "elsewhere")
        self.assertEqual(warm("--parent", "HEAD", "cpl-x").stdout.strip(), str(out / "sizes" / f"{old}.json"))


if __name__ == "__main__":
    unittest.main()
