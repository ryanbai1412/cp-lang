// expect: ok, output "103 105"
// rule: 04.4.24
// The argument after a method's range-typed parameter is checked against its own type, not the range before it.
type Small = 0..=9
struct P:
    var xs: List<Int> = [0]
    fn f(self, a: Small, b: Int) -> Int = a + b
    pub fn g(self, x: Int) -> Int = self.f(x, 100)
let p = P()
let x: Int = 5
print(p.g(3), p.f(x, 100))
