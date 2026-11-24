// expect: R0201 (checked)
// rule: 04.4.24
// A method's range-typed parameter is checked against its own argument: `self` takes no place among the arguments, so `a` checks `x`, not the `100` after it.
type Small = 0..=9
struct P:
    var xs: List<Int> = [0]
    fn f(self, a: Small, b: Int) -> Int = a + b
    pub fn g(self, x: Int) -> Int = self.f(x, 100) // <- here
let p = P()
print(p.g(42))
