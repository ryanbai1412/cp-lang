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


class ReadingOrder(unittest.TestCase):
    def test_same_order_is_accepted(self):
        text = "pub fn put(mut self, k: K, v: V)\npub fn put(mut self, k: K, v: V, at: Int = 0)\n"
        self.assertEqual(list(lint.reading_order([("m.cp", text)])), [])

    def test_a_shared_name_of_another_type_is_another_parameter(self):
        text = "pub fn set(mut self, k: K, v: V)\npub fn set(mut self, v: D, k: K)\n"
        self.assertEqual(list(lint.reading_order([("m.cp", text)])), [])

    def test_swapped_order_is_rejected(self):
        text = "pub fn put(mut self, k: K, v: V)\npub fn put(\n    mut self,\n    v: V,\n    k: K,\n)\n"
        found = list(lint.reading_order([("m.cp", text)]))
        self.assertEqual(len(found), 1)
        self.assertIn("m.cp:2: std-api/R8: `put` takes v, k; m.cp:1", found[0])


class IndexSpace(unittest.TestCase):
    def test_domain_position_is_accepted(self):
        text = "pub fn swap<D, T>(mut self: [D]T, i: D, j: D)\npub fn windows<D, T>(self: [D]T, k: Int)\n"
        self.assertEqual(list(lint.index_space([("s.cp", text)])), [])

    def test_int_position_over_a_sequence_is_rejected(self):
        text = "pub fn swap<D, T>(mut self: [D]T, i: Int, j: D)\n"
        found = list(lint.index_space([("s.cp", text)]))
        self.assertEqual(len(found), 1)
        self.assertIn("s.cp:1: std-api/R9: `swap` takes the position `i`", found[0])


LEXICON = """### R1. Every name comes from the lexicon

| Idea | Name | Not |
|---|---|---|
| size | `s.len` (a getter) | `len()`, `size()` |
| membership | `s.contains(x)` | `has`, `count(x) > 0` |
| component size | `d.size(v)` | |

### R2. Next
"""


class OffLexicon(unittest.TestCase):
    def test_banned_words_skip_own_words_and_calls(self):
        own, banned = lint.lexicon(LEXICON)
        self.assertEqual(banned, {"has"})
        self.assertIn("size", own)

    def test_a_lexicon_word_is_accepted(self):
        text = "pub struct G:\n    pub fn contains(self, u: Int) -> Bool = true\n"
        self.assertEqual(list(lint.off_lexicon([("g.cp", text)], {"has"})), [])

    def test_a_banned_word_is_rejected(self):
        text = "pub struct G:\n    pub fn has(self, u: Int) -> Bool = true\n"
        found = list(lint.off_lexicon([("g.cp", text)], {"has"}))
        self.assertEqual(len(found), 1)
        self.assertIn("g.cp:2: std-api/R1: `has`", found[0])


TABLE = """### R13. Journeys

| Type | Journeys |
|---|---|
| `SortedBag<T>` | add, k-th smallest (`nth`) |

### R14. Next
"""


class Journeys(unittest.TestCase):
    def test_every_journey_with_a_case_is_accepted(self):
        case = "// journey: add\nb.add(1)\n// journey: k-th smallest\nb.nth(0)\n"
        self.assertEqual(list(lint.journeys(TABLE, {"sorted-bag": case})), [])

    def test_a_journey_without_a_case_is_rejected(self):
        found = list(lint.journeys(TABLE, {"sorted-bag": "// journey: add\n"}))
        self.assertEqual(len(found), 1)
        self.assertIn("k-th smallest", found[0])

    def test_a_case_journey_the_table_lacks_is_rejected(self):
        case = "// journey: add\n// journey: k-th smallest\n// journey: erase\n"
        self.assertIn("not in the R13 table", list(lint.journeys(TABLE, {"sorted-bag": case}))[0])


if __name__ == "__main__":
    unittest.main()
