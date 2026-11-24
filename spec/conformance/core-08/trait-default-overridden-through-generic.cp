// expect: ok, output "0 5 5"
// rule: 08.1.4
// A trait default overridden in a type's impl is overridden everywhere, generic code included (decision 486).
trait Tot:
    fn tot(self) -> Int = 0
struct A(x: Int)
struct B(x: Int)
impl Tot for A
impl Tot for B:
    fn tot(self) -> Int = 5
fn g<T: Tot>(v: T) -> Int = v.tot()
print(g(A(1)), g(B(1)), B(1).tot())
