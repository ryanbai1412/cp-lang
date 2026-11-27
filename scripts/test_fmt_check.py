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


class SupportTest(unittest.TestCase):
    def test_a_formatted_copy_gets_the_grader_and_tests_beside_it(self):
        files = [
            "p/dream.cp",
            "p/checker.py",
            "p/dream.tests/grader.cpp",
            "p/dream.tests/1.in",
            "p/tests/2.in",
            "p/deep/x/y.txt",
            "q/readme.md",
        ]
        self.assertEqual(
            fmt_check.support(["p/dream.cp"], files),
            ["p/checker.py", "p/dream.tests/grader.cpp", "p/dream.tests/1.in", "p/tests/2.in"],
        )


class SnippetTest(unittest.TestCase):
    def test_a_timeout_gets_one_more_try_alone(self):
        tries = {}

        def judge(p):
            tries[p] = tries.get(p, 0) + 1
            if p == "slow.cp" and tries[p] == 1:
                return "cpl run passes 20 s on the original, so formatting can't be checked"
            return "cpl run output differs" if p == "bad.cp" else None
        verdicts = fmt_check.judged(["ok.cp", "slow.cp", "bad.cp"], judge, 3)
        self.assertEqual(verdicts, {"ok.cp": None, "slow.cp": None, "bad.cp": "cpl run output differs"})
        self.assertEqual(tries, {"ok.cp": 1, "slow.cp": 2, "bad.cp": 1})

    def test_takes_each_cp_block_with_or_without_attributes(self):
        doc = "# t\n```cp\nprint(1)\n```\n```cpp\nint x;\n```\n```cp run input=\"5\"\nprint(2)\n```\n"
        self.assertEqual(
            fmt_check.snippets([("docs/a.md", doc)]),
            {"docs__a.md__0.cp": "print(1)\n", "docs__a.md__1.cp": "print(2)\n"},
        )


if __name__ == "__main__":
    unittest.main()
