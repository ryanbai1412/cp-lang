import os
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from culprit import failing_tests, job_ranges, nextest_cmd, source_of, summary, triage

LOG = """\
2026-11-23T17:15:19.6Z        FAIL [   0.878s] (28/34) cpl-cli::runtime_errors wide_range_overflow_traps
2026-11-23T17:15:19.7Z        FAIL [   0.878s] (28/34) cpl-cli::runtime_errors wide_range_overflow_traps
2026-11-23T17:15:20.0Z     TIMEOUT [ 120.001s] cpl-lsp tests::parity::the_editor_shows_the_errors
2026-11-23T17:15:21.0Z        PASS [   0.100s] (29/34) cpl-cli::diagnostics golden_diagnostics
     Summary [  12.119s] 34 tests run: 33 passed, 1 failed, 468 skipped
"""


class CulpritTest(unittest.TestCase):
    def test_reads_each_failing_test_once(self):
        self.assertEqual(failing_tests(LOG), [
            ("cpl-cli::runtime_errors", "wide_range_overflow_traps"),
            ("cpl-lsp", "tests::parity::the_editor_shows_the_errors")])

    def test_runs_only_the_named_tests(self):
        cmd = nextest_cmd([("cpl-cli::runtime_errors", "a"), ("cpl-lsp", "b::c")])
        self.assertIn("(binary_id(cpl-cli::runtime_errors) & test(=a)) | (binary_id(cpl-lsp) & test(=b::c))", cmd)
        self.assertEqual(cmd[-4:], ["-p", "cpl-cli", "-p", "cpl-lsp"])

    def test_summary_names_the_culprit_and_its_revert(self):
        text = summary({"good": "a" * 40, "bad": "b" * 40, "flaky": ["x y"],
                        "already_failing": [], "unbisected": [],
                        "culprits": [{"sha": "c" * 40, "subject": "check: s", "domain": "compiler",
                                      "tests": ["t u"], "revert": {"clean": True, "passes": True,
                                                                   "branch": "culprit/revert-ccccccccc"}}]})
        self.assertIn("ccccccccc (compiler) check: s: 1 test(s), revert passes, on culprit/revert-ccccccccc", text)
        self.assertIn("flaky: x y", text)


    def test_a_red_job_ranges_from_its_own_last_pass_when_no_run_is_green(self):
        with tempfile.TemporaryDirectory() as root:
            def run(*args):
                return subprocess.run(args, cwd=root, check=True, capture_output=True, text=True).stdout.strip()

            def commit(msg):
                run("git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", msg)
                return run("git", "rev-parse", "HEAD")

            run("git", "init", "-q")
            c1 = commit("a: one\n\nLane: A")
            c2 = commit("b: two\n\nLane: B")
            c3 = commit("c: three")
            runs = [{"id": 1, "head_sha": c1}, {"id": 2, "head_sha": c2}, {"id": 3, "head_sha": c3}]
            jobs = {1: [{"name": "site", "conclusion": "success"}, {"name": "tests (1)", "conclusion": "failure"}],
                    2: [{"name": "site", "conclusion": "failure"}, {"name": "tests (1)", "conclusion": "success"}],
                    3: [{"name": "site", "conclusion": "failure"}, {"name": "tests (1)", "conclusion": "failure"},
                        {"name": "gate", "conclusion": "failure"}]}
            got = {j["job"]: j for j in job_ranges(3, c3, root, runs=runs, jobs_of=jobs.__getitem__)}
            self.assertEqual(got["site"]["good"], c1)
            self.assertEqual([(c["sha"], c["lane"]) for c in got["site"]["commits"]], [(c3, "no lane"), (c2, "B")])
            self.assertEqual((got["tests (1)"]["good"], got["tests (1)"]["count"]), (c2, 1))
            self.assertIsNone(got["gate"]["good"])
            text = summary({"good": None, "bad": c3, "jobs": list(got.values())})
            self.assertIn(f"- site: red since {c1[:9]}, 2 commit(s)", text)
            self.assertIn(f"{c2[:9]} (B) b: two", text)
            self.assertIn("- gate: red in every finished main run checked (the last 40); no range", text)

    def test_names_each_tests_suspects_before_any_bisect(self):
        with tempfile.TemporaryDirectory() as root:
            def run(*args):
                return subprocess.run(args, cwd=root, check=True, capture_output=True, text=True).stdout.strip()

            def commit(path, text, msg):
                full = os.path.join(root, path)
                os.makedirs(os.path.dirname(full), exist_ok=True)
                with open(full, "w") as f:
                    f.write(text)
                run("git", "add", path)
                run("git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", msg)
                return run("git", "rev-parse", "HEAD")

            run("git", "init", "-q")
            good = commit("crates/a/tests/it.rs", "#[test]\nfn hidden() {}\n", "a: start\n\nDomain: tools")
            near = commit("crates/a/tests/it.rs", "#[test]\nfn hidden() { 1; }\n", "a: touch it\n\nDomain: perf")
            commit("crates/a/src/lib.rs", "mod t { #[test]\nfn unit() {} }\n", "a: add unit\n\nDomain: compiler")
            bad = commit("crates/b/src/lib.rs", "", "b: elsewhere\n\nDomain: site")
            self.assertEqual(source_of(("a::it", "hidden"), root), "crates/a/tests/it.rs")
            self.assertEqual(source_of(("a", "t::unit"), root), "crates/a/src/lib.rs")
            self.assertIsNone(source_of(("a", "gone"), root))
            got = triage(good, bad, [("a::it", "hidden"), ("a", "gone")], root)
            self.assertEqual([s["sha"] for s in got[0]["suspects"]], [near])
            self.assertEqual(got[0]["suspects"][0]["domain"], "perf")
            self.assertEqual([s["domain"] for s in got[1]["suspects"]], ["compiler", "perf"])
            text = summary({"good": good, "bad": bad, "triage": got})
            self.assertIn(f"- a::it hidden [crates/a/tests/it.rs]: {near[:9]} (perf)", text)
            self.assertIn("- a gone [file not found]:", text)


if __name__ == "__main__":
    unittest.main()
