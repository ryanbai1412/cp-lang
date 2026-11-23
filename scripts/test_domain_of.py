"""Tests for scripts/domain_of.py and the commit-msg hook that uses it."""

import os
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from domain_of import domain_of  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


class DomainOf(unittest.TestCase):
    def test_the_one_lane_worklog_a_change_touches_names_its_domain(self):
        self.assertEqual(domain_of(["crates/cpl-codegen/src/lib.rs", "plan/domains/perf/worklog.md"]), "perf")

    def test_two_lanes_or_none_is_unsure(self):
        self.assertIsNone(domain_of(["plan/domains/perf/worklog.md", "plan/domains/spec/worklog.md"]))
        self.assertIsNone(domain_of(["std/graph.cp"]))
        self.assertIsNone(domain_of(["plan/domains/TEMPLATE/README.md"]))

    def test_a_configured_domain_wins(self):
        self.assertEqual(domain_of(["plan/domains/perf/worklog.md"], "stdlib"), "stdlib")
        self.assertEqual(domain_of(["plan/domains/perf/worklog.md"], "nonsense"), "perf")

    def test_the_hook_adds_a_missing_trailer_and_keeps_one_given(self):
        with tempfile.TemporaryDirectory() as tmp:
            # A commit hook exports GIT_DIR and GIT_INDEX_FILE; inheriting them
            # would commit into the repo running the hook.
            env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_") and k != "CPL_DOMAIN"}
            env.update(GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1")

            def git(*a):
                return subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@example.com",
                                       "-c", f"core.hooksPath={tmp}/hooks", *a], cwd=tmp, env=env,
                                      check=True, capture_output=True, text=True)
            git("init", "-q")
            os.makedirs(f"{tmp}/hooks")
            os.makedirs(f"{tmp}/scripts")
            os.makedirs(f"{tmp}/plan/domains/perf")
            for src, dst in ((".githooks/commit-msg", "hooks/commit-msg"), ("scripts/domain_of.py", "scripts/domain_of.py")):
                with open(os.path.join(ROOT, src), encoding="utf-8") as f, open(f"{tmp}/{dst}", "w", encoding="utf-8") as g:
                    g.write(f.read())
            os.chmod(f"{tmp}/hooks/commit-msg", 0o755)
            with open(f"{tmp}/plan/domains/perf/worklog.md", "w", encoding="utf-8") as f:
                f.write("x\n")
            git("add", "plan")
            git("commit", "-q", "-m", "codegen: lend a row")
            self.assertEqual(git("log", "-1", "--format=%(trailers:key=Domain,valueonly)").stdout.strip(), "perf")
            with open(f"{tmp}/plan/domains/perf/worklog.md", "a", encoding="utf-8") as f:
                f.write("y\n")
            git("commit", "-qam", "codegen: lend a column", "-m", "Domain: compiler")
            self.assertEqual(git("log", "-1", "--format=%(trailers:key=Domain,valueonly)").stdout.strip(), "compiler")
            with open(f"{tmp}/other.md", "w", encoding="utf-8") as f:
                f.write("z\n")
            git("add", "other.md")
            r = git("commit", "-q", "-m", "docs: say why")
            self.assertIn("no Domain: trailer", r.stderr)


if __name__ == "__main__":
    unittest.main()
