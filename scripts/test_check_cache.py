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
        reads = cc.build_reads(root)
        self.assertIn("std/README.md", reads)
        self.assertIn("editors/lsp-matrix.json", reads)
        always = ["std/prelude.cp", "Cargo.toml", "Cargo.lock", "crates/cpl-cli/src/main.rs"]
        for stage, pats in cc.STAGES.items():
            for path in always + sorted(reads):
                self.assertFalse(cc.excluded(path, pats, reads), f"{stage} excludes {path}, which the build reads")

    def test_build_reads_come_from_includes_dep_info_and_rerun_if_changed(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.git_tree(tmp, {
                "crates/a/src/lib.rs": 'const M: &str = include_str!("../../../docs/m.json");\n',
                "docs/m.json": "{}\n", "docs/dep.md": "x\n", "docs/other.md": "x\n",
                "lints/x/lint.toml": "x\n",
                "target/debug/deps/a-1.d": f"{tmp}/target/debug/deps/a-1: {tmp}/crates/a/src/lib.rs {tmp}/docs/dep.md /elsewhere/x.rs\n",
                "target/debug/build/a/9f/run/stdout": f"cargo:rerun-if-changed={tmp}/lints\ncargo:rustc-cfg=x\n",
            })
            reads = cc.build_reads(tmp)
            for path in ["docs/m.json", "docs/dep.md", "lints"]:
                self.assertIn(path, reads)
            self.assertNotIn("docs/other.md", reads)
            pats = ["docs/*", "lints/*"]
            self.assertFalse(cc.excluded("docs/m.json", pats, reads))
            self.assertFalse(cc.excluded("lints/x/lint.toml", pats, reads))
            self.assertTrue(cc.excluded("docs/other.md", pats, reads))
            env = "env\n" + cc.manifest(tmp).split("\n", 1)[1]
            before = cc.stage_key(env, "clippy", pats, reads)
            with open(os.path.join(tmp, "docs/dep.md"), "a", encoding="utf-8") as f:
                f.write("edit\n")
            after = "env\n" + cc.manifest(tmp).split("\n", 1)[1]
            self.assertNotEqual(cc.stage_key(after, "clippy", pats, reads), before)

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

    def land_repos(self, tmp, env):
        """A bare remote with one commit on main and a clone of it."""
        remote, work = os.path.join(tmp, "remote.git"), os.path.join(tmp, "work")
        def git(*a, cwd=work):
            return subprocess.run(["git", *a], cwd=cwd, check=True, env=env,
                                  capture_output=True, text=True).stdout.strip()
        git("init", "-q", "--bare", "-b", "main", remote, cwd=tmp)
        git("init", "-q", "-b", "main", work, cwd=tmp)
        with open(os.path.join(work, "a.md"), "w", encoding="utf-8") as f:
            f.write("a\n")
        git("add", "a.md")
        git("commit", "-qm", "docs: a")
        git("push", "-q", remote, "HEAD:main")
        git("remote", "add", "origin", remote)
        return remote, work, git

    def commit(self, git, work, name, body="docs: x\n\nDomain: infra"):
        with open(os.path.join(work, name), "w", encoding="utf-8") as f:
            f.write(name + "\n")
        git("add", name)
        git("commit", "-qm", body)

    def test_land_rebases_onto_a_moved_main_pushes_and_logs_its_duration(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            remote, work, git = self.land_repos(tmp, env)
            other = os.path.join(tmp, "other")
            git("clone", "-q", remote, other, cwd=tmp)
            self.commit(lambda *a: git(*a, cwd=other), other, "b.md")
            git("push", "-q", "origin", "HEAD:main", cwd=other)
            self.commit(git, work, "c.md")
            t0 = time.monotonic()
            r = subprocess.run([os.path.join(root, "scripts", "land.sh")], cwd=work,
                               capture_output=True, text=True, env=env)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertLess(time.monotonic() - t0, 10)
            self.assertEqual(git("rev-parse", "main", cwd=remote), git("rev-parse", "HEAD"))
            self.assertEqual(git("log", "--format=%s", "main", cwd=remote).count("docs: x"), 2)
            with open(os.path.join(work, ".git", "cpl-land.log"), encoding="utf-8") as f:
                row = json.loads(f.readline())
            self.assertEqual((row["domain"], row["tries"]), ("infra", 1))
            self.assertIsInstance(row["seconds"], int)

    def test_land_retries_when_the_push_is_rejected(self):
        # A pre-receive hook rejects the first push, as when another lane
        # lands between our rebase and our push.
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            remote, work, git = self.land_repos(tmp, env)
            hook = os.path.join(remote, "hooks", "pre-receive")
            with open(hook, "w", encoding="utf-8") as f:
                f.write(f'#!/bin/sh\n[ -e "{tmp}/once" ] && exit 0\ntouch "{tmp}/once"; exit 1\n')
            os.chmod(hook, 0o755)
            self.commit(git, work, "c.md")
            r = subprocess.run([os.path.join(root, "scripts", "land.sh")], cwd=work,
                               capture_output=True, text=True, env=env)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn("push rejected (try 1 of 5)", r.stderr)
            self.assertEqual(git("rev-parse", "main", cwd=remote), git("rev-parse", "HEAD"))

    def test_land_refuses_a_commit_without_a_domain_trailer(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        env = hermetic_env()
        with tempfile.TemporaryDirectory() as tmp:
            remote, work, git = self.land_repos(tmp, env)
            self.commit(git, work, "c.md")
            self.commit(git, work, "d.md", body="docs: no trailer")
            r = subprocess.run([os.path.join(root, "scripts", "land.sh")], cwd=work,
                               capture_output=True, text=True, env=env)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("docs: no trailer has no Domain: trailer", r.stderr)
            self.assertNotEqual(git("rev-parse", "main", cwd=remote), git("rev-parse", "HEAD"))

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

if __name__ == "__main__":
    unittest.main()
