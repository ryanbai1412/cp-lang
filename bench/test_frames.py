import importlib.util
import os
import unittest

spec = importlib.util.spec_from_file_location("frames", os.path.join(os.path.dirname(__file__), "frames.py"))
frames = importlib.util.module_from_spec(spec)
spec.loader.exec_module(frames)

SU = (
    "t.cpp:9:12:prog::ends_D2(int)::<lambda(auto:1&&)>\t80\tstatic\n"
    "t.cpp:5:31:std::vector<prog::Mod> prog::ends_D2(int)\t96\tstatic\n"
    "t.cpp:3:28:cpl::OptField<&prog::Span::deep> prog::span(const OptPos&, ll)\t112\tstatic\n"
)


class Frames(unittest.TestCase):
    def test_a_fn_frame_leaves_out_its_lambdas(self):
        self.assertEqual(frames.frame_of(SU, "ends_"), 96)
        self.assertEqual(frames.frame_of(SU, "span"), 112)

    def test_a_frame_over_its_bound_fails(self):
        messages, failed = frames.verdict([("cf2187f2", 160, 96)])
        self.assertTrue(failed)
        self.assertIn("160 B, over its bound of 96 B", messages[0])

    def test_a_missing_fn_fails(self):
        self.assertTrue(frames.verdict([("apio07p1", None, 112)])[1])

    def test_within_the_bound_passes(self):
        self.assertEqual(frames.verdict([("apio07p1", 112, 112)]), ([], False))

    def test_every_case_has_its_port(self):
        for package, _, _ in frames.CASES.values():
            self.assertTrue(os.path.exists(os.path.join(frames.ROOT, "corpus", "problems", package, "port.cp")))


if __name__ == "__main__":
    unittest.main()
