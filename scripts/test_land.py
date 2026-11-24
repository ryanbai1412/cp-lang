"""Tests for scripts/land.sh against a throwaway remote."""

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
GEN = "spec/conformance/COVERAGE.md"


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
        for name in ("land.sh", "retry.sh"):
            shutil.copy(HERE / name, seed / "scripts" / name)
        # A regen.sh that dirties a file the rebase never saw: land must not run it.
        (seed / "scripts" / "regen.sh").write_text("#!/bin/sh\necho x >> other.txt\necho other.txt\n")
        (seed / "scripts" / "regen.sh").chmod(0o755)
        (seed / GEN).parent.mkdir(parents=True)
        (seed / GEN).write_text("base\n")
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


if __name__ == "__main__":
    unittest.main()
