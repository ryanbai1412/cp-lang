// expect: ok, output "5 6"
// rule: R08.1.9 R08.1.10
// A bare operator at a bound takes the `@from_op` struct's impl; a closure at a bound only a fn type
// has an impl of is passed as that fn type, and the impl is the fn type's for whatever fns it holds.
struct Sum<T>(id: T, op: (T, T) -> T):
    @from_op
    pub fn of<M>(m: M) -> Sum<Int> = Sum(0, (a, b) => a + b)
trait Op3<T>:
    type Fold
    fn fold2(self, a: T, b: T) -> Self.Fold
impl<T> Op3<T> for Sum<T>:
    type Fold = T
    fn fold2(self, a: T, b: T) -> T = self.op(self.op(self.id, a), b)
impl<T> Op3<T> for (T, T) -> T:
    type Fold = ?T
    fn fold2(self, a: T, b: T) -> ?T = Some(self(a, b))
fn go<M: Op3<E>, E>(m: M, a: List<E>) -> M.Fold = m.fold2(a[0], a[1])
let x: Int = go(+, [2, 3])
let y = go((p: Int, q: Int) => p * q, [2, 3])
print(x, y!)
