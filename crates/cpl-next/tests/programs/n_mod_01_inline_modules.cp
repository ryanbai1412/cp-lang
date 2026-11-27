// expect: ok, output "1 3\n7 2\n12 5\n4"
// An inline `mod m:` is a module of its own: the file names its `pub` items as `m.f`, a
// sibling imports them with `use`, and inside it every member is visible (R11.1.3, R11.1.4).
mod geo:
    pub struct P:
        pub x: Int
        y: Int
        pub fn sum(self) -> Int = self.x + self.y
    pub fn make(x: Int) -> P = P(x: x, y: 2)
    pub fn peek(p: P) -> Int = p.y
    pub const BASE: Int = 7
mod calc:
    use geo.make
    use std.math.gcd
    pub fn twice(n: Int) -> Int = make(n).sum() + make(n).x
    pub fn g() -> Int = gcd(10, 15)
fn far(p: geo.P) -> Int = p.sum() * 4
let p = geo.make(1)
print(p.x, p.sum())
print(geo.BASE, geo.peek(p))
print(far(p), calc.g())
print(calc.twice(1))
