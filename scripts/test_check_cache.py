#!/usr/bin/env python3
"""Unit tests for scripts/check_cache.py and .githooks/pre-push."""

import json
import os
import re
import shutil
import subprocess
import tempfile
import time
import unittest

import scripts.check_cache as cc



def hermetic_env():
    """The environment for a throwaway repo: no inherited GIT_* or CPL_LAND*
    (a commit hook exports GIT_DIR), no user or system git config, and a
    fixed identity, so the tests behave the same on a CI runner that has none."""
    env = {k: v for k, v in os.environ.items() if not k.startswith(("GIT_", "CPL_LAND"))}
    env.update(GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1",
               GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@example.com",
               GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@example.com")
    return env

class CheckCacheTest(unittest.TestCase):
    def git_tree(self, tmp, files):
        env = hermetic_env()
        subprocess.run(["git", "init", "-q"], cwd=tmp, check=True, env=env)
        for rel, text in files.items():
            os.makedirs(os.path.dirname(os.path.join(tmp, rel)) or tmp, exist_ok=True)
            with open(os.path.join(tmp, rel), "w", encoding="utf-8") as f:
                f.write(text)
        return env

    def test_a_stage_key_covers_the_whole_tree_but_its_excludes(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.git_tree(tmp, {
                ".gitignore": "target/\n",
                "crates/a.rs": "fn main() {}\n",
                "plan/STDLIB.md": "design\n",
                "plan/DOMAINS.md": "status\n",
                "research/notes.md": "x\n",
                "target/out": "build output\n",
            })
            pats = ["research/*", "plan/*", "!plan/STDLIB.md"]
            key = lambda: cc.stage_key(manifest(), "clippy", pats)
            # Pin the toolchain digest so a tool shim's output can't move the key.
            manifest = lambda: "env\n" + cc.manifest(tmp).split("\n", 1)[1]
            before = key()
            for rel in ["research/notes.md", "plan/DOMAINS.md", "target/out"]:
                with open(os.path.join(tmp, rel), "a", encoding="utf-8") as f:
                    f.write("edit\n")
                self.assertEqual(key(), before, rel)
            for rel in ["crates/a.rs", "plan/STDLIB.md", "new_untracked.txt"]:
                with open(os.path.join(tmp, rel), "a", encoding="utf-8") as f:
                    f.write("edit\n")
                self.assertNotEqual(key(), before, rel)
                before = key()
            os.chmod(os.path.join(tmp, "crates/a.rs"), 0o755)
            self.assertNotEqual(key(), before)

            cache_dir = os.path.join(tmp, "cache")
            k = key()
            self.assertFalse(cc.is_hit(cache_dir, "clippy", k))
            cc.save_hit(cache_dir, "clippy", k)
            self.assertTrue(cc.is_hit(cache_dir, "clippy", k))
            self.assertFalse(cc.is_hit(cache_dir, "clippy", before))

    def test_touching_any_unexcluded_file_invalidates_every_stage(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        script = ""
        for name in ("check.sh", "site-check.sh"):
            with open(os.path.join(root, "scripts", name), encoding="utf-8") as f:
                script += f.read()
        used = set(re.findall(r"cache_(?:hit|save) (\w+)", script))
        self.assertEqual(used, set(cc.STAGES))
        env, *lines = cc.manifest(root).splitlines()
        # One file from every directory two levels down, so an exclude that
        # swallows a whole subtree by mistake shows up.
        firsts = {}
        for line in lines:
            firsts.setdefault(tuple(line.split(" ", 1)[1].split("/")[:2]), line)
        sample = sorted(firsts.values()) + ["Fnew brand-new-top-level-file"]
        for stage, pats in cc.STAGES.items():
            before = cc.stage_key("\n".join([env] + sample), stage, pats)
            kept = 0
            for i, line in enumerate(sample):
                path = line.split(" ", 1)[1]
                touched = sample[:]
                touched[i] = "Fchanged " + path
                moved = cc.stage_key("\n".join([env] + touched), stage, pats) != before
                self.assertEqual(moved, not cc.excluded(path, pats), f"{stage}: {path}")
                kept += moved
            self.assertGreater(kept, 10, stage)
            other = cc.stage_key("\n".join(["other-toolchain"] + sample), stage, pats)
            self.assertNotEqual(other, before, stage)

    def test_no_stage_excludes_a_file_the_rust_build_reads(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        reads = ["spec/std/prelude.cp", "Cargo.toml", "Cargo.lock", "crates/cpl-cli/src/main.rs"]
        for d, _, names in os.walk(os.path.join(root, "crates")):
            for n in names:
                if not n.endswith(".rs"):
                    continue
                with open(os.path.join(d, n), encoding="utf-8") as f:
                    src = f.read()
                for target in re.findall(r'include_(?:str|bytes)!\(\s*"([^"]+)"', src):
                    path = os.path.relpath(os.path.normpath(os.path.join(d, target)), root)
                    self.assertTrue(os.path.exists(os.path.join(root, path)), path)
                    reads.append(path)
        self.assertIn("spec/std/README.md", reads)
        for stage, pats in cc.STAGES.items():
            for path in reads:
                self.assertFalse(cc.excluded(path, pats), f"{stage} excludes {path}, which the build reads")

    def test_pre_push_hook_rejects_conflict_markers(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        hook = os.path.join(root, ".githooks", "pre-push")
        self.assertTrue(os.path.isfile(hook))
        self.assertTrue(os.access(hook, os.X_OK))
        # A commit hook exports GIT_DIR and GIT_INDEX_FILE; inherited, they aim these at the real repo.
        env = hermetic_env()
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

    def test_pre_commit_hook_allows_the_commit_when_the_tier_runs_past_its_limit(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            subprocess.run(["git", "init", "-q"], cwd=tmp, check=True, env=env)
            os.makedirs(os.path.join(tmp, "scripts"))
            check = os.path.join(tmp, "scripts", "check.sh")
            hook = os.path.join(root, ".githooks", "pre-commit")
            for body, code in (("echo '==> slow step'; sleep 30 | cat; true", 0), ("echo '==> lint'; exit 3", 3)):
                with open(check, "w", encoding="utf-8") as f:
                    f.write("#!/usr/bin/env bash\n" + body + "\n")
                os.chmod(check, 0o755)
                t0 = time.monotonic()
                r = subprocess.run([hook], cwd=tmp, capture_output=True, text=True,
                                   env={**env, "CPL_COMMIT_TIMEOUT": "1"})
                self.assertEqual(r.returncode, code, r.stderr)
                self.assertLess(time.monotonic() - t0, 10)
            with open(os.path.join(tmp, ".git", "cpl-commit-slow.log"), encoding="utf-8") as f:
                self.assertIn("slow step", f.read())

    def test_pre_push_hook_rejects_a_main_tip_that_never_passed_the_gate(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        hook = os.path.join(root, ".githooks", "pre-push")
        env = hermetic_env()
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
            self.assertIn("has not passed scripts/check.sh --fast or --land", r.stderr)
            with open(os.path.join(tmp, ".git", "cpl-gate-passed"), "w", encoding="utf-8") as f:
                f.write(tree + "\n")
            r = run()
            self.assertEqual(r.returncode, 0, r.stderr)


    def test_gate_env_pins_target_to_the_checkout(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env_sh = os.path.join(root, "scripts", "gate-env.sh")
        with tempfile.TemporaryDirectory() as tmp:
            tmp = os.path.realpath(tmp)
            for target_dir, want in [(None, os.path.join(tmp, "target")),
                                     ("out", os.path.join(tmp, "out")),
                                     ("/abs/t", "/abs/t")]:
                env = {k: v for k, v in os.environ.items() if k != "CARGO_TARGET_DIR"}
                if target_dir:
                    env["CARGO_TARGET_DIR"] = target_dir
                r = subprocess.run(["bash", "-c", f'. "{env_sh}"; echo "$target"'],
                                   cwd=tmp, env=env, capture_output=True, text=True, check=True)
                self.assertEqual(r.stdout.split("\n")[0], want)

    def test_land_takes_the_lock_gates_pushes_and_releases(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
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
            orphan = git("commit-tree", empty, "-m", "solo: 1-1-left")
            git("push", "-q", remote, f"{orphan}:refs/locks/solo/1-1-left")
            git("config", "core.hooksPath", os.path.join(root, ".githooks"))
            land = os.path.join(root, "scripts", "land.sh")
            r = subprocess.run([land, remote], cwd=work, capture_output=True, text=True,
                               env={**env, "CPL_LAND_GATE": gate, "CPL_LAND_TTL": "0"})
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn("taking over stale lock from land lock: other", r.stderr)
            self.assertEqual(git("rev-parse", "main", cwd=remote), git("rev-parse", "HEAD"))
            self.assertEqual(git("ls-remote", remote, "refs/locks/land"), "")
            self.assertEqual(git("ls-remote", remote, "refs/locks/*"), "")

    def test_land_pushes_again_when_the_remote_fails_the_push(self):
        # GitHub sometimes fails a push on its side; with main and the lock
        # unchanged that costs a push retry, not another gate.
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            remote, work = os.path.join(tmp, "remote.git"), os.path.join(tmp, "work")
            def git(*a, cwd=work):
                return subprocess.run(["git", *a], cwd=cwd, check=True, env=env,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q", "--bare", "-b", "main", remote, cwd=tmp)
            git("init", "-q", "-b", "main", work, cwd=tmp)
            git("config", "user.email", "t@example.com")
            git("config", "user.name", "t")
            with open(os.path.join(work, "a.md"), "w", encoding="utf-8") as f:
                f.write("ok\n")
            git("add", "a.md")
            git("commit", "-qm", "docs: a")
            git("push", "-q", remote, "HEAD:main")
            git("commit", "-q", "--allow-empty", "-m", "docs: more")
            gate = os.path.join(tmp, "gate.sh")
            with open(gate, "w", encoding="utf-8") as f:
                f.write('#!/bin/sh\necho gated >> "$GATES"\n'
                        'git rev-parse "HEAD^{tree}" >> "$(git rev-parse --git-common-dir)/cpl-gate-passed"\n')
            os.chmod(gate, 0o755)
            hook = os.path.join(remote, "hooks", "pre-receive")
            with open(hook, "w", encoding="utf-8") as f:
                f.write('#!/bin/sh\nif grep -q refs/heads/main && [ ! -e "$FAILED" ]; then\n'
                        '  touch "$FAILED"; echo "error: unable to write file" >&2; exit 1\nfi\n')
            os.chmod(hook, 0o755)
            git("config", "core.hooksPath", os.path.join(root, ".githooks"))
            gates = os.path.join(tmp, "gates")
            r = subprocess.run([os.path.join(root, "scripts", "land.sh"), remote], cwd=work,
                               capture_output=True, text=True,
                               env={**env, "CPL_LAND_GATE": gate, "GATES": gates,
                                    "FAILED": os.path.join(tmp, "failed")})
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn("the push failed on the remote's side", r.stderr)
            self.assertEqual(git("rev-parse", "main", cwd=remote), git("rev-parse", "HEAD"))
            with open(gates, encoding="utf-8") as f:
                self.assertEqual(f.read().count("gated"), 1)

    def _train(self, gate_body, bad=None, names=("first", "rider")):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
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
            for lane in names:
                path = "bad.md" if lane == bad else {"first": "one.md", "rider": "two.md"}.get(lane, lane + ".md")
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
            for lane in names:
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
            self.notes = git("log", "--notes=land", "--format=%N", "main", cwd=remote)
            return out, files, git("ls-remote", remote, "refs/locks/*")

    def test_land_carries_queued_lanes_in_one_gated_train(self):
        out, files, locks = self._train("")
        self.assertEqual(out["first"][2], 0, out["first"][1])
        self.assertEqual(out["rider"][2], 0, out["rider"][1])
        self.assertIn("gating a train of 1 rider(s)", out["first"][1])
        self.assertIn("carried to main in a train", out["rider"][0])
        self.assertEqual(sorted(files), ["one.md", "two.md"])
        self.assertEqual(locks, "")
        logged = [json.loads(l) for l in self.notes.splitlines() if l.startswith("{")]
        self.assertEqual(sorted(e["lane"] for e in logged), ["first", "rider"])
        for e in logged:
            self.assertEqual((e["riders"], e["splits"], e["holder"]), (1, 0, "first"))
            self.assertGreaterEqual(e["wait_s"], 0)

    def test_land_drops_a_failing_train_and_lands_alone(self):
        out, files, locks = self._train("! test -e bad.md || exit 1\n", bad="rider")
        self.assertEqual(out["first"][2], 0, out["first"][1])
        self.assertIn("the train failed the gate", out["first"][1])
        self.assertNotEqual(out["rider"][2], 0)
        self.assertEqual(files, ["one.md"])
        self.assertIn("refs/locks/solo/", locks)

    def test_land_keeps_riders_off_solo_when_the_holder_broke_the_train(self):
        out, files, locks = self._train("! test -e bad.md || exit 1\n", bad="first")
        self.assertNotEqual(out["first"][2], 0)
        self.assertIn("the train failed the gate", out["first"][1])
        self.assertEqual(out["rider"][2], 0, out["rider"][1])
        self.assertEqual(files, ["two.md"])
        self.assertNotIn("refs/locks/solo/", locks)

    def test_land_splits_a_failing_train_and_lands_the_passing_prefix(self):
        lanes = ("first", "rider", "breaker", "after")
        out, files, locks = self._train("! test -e bad.md || exit 1\n", bad="breaker", names=lanes)
        self.assertEqual(out["first"][2], 0, out["first"][1])
        self.assertIn("breaker", out["first"][1].split("broke the train")[0].splitlines()[-1])
        self.assertIn("pushed", out["first"][0])
        # The lane behind the breaker lands in the same cycle, not on a turn of its own.
        self.assertIn("with 2 rider(s)", out["first"][0])
        self.assertIn("gating 2 rider(s) without", out["first"][1])
        self.assertNotEqual(out["breaker"][2], 0)
        self.assertEqual(out["after"][2], 0, out["after"][1])
        self.assertIn("carried to main in a train", out["after"][0])
        self.assertEqual(sorted(files), ["after.md", "one.md", "two.md"])
        self.assertNotIn("-after", locks)

    def test_land_drops_the_rider_the_failure_points_at_without_bisecting(self):
        lanes = ("first", "rider", "breaker", "after")
        gate = 'if test -e bad.md; then echo "  --> bad.md:1:1"; exit 1; fi\n'
        out, files, locks = self._train(gate, bad="breaker", names=lanes)
        self.assertEqual(out["first"][2], 0, out["first"][1])
        self.assertIn("the failure points at", out["first"][1])
        self.assertNotIn("gating the holder with", out["first"][1])
        self.assertIn("with 2 rider(s)", out["first"][0])
        self.assertNotEqual(out["breaker"][2], 0)
        self.assertEqual(out["after"][2], 0, out["after"][1])
        self.assertEqual(sorted(files), ["after.md", "one.md", "two.md"])

    def test_land_lock_heartbeat_keeps_live_gate_and_expires_after_max_hold(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
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
                f.write("#!/bin/sh\nsleep 4\n"
                        'git rev-parse "HEAD^{tree}" >> "$(git rev-parse --git-common-dir)/cpl-gate-passed"\n')
            os.chmod(gate, 0o755)
            work = os.path.join(tmp, "work")
            git("clone", "-q", remote, work)
            git("config", "user.name", "holder", cwd=work)
            git("config", "user.email", "t@example.com", cwd=work)
            with open(os.path.join(work, "a.md"), "w", encoding="utf-8") as f:
                f.write("a\n")
            git("add", "a.md", cwd=work)
            git("commit", "-qm", "docs: a", cwd=work)
            git("config", "core.hooksPath", os.path.join(root, ".githooks"), cwd=work)
            land = os.path.join(root, "scripts", "land.sh")
            # Gate sleeps 4s with TTL=2s and LOCK_BEAT=1s: heartbeat must refresh
            # the lock past TTL, then stop once MAX_HOLD=2s is reached.
            r = subprocess.run(
                [land, remote], cwd=work, capture_output=True, text=True,
                env={**env, "CPL_LAND_GATE": gate, "CPL_LANE": "holder",
                     "CPL_LAND_TTL": "2", "CPL_LAND_LOCK_BEAT": "1",
                     "CPL_LAND_MAX_HOLD": "2", "CPL_LAND_TRAIN": "0"})
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn("lock hold limit (2s) reached", r.stderr)
            self.assertEqual(git("ls-remote", remote, "refs/locks/land"), "")

    def test_land_never_pushes_main_after_losing_the_lock(self):
        # The gate stalls long enough to look dead and another lander takes
        # the lock: the stale holder's push must fail and it must retry.
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            remote = os.path.join(tmp, "remote.git")
            def git(*a, cwd=tmp):
                return subprocess.run(["git", *a], cwd=cwd, check=True, env=env,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q", "--bare", "-b", "main", remote)
            work = os.path.join(tmp, "work")
            git("init", "-q", "-b", "main", work)
            git("config", "user.name", "holder", cwd=work)
            git("config", "user.email", "t@example.com", cwd=work)
            git("commit", "-q", "--allow-empty", "-m", "docs: seed", cwd=work)
            git("push", "-q", remote, "HEAD:main", cwd=work)
            git("commit", "-q", "--allow-empty", "-m", "docs: a", cwd=work)
            gate = os.path.join(tmp, "gate.sh")
            with open(gate, "w", encoding="utf-8") as f:
                f.write("#!/bin/sh\n"
                        f'if [ ! -e "{tmp}/stolen" ]; then touch "{tmp}/stolen"\n'
                        f'  t=$(git commit-tree $(git hash-object -t tree /dev/null) -m "land lock: thief")\n'
                        f'  git push -q --force "{remote}" "$t:refs/locks/land"; fi\n'
                        'git rev-parse "HEAD^{tree}" >> "$(git rev-parse --git-common-dir)/cpl-gate-passed"\n')
            os.chmod(gate, 0o755)
            git("config", "core.hooksPath", os.path.join(root, ".githooks"), cwd=work)
            r = subprocess.run(
                [os.path.join(root, "scripts", "land.sh"), remote], cwd=work,
                capture_output=True, text=True, timeout=120,
                env={**env, "CPL_LAND_GATE": gate, "CPL_LANE": "holder", "CPL_LAND_TTL": "2",
                     "CPL_LAND_POLL": "1", "CPL_LAND_LOCK_BEAT": "0", "CPL_LAND_TRAIN": "0"})
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn("lost the land lock during the gate", r.stderr)
            self.assertIn("waiting for land lock: thief", r.stderr)
            self.assertIn("taking over stale lock from land lock: thief", r.stderr)
            self.assertEqual(git("rev-parse", "main", cwd=remote), git("rev-parse", "HEAD", cwd=work))
            self.assertEqual(git("ls-remote", remote, "refs/locks/*"), "")

    def test_land_queues_fairly_in_arrival_order(self):
        # With CPL_LAND_TRAIN=0, two queued lanes must land in strict FIFO order.
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
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
            order_log = os.path.join(tmp, "order.log")
            gate = os.path.join(tmp, "gate.sh")
            with open(gate, "w", encoding="utf-8") as f:
                f.write("#!/bin/sh\n"
                        f'git log -1 --format=%s >> "{order_log}"\n'
                        'git rev-parse "HEAD^{tree}" >> "$(git rev-parse --git-common-dir)/cpl-gate-passed"\n')
            os.chmod(gate, 0o755)
            lanes = {}
            for lane in ("first", "second"):
                work = os.path.join(tmp, lane)
                git("clone", "-q", remote, work)
                git("config", "user.name", lane, cwd=work)
                git("config", "user.email", "t@example.com", cwd=work)
                with open(os.path.join(work, f"{lane}.md"), "w", encoding="utf-8") as f:
                    f.write(lane + "\n")
                git("add", f"{lane}.md", cwd=work)
                git("commit", "-qm", f"docs: {lane}", cwd=work)
                git("config", "core.hooksPath", os.path.join(root, ".githooks"), cwd=work)
                lanes[lane] = work
            empty = git("hash-object", "-t", "tree", "-w", "/dev/null", cwd=seed)
            busy = git("commit-tree", empty, "-m", "land lock: other", cwd=seed)
            git("push", "-q", remote, f"{busy}:refs/locks/land", cwd=seed)
            land = os.path.join(root, "scripts", "land.sh")
            procs = {}
            for lane, poll in (("first", "2"), ("second", "1")):
                procs[lane] = subprocess.Popen(
                    [land, remote], cwd=lanes[lane], stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE, text=True,
                    env={**env, "CPL_LAND_GATE": gate, "CPL_LANE": lane,
                         "CPL_LAND_POLL": poll, "CPL_LAND_TRAIN": "0"})
                deadline = time.time() + 30
                while f"-{lane}" not in git("ls-remote", remote, "refs/locks/queue/*"):
                    self.assertLess(time.time(), deadline, f"{lane} never queued")
                    time.sleep(0.2)
            git("push", "-q", remote, ":refs/locks/land", cwd=seed)
            for lane, p in procs.items():
                out, err = p.communicate(timeout=60)
                self.assertEqual(p.returncode, 0, f"{lane}: {err}")
            with open(order_log, encoding="utf-8") as f:
                order = [line.strip() for line in f if line.strip()]
            self.assertEqual(order, ["docs: first", "docs: second"])

    def test_land_stops_with_the_hook_error_when_pre_push_rejects_head(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            remote, work = os.path.join(tmp, "remote.git"), os.path.join(tmp, "work")
            def git(*a, cwd=work):
                return subprocess.run(["git", *a], cwd=cwd, check=True, env=env,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q", "--bare", "-b", "main", remote, cwd=tmp)
            git("init", "-q", "-b", "main", work, cwd=tmp)
            git("config", "user.email", "t@example.com")
            git("config", "user.name", "t")
            git("commit", "-q", "--allow-empty", "-m", "docs: seed")
            git("push", "-q", remote, "HEAD:main")
            with open(os.path.join(work, "q.md"), "w", encoding="utf-8") as f:
                f.write("mine\n" + "|" * 7 + " parent of abc\ntheirs\n")
            git("add", "q.md")
            git("commit", "-qm", "plan: q")
            git("config", "core.hooksPath", os.path.join(root, ".githooks"))
            r = subprocess.run([os.path.join(root, "scripts", "land.sh"), remote], cwd=work,
                               capture_output=True, text=True, timeout=60,
                               env={**env, "CPL_LAND_GATE": "true", "CPL_LAND_POLL": "1"})
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("pre-push: merge conflict markers", r.stderr)
            self.assertIn("pre-push hook rejects HEAD", r.stderr)
            self.assertNotIn("heartbeat failed", r.stderr)
            self.assertEqual(git("ls-remote", remote, "refs/locks/*"), "")

    def test_append_only_shared_files_rebase_without_conflicts(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            def git(*a, cwd=tmp):
                return subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@example.com", *a],
                                      cwd=cwd, check=True, env=env,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q", "-b", "main")
            shutil.copy(os.path.join(root, ".gitattributes"), tmp)
            paths = ["plan/QUESTIONS.md", "plan/domains/infra/worklog.md"]
            for p in paths:
                os.makedirs(os.path.join(tmp, os.path.dirname(p)), exist_ok=True)
                with open(os.path.join(tmp, p), "w", encoding="utf-8") as f:
                    f.write("# log\n\n## Open\n\n## Answered\n")
            git("add", ".")
            git("commit", "-qm", "plan: seed")
            def append(branch, text):
                git("checkout", "-q", "-b", branch, "main")
                for p in paths:
                    path = os.path.join(tmp, p)
                    with open(path, encoding="utf-8") as f:
                        body = f.read()
                    with open(path, "w", encoding="utf-8") as f:
                        f.write(body.replace("## Answered", text + "\n\n## Answered"))
                git("commit", "-qam", f"plan: {branch}")
            append("a", "### a: first?")
            append("b", "### b: second?")
            git("checkout", "-q", "main")
            git("merge", "-q", "--ff-only", "a")
            git("checkout", "-q", "b")
            git("rebase", "-q", "main")
            for p in paths:
                with open(os.path.join(tmp, p), encoding="utf-8") as f:
                    body = f.read()
                self.assertIn("### a: first?", body)
                self.assertIn("### b: second?", body)
                self.assertNotIn("<" * 7, body)

    def test_land_rejects_unformatted_rust_before_queuing(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            work = os.path.join(tmp, "work")
            os.makedirs(os.path.join(work, "src"))
            def git(*a):
                return subprocess.run(["git", *a], cwd=work, check=True, env=env,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q", "-b", "main")
            git("config", "user.name", "t")
            git("config", "user.email", "t@example.com")
            with open(os.path.join(work, "Cargo.toml"), "w", encoding="utf-8") as f:
                f.write('[package]\nname = "t"\nversion = "0.1.0"\nedition = "2021"\n')
            with open(os.path.join(work, "src", "lib.rs"), "w", encoding="utf-8") as f:
                f.write("pub fn   f(x:i32)->i32{x+1}\n")
            git("add", ".")
            git("commit", "-qm", "fmt: bad")
            land = os.path.join(root, "scripts", "land.sh")
            r = subprocess.run([land, "origin"], cwd=work, capture_output=True, text=True, env=env)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("Rust files are not formatted", r.stderr)

if __name__ == "__main__":
    unittest.main()
