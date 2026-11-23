import importlib.util
import unittest
from pathlib import Path

_spec = importlib.util.spec_from_file_location("gaps", Path(__file__).parent / "conformance-gaps.py")
gaps = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gaps)


class SuspectsTest(unittest.TestCase):
    def test_same_code_is_a_case_bug(self) -> None:
        log = "  core-09/x.cp: fail: wanted trap R0001 at 6; got: error[R0001]: ln(-1)\n"
        self.assertEqual(gaps.suspects(log), [("core-09/x.cp", "same-code", "error[R0001]: ln(-1)")])

    def test_printed_optional_is_a_case_bug(self) -> None:
        log = "  a/b.cp: fail: wanted ok; got: error[E0606]: `xs.min()` has type `?Int`, which does not implement `Show`\n"
        self.assertEqual(gaps.suspects(log)[0][1], "optional")

    def test_compiler_gap_is_not_flagged(self) -> None:
        log = "  a/c.cp: fail: wanted ok; got: error[E0203]: a `?Range<Int>` has no member `len`\n"
        self.assertEqual(gaps.suspects(log), [])


if __name__ == "__main__":
    unittest.main()
