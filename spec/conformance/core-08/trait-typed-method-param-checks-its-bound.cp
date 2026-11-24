// expect: E0302
// rule: 08.2.1
// A trait-typed parameter of a method is a bound checked at the call, as on a free fn: `W(4)` is refused there, naming the bound, and the body never sees an `Int`.
trait Big<V>:
    fn big(self) -> V
struct A(x: Int)
impl Big<Int> for A:
    fn big(self) -> Int = self.x * 10
struct W(n: Int):
    pub fn new(b: Big<Int>) -> W = W(b.big())
print(W(A(1)).n)
print(W(4).n)  // <- here
