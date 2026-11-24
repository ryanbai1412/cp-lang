import importlib.util
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "std_api_lint", Path(__file__).resolve().parent / "std-api-lint.py")
lint = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(lint)

PT = """pub struct Pt:
    x: Int
    y: Int

    pub fn norm2(self) -> Int = x * x + y * y
"""


class BothForms(unittest.TestCase):
    def test_free_fn_alone_is_accepted(self):
        text = PT + "\npub fn dot(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y\n"
        self.assertEqual(list(lint.both_forms([("g.cp", text)])), [])

    def test_free_fn_and_method_is_rejected(self):
        text = PT + "    pub fn dot(self, o: Pt) -> Int = x * o.x + y * o.y\n" \
            "\npub fn dot(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y\n"
        found = list(lint.both_forms([("g.cp", text)]))
        self.assertEqual(len(found), 1)
        self.assertIn("g.cp:8: std-api/R11: `dot`", found[0])


if __name__ == "__main__":
    unittest.main()
