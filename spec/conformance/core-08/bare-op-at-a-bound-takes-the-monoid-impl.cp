// expect: ok, output "5 3 6"
// rule: 08.1.9
// A bare operator passed for a param whose bound has an impl for `Monoid<T>` is that monoid and gets its associated types; a closure keeps its fn type and gets the fn-type impl's (decision 456, Q6).
trait Op3<T>:
    type Fold
    fn fold2(self, a: T, b: T) -> Self.Fold
impl<T> Op3<T> for Monoid<T>:
    type Fold = T
    fn fold2(self, a: T, b: T) -> T = self.op(a, b)
impl<T> Op3<T> for (T, T) -> T:
    type Fold = ?T
    fn fold2(self, a: T, b: T) -> ?T = Some(self(a, b))
fn go<M: Op3<Int>>(m: M) -> M.Fold = m.fold2(2, 3)
let x: Int = go(+)
let m: Int = go(max)
let y = go((a: Int, b: Int) => a * b)
print(x, m, y ?? 0)
