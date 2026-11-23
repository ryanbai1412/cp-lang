// expect: ok, output "4 1\n1 3"
// rule: R11.1.4
// A field takes `pub` the way a method does: in a body, in a one-line body and in a header.
// Inside its own module every member is visible, so `z` and `y` need no `pub` here.
pub struct S:
    pub x: Int
    pub var y: Int = 0
    z: Int
    pub fn f(self) -> Int = self.x + self.z
    @get pub fn g(self) -> Int = self.x
print(S(1, 2, 3).f(), S(1, 2, 3).g)
struct P(pub x: Int, y: Int)
struct Q: pub a: Int, b: Int
print(P(1, 2).x, Q(3, 4).a)
