// expect: ok, output "7 2\n5 2\n7 2\n6"
// A `lends mut` fn's one expression is the place it lends, so a write through the call lands
// in the argument, for a static fn and a projection alike (R05.6.3, R05.8.3).
fn head(mut xs: List<Int>) -> lends mut Int = xs[0]
struct P:
    var xs: List<Int>
    pub fn lo(mut self) -> lends mut Int = self.xs[0]
var a = [1, 2]
head(mut a) = 7
print(a[0], a[1])
var p = P(xs: [1, 2])
p.lo() = 5
print(p.xs[0], p.xs[1])
p.lo() = 3
p.lo() += 4
print(p.xs[0], p.xs[1])
let f: (mut Int, Int) -> () = (mut s, d) => s += d
var x = 1
f(mut x, 5)
print(x)
