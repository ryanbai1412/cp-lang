"""scripts/conformance-coverage.py reads a rule's `permits` kind from the spec, not a side list."""
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("coverage", ROOT / "scripts/conformance-coverage.py")
cov = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cov)


class PermitsTest(unittest.TestCase):
    def test_tag_parsed(self):
        m = cov.RULE_DEF.search("is legal and adds a flag. (R02.4.3, permits)")
        self.assertEqual((m.group(1), m.group(2)), ("02.4.3", ", permits"))
        self.assertIsNone(cov.RULE_DEF.search("rejects it. (R02.4.4)").group(2))

    def test_spec_marks_permits(self):
        rules = cov.rule_texts()
        self.assertIn("01.3.6", cov.PERMITS)
        self.assertIn("01.3.6", rules)
        self.assertNotIn("01.3.5", cov.PERMITS)

    def test_no_side_list(self):
        self.assertFalse((ROOT / "spec/conformance/permits.txt").exists())


if __name__ == "__main__":
    unittest.main()
