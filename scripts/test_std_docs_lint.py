import importlib.util
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "std_docs_lint", Path(__file__).resolve().parent / "std-docs-lint.py")
lint = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(lint)

HEAD = "//! Bags.\n"


def found(body):
    return list(lint.label_findings([("std/bag.cp", HEAD + body)]))


class Labels(unittest.TestCase):
    def test_a_documented_sequence_fn_is_accepted(self):
        body = ("/// The total.\n///\n/// empty: `0`\n/// example: `[1, 2].total() == 3`\n"
                "/// example: `[1, 2].total(from: 1) == 4`\n"
                "@lint(x, from: \"a(b)\", to: \"c\")\npub fn total<D>(self: [D]Int, from: Int = 0) -> Int = 0\n")
        self.assertEqual(found(body), [])

    def test_a_sequence_fn_without_empty_is_rejected(self):
        self.assertIn("std-docs/R2: `total`", found("/// The total.\npub fn total<D>(self: [D]Int) -> Int = 0\n")[0])

    def test_an_option_first_example_is_rejected(self):
        body = ("/// Text.\n/// example: `5.text(base: 2) == \"101\"`\n/// example: `5.text() == \"5\"`\n"
                "pub fn text(self: Int, base: Int = 10) -> Str = \"\"\n")
        self.assertIn("std-docs/R7: `text`'s first example passes `base:`", found(body)[0])

    def test_options_in_every_example_are_accepted(self):
        body = ("/// Keep.\n/// example: `g.only(keep: f)`\n/// example: `g.only(from: f)`\n"
                "pub fn only(self: G, keep: Int = 0, from: Int = 0) -> G = self\n")
        self.assertEqual(found(body), [])

    def test_a_module_without_its_job_line_is_rejected(self):
        self.assertIn("std-docs/R10", list(lint.label_findings([("std/bag.cp", "use std.x\n")]))[0])


if __name__ == "__main__":
    unittest.main()
