#!/usr/bin/env python3
"""One test per gate behind scripts/spec-gate.py."""
import importlib.util
import os
import pathlib
import sys
import tempfile
import unittest
from unittest import mock

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
SPEC = importlib.util.spec_from_file_location("spec_gate", HERE / "spec-gate.py")
assert SPEC and SPEC.loader
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)

import specgate  # noqa: E402


class Tree:
    """A scratch `spec/conformance/` the gates read instead of the repo's."""

    def __init__(self, files: dict[str, str]):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        self.conf = self.root / "spec" / "conformance"
        for rel, text in files.items():
            p = self.root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(text.encode())
        self.patches = [mock.patch.object(specgate, "ROOT", self.root),
                        mock.patch.object(specgate, "CONFORMANCE", self.conf)]

    def __enter__(self):
        for p in self.patches:
            p.start()
        return self

    def __exit__(self, *exc):
        for p in self.patches:
            p.stop()
        self.tmp.cleanup()

    def read(self, rel: str) -> str:
        return (self.root / rel).read_bytes().decode()


def run(name: str, *argv: str):
    import argparse
    gate = GATE.load(name)
    p = argparse.ArgumentParser()
    gate.args(p)
    return gate.run(p.parse_args(list(argv)))


class GateListTest(unittest.TestCase):
    def test_every_gate_has_a_test_here(self):
        tested = {n[len("test_"):].replace("_", "-") for n in dir(GatesTest) if n.startswith("test_")}
        tested |= {t.split("-rejects")[0] for t in tested}
        self.assertEqual(set(GATE.GATES) - tested, set())


class GatesTest(unittest.TestCase):
    def test_markers(self):
        old = "// expect: E0601 at 2 (checked)\n// rule: 05.1\n// a note\nlet a = 1\r\nf(a)\n"
        with Tree({"spec/conformance/c/a.cp": old}) as t:
            self.assertEqual(len(run("markers")), 1)
            self.assertEqual(run("markers", "--fix"), [])
            self.assertEqual(t.read("spec/conformance/c/a.cp"),
                             "// expect: E0601 (checked)\n// rule: 05.1\n// a note\nlet a = 1 // <- here\r\nf(a)\n")
            self.assertEqual(run("markers"), [])

    def test_gaps(self):
        files = {"spec/conformance/c/f.cp": "// expect: ok\n// gap: compiler 2026-11-24T05Z\nx\n",
                 "spec/conformance/c/g.cp": "// expect: ok\nx\n",
                 "spec/conformance/c/h.cp": "// expect: ok\nx\n// gap: in the body is program text\n"}
        with Tree(files):
            self.assertEqual([p.where for p in run("gaps")], ["c/f.cp"])

    def test_subset(self):
        from specgate import subset
        files = {"spec/core/01-x.md": "# 1. Lexing\n\n## 1.5 Literals\n\nA rule (R01.5.1).\n",
                 "spec/core/02-y.md": "## 2.1 Types\n\nA rule (R02.1.1).\n",
                 "spec/conformance/c/f.cp": "// expect: ok\n// rule: 01.5.1\nx\n",
                 "research/scoreboard.json": '{"suites": {"conformance": {"cases": {"c/f.cp": "fail"}}}}',
                 "spec/SUBSET.md": "| R02.1 | 2.1 Types | documented | later |\n"}
        with Tree(files) as t, mock.patch.object(subset, "SUBSET", t.root / "spec/SUBSET.md"), \
                mock.patch.object(subset, "SPEC", t.root / "spec"):
            self.assertEqual([p.what for p in run("subset")][-1][:8], "is stale")
            self.assertEqual(run("subset", "--write"), [])
            text = t.read("spec/SUBSET.md")
            self.assertIn("| R01.5 | 1.5 Literals | supported | 1 known gap |", text)
            self.assertIn("| R02.1 | 2.1 Types | documented | later |", text)
            self.assertIn("- c/f.cp: fail", text)
            self.assertEqual(run("subset"), [])


if __name__ == "__main__":
    unittest.main()
