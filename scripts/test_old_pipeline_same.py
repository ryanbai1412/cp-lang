"""Tests for scripts/old-pipeline-same.py: what counts as the old pipeline doing something else."""
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("old_pipeline_same", HERE / "old-pipeline-same.py")
ops = importlib.util.module_from_spec(spec)
sys.modules["old_pipeline_same"] = ops
spec.loader.exec_module(ops)


def rec(exit="0", build="b", cpp="c", run="r"):
    r = {"exit": exit, "build": build, "cpp": cpp}
    if run is not None:
        r["run"] = run
    return r


class Changed(unittest.TestCase):
    def test_renumbered_symbols_mask_to_the_same_cpp(self):
        a = b"int64_t m10_bit_reverse_S23_282(int64_t x_4);"
        b = b"int64_t m10_bit_reverse_S23_280(int64_t x_9);"
        self.assertEqual(ops.SUFFIX.sub(b"#", a), ops.SUFFIX.sub(b"#", b))
        self.assertNotEqual(ops.SUFFIX.sub(b"#", a), ops.SUFFIX.sub(b"#", b"int64_t other_S23_280(int64_t x_9);"))

    def test_other_cpp_with_the_same_run_passes_and_is_counted(self):
        self.assertEqual(ops.changed(["a"], {"a": rec(cpp="x")}, {"a": rec(cpp="y")}), ([], 1))

    def test_a_changed_run_fails(self):
        differ, _ = ops.changed(["a"], {"a": rec(cpp="x", run="1")}, {"a": rec(cpp="y", run="2")})
        self.assertEqual([n for n, _ in differ], ["a"])

    def test_an_accepted_case_now_refused_fails(self):
        differ, _ = ops.changed(["a"], {"a": rec(exit="1", build="e", run=None)}, {"a": rec()})
        self.assertEqual(differ, [("a", "exit status or diagnostic codes")])

    def test_a_case_the_base_refused_may_change(self):
        ours = {"a": rec(), "b": rec(exit="1", build="E0305", run=None)}
        theirs = {"a": rec(exit="1", build="E0606", run=None), "b": rec(exit="1", build="E0606", run=None)}
        self.assertEqual(ops.changed(["a", "b"], ours, theirs), ([], 0))

    def test_codes_are_the_set_of_diagnostic_codes(self):
        self.assertEqual(ops.codes(b"error[E0606]: x\nerror[E0203]: y\nerror[E0606]: z"), b"E0203 E0606")

    def test_case_input_reads_the_header_escapes(self):
        self.assertEqual(ops.case_input('// expect: ok\n// input: "4 3\\n4 9\\n"\nprint(1)'), b"4 3\n4 9\n")
        self.assertEqual(ops.case_input("print(1)"), b"")

    def test_engines_next_cases_are_not_the_old_pipelines(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "spec/conformance/x").mkdir(parents=True)
            (root / "spec/conformance/x/old.cp").write_text("// expect: ok\nprint(1)\n")
            (root / "spec/conformance/x/new.cp").write_text("// expect: ok\n// engines: next\nprint(1)\n")
            self.assertEqual(ops.cases(root), ["spec/conformance/x/old.cp"])


if __name__ == "__main__":
    unittest.main()
