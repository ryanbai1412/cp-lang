import importlib.util
import os
import unittest

spec = importlib.util.spec_from_file_location("compile_time", os.path.join(os.path.dirname(__file__), "compile_time.py"))
ct = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ct)


class Verdict(unittest.TestCase):
    def test_programs_within_the_bars_pass(self):
        _, failed = ct.verdict({"a.cp": 0.5, "b.cp": 0.6, "c.cp": ct.MAX_BAR})
        self.assertFalse(failed)

    def test_one_program_over_the_max_bar_fails(self):
        messages, failed = ct.verdict({"a.cp": 0.5, "b.cp": 0.6, "c.cp": ct.MAX_BAR + 0.1})
        self.assertTrue(failed)
        self.assertTrue(any("c.cp" in m for m in messages))

    def test_a_median_over_its_bar_fails(self):
        _, failed = ct.verdict({p: ct.P50_BAR + 0.1 for p in ("a.cp", "b.cp", "c.cp")})
        self.assertTrue(failed)

    def test_nothing_compiled_fails(self):
        self.assertTrue(ct.verdict({})[1])

    def test_the_bars_leave_room_over_the_measured_median(self):
        self.assertLess(ct.P50_BAR, ct.MAX_BAR)


if __name__ == "__main__":
    unittest.main()
