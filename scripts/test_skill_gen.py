import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location("skill_gen", ROOT / "scripts/skill-gen.py")
assert SPEC is not None and SPEC.loader is not None
skill_gen = importlib.util.module_from_spec(SPEC)
sys.modules["skill_gen"] = skill_gen
SPEC.loader.exec_module(skill_gen)


class SkillGenTests(unittest.TestCase):
    def test_generates_all_spec_std_modules(self) -> None:
        files = skill_gen.generate_all()
        expected = {
            skill_gen.STDLIB_OUT / f"{p.stem}.md"
            for p in (ROOT / "spec/std").glob("*.cp")
            if p.stem != "prelude"
        }
        expected.add(skill_gen.STDLIB_OUT / "README.md")
        self.assertEqual(set(files.keys()), expected)

    def test_drift_names_a_stale_page(self) -> None:
        import tempfile
        saved = skill_gen.STDLIB_OUT
        with tempfile.TemporaryDirectory() as d:
            files = skill_gen.generate_all()
            out = Path(d)
            for path, content in files.items():
                (out / path.name).write_text(content)
            skill_gen.STDLIB_OUT = out
            try:
                orig = skill_gen.generate_all
                skill_gen.generate_all = lambda: {out / p.name: c for p, c in files.items()}
                self.assertEqual(skill_gen.drift_errors(), [])
                (out / "seq.md").write_text("old")
                self.assertTrue(any("seq.md" in e and "stale" in e for e in skill_gen.drift_errors()))
            finally:
                skill_gen.generate_all = orig
                skill_gen.STDLIB_OUT = saved

    def test_read_header_preserves_default_args(self) -> None:
        lines = [
            "    pub fn new(a: ?[D]T = None, fill: ?T = None) -> Foo<D, T> = todo",
        ]
        sig = skill_gen.read_header(lines, 0)
        self.assertEqual(sig, "pub fn new(a: ?[D]T = None, fill: ?T = None) -> Foo<D, T>")

    def test_forbidden_regex_rejects_legacy_terms_but_allows_skill_links(self) -> None:
        self.assertIsNotNone(skill_gen.FORBIDDEN_RE.search("set CPL_SPEC=1"))
        self.assertIsNotNone(skill_gen.FORBIDDEN_RE.search("CPL_EDITION=0.2"))
        self.assertIsNotNone(skill_gen.FORBIDDEN_RE.search("see stdlib/seq.cp"))
        self.assertIsNone(skill_gen.FORBIDDEN_RE.search("see spec/std/seq.cp"))
        self.assertIsNone(skill_gen.FORBIDDEN_RE.search("see stdlib/seq.md"))

    def test_parse_output_lines(self) -> None:
        code = "print(1)\n// output: 1\n// output: 2 3\n"
        stdin, outputs = skill_gen.parse_io_annotations(code)
        self.assertIsNone(stdin)
        self.assertEqual(outputs, ["1", "2 3"])

    def test_skill_md_front_matter_and_length(self) -> None:
        skill = ROOT / "skills/cp-lang/SKILL.md"
        text = skill.read_text()
        self.assertTrue(text.startswith("---\n"))
        self.assertIn("name: cp-lang", text)
        self.assertLessEqual(len(text.splitlines()), skill_gen.MAX_SKILL_LINES)


if __name__ == "__main__":
    unittest.main()
