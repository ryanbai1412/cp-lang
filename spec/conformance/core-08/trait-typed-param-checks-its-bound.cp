// expect: E0606
// rule: 08.2.1
// A trait-typed parameter is a bound on its argument's type, as `<B: Big<Int>>` would be: an argument whose type has no impl of the trait is refused at the call.
trait Big<V>:
    fn big(self) -> V
struct A(x: Int)
impl Big<Int> for A:
    fn big(self) -> Int = self.x * 10
fn show(b: Big<Int>) -> Int = b.big()
print(show(A(3)), show(4))  // <- here
