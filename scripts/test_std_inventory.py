import importlib.util
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "std_inventory", Path(__file__).resolve().parent / "std-inventory.py")
inv = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(inv)

KNOWN = {"first_ge", "Heap", "scan", "SegTree"}


class Inventory(unittest.TestCase):
    def test_documented_spellings_pass(self):
        text = "| C++ | `lower_bound` | `a.first_ge(x)` |\n| Python | `heapq` | `Heap` |\n"
        self.assertEqual(inv.findings(text, KNOWN), [])

    def test_operators_and_params_are_not_names(self):
        text = "| ACL | `lazy_segtree` | `SegTree(op, a, act: f)`, `a.scan(+)` |\n"
        self.assertEqual(inv.findings(text, KNOWN), [])

    def test_an_undocumented_name_fails(self):
        text = "| C++ | `upper_bound` | `a.first_gt(x)` |\n"
        self.assertEqual(len(inv.findings(text, KNOWN)), 1)

    def test_a_row_with_no_spelling_fails(self):
        text = "| Rust | `retain` | none |\n"
        self.assertIn("no CP Lang spelling", inv.findings(text, KNOWN)[0])


if __name__ == "__main__":
    unittest.main()
