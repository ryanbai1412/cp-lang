// expect: ok, output "5 max"
// rule: std.algebra.opt
// A fn type with an impl of its own still answers the std fns that take it as `self`: `f.opt()` beside `f.name()`.
trait Named:
    fn name(self) -> Str
impl Named for (Int, Int) -> Int:
    fn name(self) -> Str = "max"
let f = (a: Int, b: Int) => if a > b: a else: b
print([Some(1), None, Some(5)].fold(f.opt())!, f.name())
