#!/usr/bin/env python3
"""Unit tests for scripts/check-usecases.py."""
import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "check-usecases.py")
SPEC = importlib.util.spec_from_file_location("check_usecases", SCRIPT)
assert SPEC and SPEC.loader
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)

ENTRY = """### {id}: {title}
Need: a need.
Design: a design.
Milestone: {milestone}. Status: {status}
Evidence: {evidence}
"""


def entry(id_, status="supported", milestone="M1-critical", evidence="integration `tests/a.rs`", title="t"):
    return ENTRY.format(id=id_, status=status, milestone=milestone, evidence=evidence, title=title)


class CheckUsecasesTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "tests").mkdir()
        (self.root / "tests/a.rs").write_text("#[test]\nfn finds_the_bug() {}\n")
        (self.root / "tests/gaps.txt").write_text("x\n")
        (self.root / "handbook/product/x").mkdir(parents=True)
        (self.root / "scripts").mkdir()
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, *entries):
        (self.root / "handbook/product/x/x-usecases.md").write_text("# X\n\n" + "\n".join(entries))

    def commit(self):
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "x"],
                       cwd=self.root, check=True)

    def run_check(self):
        entries, errors, state = MOD.load(self.root)
        return {e["id"]: e for e in entries}, errors, state

    def test_evidence_backs_the_status(self):
        self.write(
            entry("x/U1"),
            entry("x/U2", evidence="trial: a contest friction"),
            entry("x/U3", status="partial", evidence="list `tests/gaps.txt`"),
            entry("x/U4", status="gap", evidence="gap"),
            entry("x/U5", evidence="`tests/a.rs` (`finds_the_bug`)"),
        )
        by_id, errors, _ = self.run_check()
        self.assertEqual(errors, [])
        self.assertEqual({k: v["effective"] for k, v in by_id.items()},
                         {"x/U1": "supported", "x/U2": "gap", "x/U3": "gap", "x/U4": "gap", "x/U5": "supported"})

    def test_missing_evidence_test_field_and_status_are_errors(self):
        self.write(
            entry("x/U1", evidence="`tests/gone.rs`"),
            entry("x/U2", evidence="`tests/a.rs` (`no_such_test`)"),
            entry("x/U3", status="done"),
            "### x/U4: no fields\nNeed: n.\n",
            entry("x/U5", evidence="like `x/U9`"),
            "### x/5: bad heading\n",
        )
        _, errors, _ = self.run_check()
        text = "\n".join(errors)
        for want in ("no such evidence tests/gone.rs", "no such evidence tests/a.rs (no_such_test)",
                     "unknown status done", "x/U4: missing Design, Milestone, Evidence",
                     "no such use case x/U9", "heading isn't"):
            self.assertIn(want, text)

    def test_ids_are_unique_and_never_reused(self):
        self.write(entry("x/U1"), entry("x/U2"), entry("x/U3"))
        self.commit()
        self.write(entry("x/U1"), entry("x/U1", title="again"), entry("x/U3"))
        _, errors, _ = self.run_check()
        self.assertTrue(any("x/U1 is also" in e for e in errors), errors)
        self.write(entry("x/U1"), entry("x/U3"))
        self.commit()
        (self.root / "scripts/usecases.json").write_text(json.dumps({"baseline": [], "high_water": {"x": 3}}))
        self.write(entry("x/U1"), entry("x/U2"), entry("x/U3"), entry("x/U4"))
        _, errors, _ = self.run_check()
        self.assertEqual(errors, ["handbook/product/x/x-usecases.md:9: x/U2 reuses a number; the next is U4"])

    def test_baseline_only_goes_down_and_readiness_counts_new_gaps(self):
        self.write(entry("x/U1", status="gap", evidence="gap"), entry("x/U2", status="partial"),
                   entry("x/U3", status="gap", milestone="later", evidence="gap"))
        self.assertEqual(MOD.main(["--update"], self.root), 0)
        self.assertEqual(json.loads((self.root / "scripts/usecases.json").read_text())["baseline"],
                         ["x/U1", "x/U2"])
        self.write(entry("x/U1"), entry("x/U2", status="partial"), entry("x/U3", status="gap", evidence="gap"),
                   entry("x/U4", status="gap", evidence="gap"))
        by_id, _, state = self.run_check()
        n = MOD.numbers(list(by_id.values()), state)["all"]
        self.assertEqual((n["baseline"], n["readiness"]), (1, 3))
        MOD.main(["--update"], self.root)
        self.assertEqual(json.loads((self.root / "scripts/usecases.json").read_text())["baseline"], ["x/U2"])

    def test_repo_catalogues_check(self):
        self.assertEqual(MOD.main(["--check"]), 0)


if __name__ == "__main__":
    unittest.main()
