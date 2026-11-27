// expect: ok, output "max 6 max"
// rule: R08.1.10
// A closure or fn item called as a receiver takes the methods of its fn type's impl, with the
// closures it holds.
trait Named:
    fn name(self) -> Str
    fn twice(self, a: Int) -> Int
impl Named for (Int, Int) -> Int:
    fn name(self) -> Str = "max"
    fn twice(self, a: Int) -> Int = self(a, a) + 1
let k = 3
let f = (a: Int, b: Int) => if a > b: a else: b + k
fn g(a: Int, b: Int) -> Int = a + b
print(f.name(), f.twice(2) + g.twice(0) - 1, g.name())
