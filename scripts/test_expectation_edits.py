import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("ee", Path(__file__).with_name("expectation-edits.py"))
ee = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ee)


class FlaggedTest(unittest.TestCase):
    def test_modified_expectation_beside_code_is_flagged(self) -> None:
        changes = [("M", "crates/cpl-cli/tests/programs/a.trace"), ("M", "crates/cpl-interp/src/vm/mod.rs")]
        self.assertEqual(ee.flagged(changes),
                         (["crates/cpl-cli/tests/programs/a.trace"], ["crates/cpl-interp/src/vm/mod.rs"]))

    def test_new_expectations_and_test_only_code_are_not(self) -> None:
        self.assertIsNone(ee.flagged([("A", "tests/a.out"), ("M", "crates/x/src/lib.rs")]))
        self.assertIsNone(ee.flagged([("M", "tests/a.out"), ("M", "crates/x/src/tests.rs")]))
        self.assertIsNone(ee.flagged([("M", "tests/a.out"), ("M", "crates/x/tests/t.rs")]))

    def test_std_source_counts_as_code(self) -> None:
        self.assertIsNotNone(ee.flagged([("M", "x.err"), ("M", "std/seq.cp")]))


if __name__ == "__main__":
    unittest.main()
