// expect: ok, output "5 6"
// rule: 08.1.9
// A bare operator passed for `M: Op3<E>` takes the impl for whichever struct marks a static fn `@from_op`, a user's as much as std's `Monoid` (R08.1.9).
struct Sum<T>(id: T, op: (T, T) -> T):
    @from_op
    pub fn of<M>(m: M) -> Sum<T> = Sum(T.zero, (a, b) => a + b)
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
