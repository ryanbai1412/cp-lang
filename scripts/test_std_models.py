import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("std_models", Path(__file__).with_name("std-models.py"))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

STD = [("std/q.cp", """\
pub struct Q<T>(
    xs: List<T>,
)

impl Q<T>:
    pub fn push(mut self, x: T):
        xs.push(x)
    @get pub fn len(self) -> Int = xs.len

pub struct Frozen(n: Int)

impl Frozen:
    pub fn n(self) -> Int = n

pub fn total(xs: List<Int>) -> Int = 0

pub fn size(self: Q<Int>) -> Int = 0

fn _helper() = 0

struct Hidden(n: Int)

impl Hidden:
    pub fn bump(mut self):
        n += 1
""")]


class StdModelsTest(unittest.TestCase):
    def test_a_public_type_with_a_method_and_a_pub_fn_need_a_model(self):
        self.assertEqual(m.std_types(STD), {"Q": "std/q.cp", "Frozen": "std/q.cp"})
        self.assertEqual(m.gaps(STD, []), [
            "std/q.cp: `Frozen` has no model case in spec/conformance/std-models/",
            "std/q.cp: `Q` has no model case in spec/conformance/std-models/",
            "std/q.cp: `size` has no model case in spec/conformance/std-models/",
            "std/q.cp: `total` has no model case in spec/conformance/std-models/",
        ])

    def test_a_type_model_covers_its_self_fns_and_a_brute_line_covers_a_fn(self):
        self.assertEqual(m.gaps(STD, [("c/q.cp", "// model: Q\n// model: Frozen\n// brute: total\nprint(\"ok\")\n")]), [])

    def test_a_model_of_no_std_type_or_fn_is_reported(self):
        self.assertEqual(m.gaps(STD, [("c/q.cp", "// model: Q\n// model: Frozen\n// model: total\n// model: Nope\n")]),
                         ["c/q.cp: `// model: Nope` names no std type or pub std fn"])

    def test_shrink_finds_the_shortest_failing_prefix(self):
        self.assertEqual(m.shrink(lambda n: n >= 17, 300), 17)
        self.assertEqual(m.shrink(lambda n: True, 300), 1)


if __name__ == "__main__":
    unittest.main()
