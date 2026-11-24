import importlib.util
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "std_api_snapshot", Path(__file__).resolve().parent / "std-api-snapshot.py")
snap = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(snap)

STD = """pub struct Bag<T>:
    xs: List<T>

    /// Adds one.
    @get pub fn len(self) -> Int = xs.len

    pub fn put(
        mut self,
        x: T,
        times: Int = 1,
    ):
        pass

pub fn merge<T>(a: Bag<T>, b: Bag<T>) -> Bag<T> = a
"""


class Entries(unittest.TestCase):
    def test_methods_carry_their_owner_and_one_line_signature(self):
        self.assertEqual(snap.entries([("std/bag.cp", STD)]), [
            "std/bag.cp: Bag.len(self) -> Int",
            "std/bag.cp: Bag.put(mut self, x: T, times: Int = 1)",
            "std/bag.cp: merge<T>(a: Bag<T>, b: Bag<T>) -> Bag<T>",
        ])


class Unannounced(unittest.TestCase):
    OLD = ["std/bag.cp: Bag.len(self) -> Int", "std/bag.cp: Bag.put(mut self, x: T)"]
    NEW = ["std/bag.cp: Bag.len(self) -> Int"]

    def test_a_removal_with_a_codemod_is_announced(self):
        self.assertEqual(snap.unannounced(self.OLD, self.NEW, '"put" => &["insert"]', ""), [])

    def test_a_removal_in_the_changelog_is_announced(self):
        self.assertEqual(snap.unannounced(self.OLD, self.NEW, "", "- `put` became `insert`"), [])

    def test_a_silent_removal_is_rejected(self):
        self.assertEqual(snap.unannounced(self.OLD, self.NEW, "", ""), ["Bag.put"])

    def test_a_changed_signature_keeps_its_name(self):
        new = self.NEW + ["std/bag.cp: Bag.put(mut self, x: T, times: Int = 1)"]
        self.assertEqual(snap.unannounced(self.OLD, new, "", ""), [])


if __name__ == "__main__":
    unittest.main()
