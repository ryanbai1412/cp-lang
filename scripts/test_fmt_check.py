import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fmt_check  # noqa: E402


class SampleInputTest(unittest.TestCase):
    def test_the_input_header_wins_and_unescapes(self):
        text = '// input: "3\\n1 2 3\\n"\nprint(1)\n'
        self.assertEqual(fmt_check.sample_input("x.cp", text), b"3\n1 2 3\n")

    def test_falls_back_to_the_first_test_beside_the_file_then_empty(self):
        with tempfile.TemporaryDirectory() as root:
            old, fmt_check.ROOT = fmt_check.ROOT, root
            try:
                os.makedirs(os.path.join(root, "p", "sol.tests"))
                for name, body in (("b.in", b"2"), ("a.in", b"1")):
                    with open(os.path.join(root, "p", "sol.tests", name), "wb") as f:
                        f.write(body)
                self.assertEqual(fmt_check.sample_input("p/sol.cp", "print(1)\n"), b"1")
                self.assertEqual(fmt_check.sample_input("q/other.cp", "print(1)\n"), b"")
            finally:
                fmt_check.ROOT = old


if __name__ == "__main__":
    unittest.main()
