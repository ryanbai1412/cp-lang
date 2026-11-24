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
    def test_changelog(self):
        files = {"spec/diagnostics.md": "| E0503 | removed | `a.size` is `a.len`, `pop_front` is `pop` |\n",
                 "CHANGELOG.md": "# Changelog\n\n## 0.3\n\n- `a.size` is gone\n\n## 0.2\n\n- `pop_front`\n"}
        with Tree(files):
            self.assertEqual([p.what.split(" ")[0] for p in run("changelog")], ["`pop_front`"])

    def test_decisions_explained(self):
        files = {"spec/core/a.md": "Per decision 12 and decision 345.\n",
                 "handbook/language/b.md": "Decision 345 chose this.\n"}
        with Tree(files):
            self.assertEqual([p.where for p in run("decisions-explained")], ["decision 12"])

    def test_vocab(self):
        files = {"std/a.cp": "pub fn f() -> ?Int = some(1)\n/// some(x) in a doc is prose\n",
                 "spec/core/b.md": "Mint in prose\n```\nlet m: Mint = 1\n```\n"}
        with Tree(files):
            self.assertEqual(sorted(p.where for p in run("vocab")), ["spec/core/b.md:1", "spec/core/b.md:3", "std/a.cp:1"])

    def test_xref(self):
        import subprocess
        files = {"spec/core/01-a.md": "A rule (R01.1.1). A permit (R01.1.3, permits).\n",
                 "handbook/x.md": "See R01.1.1, R01.1.3 and R01.1.2.\n"}
        with Tree(files) as t:
            subprocess.run(["git", "init", "-q"], cwd=t.root, check=True)
            subprocess.run(["git", "add", "."], cwd=t.root, check=True)
            self.assertEqual([(p.where, p.what[:7]) for p in run("xref")], [("handbook/x.md:1", "R01.1.2")])

    def test_flakes(self):
        from specgate import flakes
        verdicts = iter([{"a.cp", "b.cp"}, {"a.cp"}])
        with mock.patch.object(flakes, "failing", lambda f: next(verdicts)):
            self.assertEqual([(p.where, p.what) for p in run("flakes")], [("b.cp", "failed on 1 of 2 runs")])

    def test_speed(self):
        with Tree({}):
            self.assertEqual(run("speed", "--cpl-conform", "true", "--budget", "1e9"), [])
            self.assertEqual(len(run("speed", "--cpl-conform", "true", "--budget", "-1")), 1)
            self.assertIn("exited 2", run("speed", "--cpl-conform", "sh", "--budget", "1e9")[0].what)

    def test_tool_rules(self):
        files = {"spec/tools/a.md": "- `cpl x` does x (Rx.1).\n- `cpl y` does\n  y.\n\n```\n- not a feature\n```\n"}
        with Tree(files):
            self.assertEqual([p.where for p in run("tool-rules")], ["spec/tools/a.md:2"])

    def test_widths(self):
        from specgate import widths
        self.assertEqual(widths.exact("div", -7, 2), -4)
        self.assertEqual(widths.wrap(2**63), -2**63)
        with Tree({"spec/conformance/widths/old.cp": "x"}) as t:
            self.assertEqual(len(run("widths")), len(widths.cases()) + 1)
            self.assertEqual(run("widths", "--write"), [])
            self.assertFalse((t.conf / "widths/old.cp").exists())
            self.assertIn("print(id(1) + id(2))", t.read("spec/conformance/widths/add-in-range.cp"))
            self.assertEqual(run("widths"), [])

    def test_matrix(self):
        toml = ('[m.col.c]\ncode = "print($v)"\nout = "$v"\n[m.col.d]\ncode = "print($v + 1)"\nout = "$w"\n'
                '[m.row.r]\nv = 2\nw = 3\n[m.row.s]\nv = 5\n[m.na]\n"s--d" = "no w"\n')
        with Tree({"spec/conformance/matrix.toml": toml, "spec/conformance/matrix/m/old.cp": "x"}) as t:
            self.assertEqual(sorted(p.where.split("/")[-1] for p in run("matrix")), ["old.cp", "r--c.cp", "r--d.cp", "s--c.cp"])
            self.assertEqual(run("matrix", "--write"), [])
            self.assertEqual(t.read("spec/conformance/matrix/m/r--d.cp"), '// expect: ok, output "3"\nprint(2 + 1)\n')
            self.assertFalse((t.conf / "matrix/m/old.cp").exists())
            self.assertEqual(run("matrix"), [])

    def test_ownership(self):
        from specgate import ownership
        cases = ownership.cases()
        self.assertTrue(cases["top-temp-let.cp"].startswith("// expect: E0603\n"))
        self.assertTrue(cases["fn-named-let.cp"].startswith('// expect: ok, output "4"\n'))
        with Tree({}) as t:
            self.assertEqual(len(run("ownership")), len(cases))
            self.assertEqual(run("ownership", "--write"), [])
            self.assertEqual(run("ownership"), [])

    def test_traits(self):
        files = {"std/a.cp": "pub trait Shout:\n    fn shout(self) -> Str\n\npub struct Dog(n: Int)\nimpl Shout for Dog:\n    fn shout(self) -> Str = \"woof\"\n",
                 "spec/conformance/traits.toml": '[case."Shout/Dog/shout"]\ncode = "print(Dog(1).shout())"\nout = "woof"\n'}
        with Tree(files) as t:
            self.assertEqual([p.where.split("/")[-1] for p in run("traits")], ["Dog--shout.cp"])
            self.assertEqual(run("traits", "--write"), [])
            self.assertEqual(t.read("spec/conformance/traits/Shout/Dog--shout.cp"), '// expect: ok, output "woof"\nprint(Dog(1).shout())\n')
        with Tree({"std/a.cp": files["std/a.cp"], "spec/conformance/traits.toml": ""}):
            self.assertEqual([p.what for p in run("traits")], ["no case: Shout/Dog/shout (write one)"])

    def test_markers(self):
        old = "// expect: E0601 at 2 (checked)\n// rule: 05.1\n// a note\nlet a = 1\r\nf(a)\n"
        with Tree({"spec/conformance/c/a.cp": old}) as t:
            self.assertEqual(len(run("markers")), 1)
            self.assertEqual(run("markers", "--fix"), [])
            self.assertEqual(t.read("spec/conformance/c/a.cp"),
                             "// expect: E0601 (checked)\n// rule: 05.1\n// a note\nlet a = 1 // <- here\r\nf(a)\n")
            self.assertEqual(run("markers"), [])

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
