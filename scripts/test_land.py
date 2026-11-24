"""Tests for scripts/land.sh against a throwaway remote."""

import json
import os
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
        for name in ("land.sh", "wasm-warm.sh", "retry.sh", "ratchet_merge.py", "ratchet_remeasure.py", "gen.py"):
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
        git(repo, "commit", "-q", "--no-verify", "-m", msg, "-m", "Domain: infra")

    def land(self, repo, **extra):
        env = {**{k: v for k, v in os.environ.items() if not k.startswith("CPL_LAND")}, "CPL_LAND_TRIES": "1", **extra}
        return subprocess.run(["bash", "scripts/land.sh"], cwd=repo, env=env, capture_output=True, text=True)

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
        land.write_text(land.read_text().replace('t0=$(date +%s)\n', 't0=$(date +%s)\necho "land: main\'s copy" >&2\n', 1))
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
            "sys.exit(1 if sys.argv[1:] == ['--run-slices', 'copies'] else 0)\n")
        self.commit(ours, "a slice runner")
        git(ours, "push", "-q", "origin", "HEAD:main")
        for slices, code in (("copies\n", 1), ("vm-trace\nvm-trace\n", 0)):
            pending = ours / ".git" / "cpl-pending-slices"
            pending.write_text(slices)
            (ours / "pkg.txt").write_text(slices)
            self.commit(ours, "ours")
            r = self.land(ours)
            self.assertEqual(r.returncode, code, r.stderr)
            self.assertIn("slices --run-slices " + slices.split()[0], r.stderr)
            self.assertEqual(pending.exists(), code != 0)
            git(ours, "fetch", "-q")
            self.assertEqual(git(ours, "show", "origin/main:pkg.txt") if code == 0 else
                             git(ours, "ls-tree", "origin/main", "pkg.txt"), "" if code else slices)

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
            self.assertIn(f"cases --build {case}", r.stderr)
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
            "    held = subprocess.run(['git', 'ls-remote', 'origin', 'refs/land/lock'], capture_output=True, text=True).stdout\n"
            "    open(os.environ['LOCKS'], 'a').write((held.split() or ['free'])[0] + '\\n')\n"
            "sys.exit('warning: field `0` is never read' if 'wasm' in sys.argv[-1] else 0)\n")
        self.commit(repo, "a commit tier")
        git(repo, "push", "-q", "origin", "HEAD:main")
        return repo / ".git" / "cpl-pending-steps", self.tmp / "ci.runs"

    def _hooked_commit(self, repo, msg):
        env = {**os.environ, "CPL_COMMIT_TIMEOUT": "1"}
        git(repo, "add", "-A")
        subprocess.run(["git", "-c", "core.hooksPath=.githooks", "commit", "-q", "-m", msg, "-m", "Domain: infra"],
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

    def test_after_a_lost_push_race_the_pending_steps_run_again_on_the_new_tip(self):
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
        self.assertEqual(runs.read_text(), "commit --only nextest\n" * 2)
        self.assertIn("pushed", r.stdout)
        self.assertFalse(pending.exists())

    def lock(self, repo, age=0):
        """Hold refs/land/lock on the remote as a lane that took it `age` s ago."""
        empty = git(repo, "hash-object", "-t", "tree", "/dev/null").strip()
        sha = git(repo, "commit-tree", empty, "-m", f"land-lock other 1 {int(time.time()) - age} infra").strip()
        git(repo, "push", "-q", "origin", f"{sha}:refs/land/lock")
        return sha

    def lock_on_remote(self):
        return git(self.tmp, "ls-remote", str(self.tmp / "remote.git"), "refs/land/lock").strip()

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
        r = self.land(repo)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("taking over a land lock", r.stderr)
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_lock_taken_after_a_wait_is_stamped_when_it_is_taken(self):
        repo, other = self.clone("lane"), self.clone("other")
        self.lock(repo)
        stamps = self.tmp / "stamps"
        hook = self.tmp / "remote.git" / "hooks" / "pre-receive"
        hook.write_text("#!/bin/sh\nwhile read old new ref; do\n"
                        f'  [ "$ref" != refs/land/lock ] || git log -1 --format=%s "$new" >>"{stamps}" 2>/dev/null || :\n'
                        "done\n")
        hook.chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        freed = []
        timer = threading.Timer(3, lambda: (freed.append(time.time()),
                                            git(other, "push", "-q", "origin", ":refs/land/lock")))
        timer.start()
        r = self.land(repo)
        timer.join()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("took the land lock after", r.stderr)
        taken = [int(s.split()[3]) for s in stamps.read_text().splitlines() if " other " not in f" {s} "]
        self.assertTrue(taken and taken[0] >= int(freed[0]), (taken, freed))

    def test_a_land_that_loses_the_race_reruns_without_the_land_lock(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        git(theirs, "pull", "-q", "origin", "main")
        (theirs / "theirs.txt").write_text("x\n")
        self.commit(theirs, "theirs")
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("nextest\n")
        locks = self.tmp / "locks"
        r = self.land(ours, CI_RUNS=str(runs), RACE=str(theirs), LOCKS=str(locks), CPL_LAND_TRIES="2")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("pushed", r.stdout)
        # The rerun after the lost race ran while another lane could take the lock.
        self.assertEqual(locks.read_text(), "free\nfree\n")
        self.assertEqual(self.lock_on_remote(), "")

    def test_a_lock_held_past_120s_is_stale_by_default(self):
        repo = self.clone("lane")
        self.lock(repo, age=150)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("taking over a land lock held over 120s", r.stderr)

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
            'case "$*" in *"--force-with-lease=refs/land/lock:"[0-9a-f]*)\n'
            f'  [ -e "{self.tmp}/rival.done" ] || {{ touch "{self.tmp}/rival.done"; '
            f'"{real}" -C "{rival}" push -q -f origin "{theirs}:refs/land/lock"; }} ;;\n'
            "esac\n"
            f'exec "{real}" "$@"\n')
        (shim / "git").chmod(0o755)
        (repo / "pkg.txt").write_text("x\n")
        self.commit(repo, "ours")
        r = self.land(repo, PATH=f"{shim}:{os.environ['PATH']}", CPL_LAND_LOCK_WAIT="2")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("taking over a land lock", r.stderr)
        self.assertIn("waiting for the land lock (land-lock rival", r.stderr)
        self.assertTrue(self.lock_on_remote().startswith(theirs))

    def test_a_refused_land_drops_the_land_lock(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        pending, runs = self._steps_runner(ours)
        (ours / "pkg.txt").write_text("x\n")
        self.commit(ours, "ours")
        pending.write_text("wasm\n")
        r = self.land(ours, CI_RUNS=str(runs))
        self.assertNotEqual(r.returncode, 0)
        self.assertEqual(self.lock_on_remote(), "")

    def test_the_pending_steps_rerun_once_after_lost_races_then_land(self):
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
        self.assertEqual(runs.read_text(), "commit --only nextest\n" * 2)
        self.assertIn("landing on the skipped steps' rerun", r.stderr)

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
            "print('spec/conformance/c.cp')\n")
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
        self.assertIn("cases --build spec/conformance/c.cp", r.stderr)

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
        # The lower of 8 (new rule) and 3 (old rule) would be 3; the tip measures 6.
        self.assertEqual(json.loads((ours / RATCHET).read_text())["lines"], 6)
        self.assertIn("lines 3 -> 6", r.stderr)
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
        git(ours, "commit", "-q", "--no-verify", "-m", "ours", "-m", "Domain: infra")
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
        for merged in (merge(base, raised, base), merge(base, base, raised)):
            self.assertEqual(merged, base)

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
        """A warm wasm build of ours' tree as it is now, and the sizes of main's tip."""
        out = self.tmp / "tgt" / "wasm-warm"
        out.mkdir(parents=True)
        (out / "cpl.wasm").write_text("wasm")
        (out / "tree").write_text(git(ours, "rev-parse", "HEAD^{tree}"))
        if parent:
            git(ours, "fetch", "-q", "origin")
            (out / "sizes").mkdir()
            (out / "sizes" / f'{git(ours, "rev-parse", "origin/main^{tree}").strip()}.json').write_text("{}")
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

    def test_without_main_tips_wasm_sizes_land_warns_and_lands(self):
        ours = self._crate_change("cpl-y")
        self._built(ours, parent=False)
        r = self.land(ours, **self._wasm(ours, 1))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("land: no wasm sizes of main's tip to compare with; CI checks it", r.stderr)
        self.assertFalse((self.tmp / "node.log").exists())

    def test_a_warm_build_within_budget_lands(self):
        ours = self._crate_change("cpl-y")
        self._built(ours)
        r = self.land(ours, **self._wasm(ours, 0))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("--parent", (self.tmp / "node.log").read_text())

    def _ratchet(self, ours_gz, parent_gz, budgets):
        """A land of a cpl-x change through the real wasm-crates.mjs: a warm build measuring
        `ours_gz` and main's tip `parent_gz` (crate -> B gz), under `budgets`."""
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
        self.commit(ours, "ours")
        out = self._built(ours)
        (out / "cpl.wasm").write_text(json.dumps({"crates": ours_gz, "sorts": 0}))
        (next((out / "sizes").iterdir())).write_text(json.dumps({"crates": parent_gz, "sorts": 0}))
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
        self.assertIn("but no more than its parent's 150 B", r.stderr)

    def test_the_wasm_ratchet_lands_a_crate_over_budget_that_held_its_size(self):
        r = self._ratchet({"cpl_x": 150}, {"cpl_x": 150}, {"cpl_x": 100})
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("but no more than its parent's 150 B", r.stderr)

    def test_the_wasm_ratchet_keeps_a_crate_within_budget_within_it(self):
        r = self._ratchet({"cpl_x": 50, "cpl_y": 101}, {"cpl_x": 50, "cpl_y": 90}, {"cpl_x": 100, "cpl_y": 100})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("cpl_y is 101 B gzipped, over its budget of 100", r.stderr)
        self.assertNotIn("ours", git(self.tmp / "remote.git", "log", "--format=%s", "main"))

    def _check(self, ours_gz, budgets, parent_gz=None):
        """The commit tier's wasm step (ci.py wasm_data runs scripts/wasm-warm.sh --check)
        through the real wasm-crates.mjs: a build measuring `ours_gz` (crate -> B gz)
        against HEAD's recorded sizes `parent_gz`, or none, under `budgets`."""
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
        self.commit(repo, "gate")
        wasm = self.tmp / "gate.wasm"
        wasm.write_text(json.dumps({"crates": {data: 90, **ours_gz}, "sorts": 0}))
        if parent_gz is not None:
            sizes = self.tmp / "tgt" / "wasm-warm" / "sizes"
            sizes.mkdir(parents=True)
            tree = git(repo, "rev-parse", "HEAD^{tree}").strip()
            (sizes / f"{tree}.json").write_text(json.dumps({"crates": {data: 90, **parent_gz}, "sorts": 0}))
        env = {**os.environ, "CARGO_TARGET_DIR": str(self.tmp / "tgt")}
        return subprocess.run(["bash", "scripts/wasm-warm.sh", "--check", str(wasm), "HEAD", "cpl-x"],
                              cwd=repo, env=env, capture_output=True, text=True)

    def test_the_commit_tier_passes_a_crate_over_budget_that_shrank(self):
        # D8's follow-up cut cpl_check 277 B and the commit tier still refused it.
        r = self._check({"cpl_x": 149}, {"cpl_x": 100}, parent_gz={"cpl_x": 150})
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("but no more than its parent's 150 B", r.stderr)

    def test_the_commit_tier_refuses_a_crate_over_budget_that_grew(self):
        r = self._check({"cpl_x": 151}, {"cpl_x": 100}, parent_gz={"cpl_x": 150})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("it grew 1 B against its parent's 150 B", r.stderr)

    def test_the_commit_tier_without_parent_sizes_holds_touched_crates_to_their_budgets(self):
        r = self._check({"cpl_x": 149}, {"cpl_x": 100})
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("wasm-warm: no wasm sizes of HEAD's tree", r.stderr)
        self.assertIn("cpl_x is 149 B gzipped, over its budget of 100", r.stderr)

    def test_land_never_waits_on_a_wasm_build_it_lacks(self):
        ours = self._crate_change("cpl-y")
        r = self.land(ours, **self._wasm(ours, 1))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("land: no current wasm build; CI checks it", r.stderr)
        self.assertFalse((self.tmp / "node.log").exists())

    def test_a_warm_build_of_a_touched_crate_main_has_since_changed_is_not_current(self):
        ours = self._crate_change("cpl-x")
        self._built(ours)
        r = self.land(ours, **self._wasm(ours, 1))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("land: no current wasm build; CI checks it", r.stderr)
        self.assertFalse((self.tmp / "node.log").exists())

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
        node.write_text(node.read_text().replace("exit 0", '[ "$2" != --sizes ] || echo "{}" > "$4"\nexit 0'))
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
        self.assertEqual(warm("--parent", "HEAD", "cpl_x").returncode, 2)
        self.assertEqual(warm("--current", "cpl_x").returncode, 2)
        self.assertEqual(warm("--parent", "HEAD", "cpl-x").stdout.strip(), str(out / "sizes" / f'{git(ours, "rev-parse", "HEAD^{tree}").strip()}.json'))
        (ours / "crates" / "cpl-y").mkdir(parents=True)
        (ours / "crates" / "cpl-y" / "lib.rs").write_text("other crate\n")
        self.commit(ours, "other crate")
        newer = out / "sizes" / f'{git(ours, "rev-parse", "HEAD^{tree}").strip()}.json'
        newer.write_text("{}")
        os.utime(newer, (time.time() + 60, time.time() + 60))
        for touched in ("cpl-x", "cpl_x"):
            r = warm("--parent", "origin/main", touched)
            self.assertEqual(r.stdout.strip(), str(out / "sizes" / f"{main_tree}.json"), touched)

    def test_wasm_warm_parent_reads_a_cpl_x_name_as_its_crate_and_std_as_an_input(self):
        ours = self.clone("ours")
        out = self.tmp / "tgt" / "wasm-warm"
        (out / "sizes").mkdir(parents=True)
        (ours / "crates" / "cpl-x").mkdir(parents=True)
        (ours / "crates" / "cpl-x" / "lib.rs").write_text("one\n")
        self.commit(ours, "one")
        old = git(ours, "rev-parse", "HEAD^{tree}").strip()
        (out / "sizes" / f"{old}.json").write_text("{}")
        warm = lambda *a: subprocess.run(["bash", "scripts/wasm-warm.sh", *a], cwd=ours, env={**os.environ, **self._wasm(ours, 0)},
                                         capture_output=True, text=True)
        for edit in (ours / "crates" / "cpl-x" / "lib.rs", ours / "std" / "seq.cp"):
            edit.parent.mkdir(parents=True, exist_ok=True)
            edit.write_text("two\n")
            self.commit(ours, "two")
            for name in ("cpl-x", "cpl_x"):
                self.assertEqual(warm("--parent", "HEAD", name).returncode, 1, (edit, name))
            git(ours, "reset", "-q", "--hard", "HEAD~1")
        (ours / "README.md").write_text("elsewhere\n")
        self.commit(ours, "elsewhere")
        self.assertEqual(warm("--parent", "HEAD", "cpl_x").stdout.strip(), str(out / "sizes" / f"{old}.json"))


if __name__ == "__main__":
    unittest.main()
