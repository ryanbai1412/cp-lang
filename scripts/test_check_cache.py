#!/usr/bin/env python3
"""Unit tests for scripts/check_cache.py and .githooks/pre-push."""

import os
import subprocess
import tempfile
import time
import unittest

import scripts.check_cache as cc


class CheckCacheTest(unittest.TestCase):
    def test_hash_changes_on_edit_and_ignores_pycache(self):
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "src", "__pycache__"), exist_ok=True)
            f1 = os.path.join(tmp, "src", "a.rs")
            with open(f1, "w", encoding="utf-8") as f:
                f.write("fn main() {}\n")
            d1 = cc.hash_paths(tmp, ["src"])

            # Writing inside __pycache__ must not invalidate the hash.
            with open(os.path.join(tmp, "src", "__pycache__", "x.pyc"), "wb") as f:
                f.write(b"1234")
            self.assertEqual(cc.hash_paths(tmp, ["src"]), d1)

            # Editing a tracked source file must invalidate the hash.
            with open(f1, "w", encoding="utf-8") as f:
                f.write("fn main() { let _ = 1; }\n")
            d2 = cc.hash_paths(tmp, ["src"])
            self.assertNotEqual(d1, d2)

            cache_dir = os.path.join(tmp, "cache")
            self.assertFalse(cc.is_hit(cache_dir, "clippy", d2))
            cc.save_hit(cache_dir, "clippy", d2)
            self.assertTrue(cc.is_hit(cache_dir, "clippy", d2))
            self.assertFalse(cc.is_hit(cache_dir, "clippy", d1))

    def test_pre_push_hook_rejects_conflict_markers(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        hook = os.path.join(root, ".githooks", "pre-push")
        self.assertTrue(os.path.isfile(hook))
        self.assertTrue(os.access(hook, os.X_OK))
        # A commit hook exports GIT_DIR and GIT_INDEX_FILE; inherited, they aim these at the real repo.
        env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        with tempfile.TemporaryDirectory() as tmp:
            subprocess.run(["git", "init", "-q"], cwd=tmp, check=True, env=env)
            subprocess.run(["git", "config", "user.email", "t@example.com"], cwd=tmp, check=True, env=env)
            subprocess.run(["git", "config", "user.name", "t"], cwd=tmp, check=True, env=env)
            p = os.path.join(tmp, "bad.md")
            with open(p, "w", encoding="utf-8") as f:
                f.write("hello\n<<<<<<< HEAD\nfoo\n=======\nbar\n>>>>>>> other\n")
            subprocess.run(["git", "add", "bad.md"], cwd=tmp, check=True, env=env)
            r = subprocess.run([hook], cwd=tmp, capture_output=True, text=True, env=env)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("merge conflict markers", r.stderr)

    def test_pre_push_hook_rejects_a_main_tip_that_never_passed_the_gate(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        hook = os.path.join(root, ".githooks", "pre-push")
        env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        with tempfile.TemporaryDirectory() as tmp:
            git = lambda *a: subprocess.run(["git", *a], cwd=tmp, check=True, env=env,
                                            capture_output=True, text=True).stdout.strip()
            git("init", "-q")
            git("config", "user.email", "t@example.com")
            git("config", "user.name", "t")
            with open(os.path.join(tmp, "a.md"), "w", encoding="utf-8") as f:
                f.write("ok\n")
            git("add", "a.md")
            git("commit", "-qm", "docs: a")
            sha, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
            line = f"refs/heads/x {sha} refs/heads/main {'0' * 40}\n"
            run = lambda: subprocess.run([hook], cwd=tmp, input=line, capture_output=True,
                                         text=True, env=env)
            r = run()
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("has not passed scripts/check.sh --fast", r.stderr)
            with open(os.path.join(tmp, ".git", "cpl-gate-passed"), "w", encoding="utf-8") as f:
                f.write(tree + "\n")
            r = run()
            self.assertEqual(r.returncode, 0, r.stderr)


    def test_land_takes_the_lock_gates_pushes_and_releases(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        with tempfile.TemporaryDirectory() as tmp:
            remote, work = os.path.join(tmp, "remote.git"), os.path.join(tmp, "work")
            def git(*a, cwd=work):
                return subprocess.run(["git", *a], cwd=cwd, check=True, env=env,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q", "--bare", "-b", "main", remote, cwd=tmp)
            git("init", "-q", "-b", "main", work, cwd=tmp)
            git("config", "user.email", "t@example.com")
            git("config", "user.name", "t")
            os.makedirs(os.path.join(work, "scripts"))
            gate = os.path.join(tmp, "gate.sh")
            with open(gate, "w", encoding="utf-8") as f:
                f.write('#!/bin/sh\ngit rev-parse "HEAD^{tree}" >> "$(git rev-parse --git-common-dir)/cpl-gate-passed"\n')
            os.chmod(gate, 0o755)
            with open(os.path.join(work, "a.md"), "w", encoding="utf-8") as f:
                f.write("ok\n")
            git("add", "a.md")
            git("commit", "-qm", "docs: a")
            git("push", "-q", remote, "HEAD:main")
            with open(os.path.join(work, "a.md"), "a", encoding="utf-8") as f:
                f.write("more\n")
            git("commit", "-qam", "docs: more")
            empty = git("hash-object", "-t", "tree", "-w", "/dev/null")
            stale = git("commit-tree", empty, "-m", "land lock: other")
            git("push", "-q", remote, f"{stale}:refs/locks/land")
            dead = subprocess.run(
                ["git", "commit-tree", empty, "-m", "land ticket: gone"], cwd=work, check=True,
                capture_output=True, text=True,
                env={**env, "GIT_COMMITTER_DATE": "2000-01-01T00:00:00Z"}).stdout.strip()
            git("push", "-q", remote, f"{dead}:refs/locks/queue/0000000000000000000-1-gone")
            git("config", "core.hooksPath", os.path.join(root, ".githooks"))
            land = os.path.join(root, "scripts", "land.sh")
            r = subprocess.run([land, remote], cwd=work, capture_output=True, text=True,
                               env={**env, "CPL_LAND_GATE": gate, "CPL_LAND_TTL": "0"})
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn("taking over stale lock from land lock: other", r.stderr)
            self.assertEqual(git("rev-parse", "main", cwd=remote), git("rev-parse", "HEAD"))
            self.assertEqual(git("ls-remote", remote, "refs/locks/land"), "")
            self.assertEqual(git("ls-remote", remote, "refs/locks/*"), "")

    def _train(self, gate_body, riders_ok):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        with tempfile.TemporaryDirectory() as tmp:
            remote = os.path.join(tmp, "remote.git")
            def git(*a, cwd=tmp):
                return subprocess.run(["git", *a], cwd=cwd, check=True, env=env,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q", "--bare", "-b", "main", remote)
            seed = os.path.join(tmp, "seed")
            git("init", "-q", "-b", "main", seed)
            git("-c", "user.name=t", "-c", "user.email=t@example.com",
                "commit", "-q", "--allow-empty", "-m", "docs: seed", cwd=seed)
            git("push", "-q", remote, "HEAD:main", cwd=seed)
            gate = os.path.join(tmp, "gate.sh")
            with open(gate, "w", encoding="utf-8") as f:
                f.write("#!/bin/sh\n" + gate_body +
                        'git rev-parse "HEAD^{tree}" >> "$(git rev-parse --git-common-dir)/cpl-gate-passed"\n')
            os.chmod(gate, 0o755)
            lanes = {}
            for lane, path in (("first", "one.md"), ("rider", "bad.md" if not riders_ok else "two.md")):
                work = os.path.join(tmp, lane)
                git("clone", "-q", remote, work)
                git("config", "user.name", lane, cwd=work)
                git("config", "user.email", "t@example.com", cwd=work)
                with open(os.path.join(work, path), "w", encoding="utf-8") as f:
                    f.write(lane + "\n")
                git("add", path, cwd=work)
                git("commit", "-qm", f"docs: {lane}", cwd=work)
                git("config", "core.hooksPath", os.path.join(root, ".githooks"), cwd=work)
                lanes[lane] = work
            empty = git("hash-object", "-t", "tree", "-w", "/dev/null", cwd=seed)
            busy = git("commit-tree", empty, "-m", "land lock: other", cwd=seed)
            git("push", "-q", remote, f"{busy}:refs/locks/land", cwd=seed)
            land = os.path.join(root, "scripts", "land.sh")
            procs = {}
            for lane in ("first", "rider"):
                procs[lane] = subprocess.Popen(
                    [land, remote], cwd=lanes[lane], stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE, text=True,
                    env={**env, "CPL_LAND_GATE": gate, "CPL_LANE": lane, "CPL_LAND_POLL": "1"})
                deadline = time.time() + 30
                while f"-{lane}" not in git("ls-remote", remote, "refs/locks/queue/*"):
                    self.assertLess(time.time(), deadline, f"{lane} never queued")
                    time.sleep(0.2)
            git("push", "-q", remote, ":refs/locks/land", cwd=seed)
            out = {lane: p.communicate(timeout=120) + (p.returncode,) for lane, p in procs.items()}
            files = git("ls-tree", "--name-only", "main", cwd=remote).split()
            return out, files, git("ls-remote", remote, "refs/locks/*")

    def test_land_carries_queued_lanes_in_one_gated_train(self):
        out, files, locks = self._train("", riders_ok=True)
        self.assertEqual(out["first"][2], 0, out["first"][1])
        self.assertEqual(out["rider"][2], 0, out["rider"][1])
        self.assertIn("gating a train of 1 rider(s)", out["first"][1])
        self.assertIn("carried to main in a train", out["rider"][0])
        self.assertEqual(sorted(files), ["one.md", "two.md"])
        self.assertEqual(locks, "")

    def test_land_drops_a_failing_train_and_lands_alone(self):
        out, files, locks = self._train("! test -e bad.md || exit 1\n", riders_ok=False)
        self.assertEqual(out["first"][2], 0, out["first"][1])
        self.assertIn("the train failed the gate", out["first"][1])
        self.assertNotEqual(out["rider"][2], 0)
        self.assertEqual(files, ["one.md"])
        self.assertIn("refs/locks/solo/", locks)

if __name__ == "__main__":
    unittest.main()
