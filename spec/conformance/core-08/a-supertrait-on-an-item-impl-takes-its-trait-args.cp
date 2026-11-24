// expect: ok, output "5"
// rule: 08.1.9
// `HasI<T>: Op2<T>` on `type_of(*)`: the supertrait's impl is checked at the T that `HasI<Int>` fixes, so its `T: Mul` bound holds (G7.3).
trait Op2<T>:
    fn op2(self, a: T, b: T) -> T
trait HasI<T>: Op2<T>:
    fn i(self) -> T
impl<T: Mul> Op2<T> for type_of(*):
    fn op2(self, a: T, b: T) -> T = a * b
impl<T: Mul + One> HasI<T> for type_of(*):
    fn i(self) -> T = T.one
fn f<M: HasI<Int>>(m: M) -> Int = m.op2(m.i(), 5)
print(f(*))
