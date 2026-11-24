"""Tests for scripts/land.sh against a throwaway remote."""

import json
import os
import shutil
import subprocess
import sys
import tempfile
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
        for name in ("land.sh", "retry.sh", "ratchet_merge.py", "gen.py"):
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

    def land(self, repo):
        env = {**os.environ, "CPL_LAND_TRIES": "1"}
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

    def test_the_rerun_checks_packages_with_the_cpl_the_hook_used(self):
        theirs, ours = self.clone("theirs"), self.clone("ours")
        self._hook(theirs, 'echo "rerun cpl=$CPL"; [ -x "$CPL" ]\n')
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
        self.assertIn("rerun cpl=", r.stderr)
        self.assertIn("cpl-land-cpl", r.stderr)

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


if __name__ == "__main__":
    unittest.main()
