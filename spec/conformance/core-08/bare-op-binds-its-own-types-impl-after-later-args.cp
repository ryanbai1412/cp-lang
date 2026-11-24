// expect: ok, output "5 6 6"
// rule: 08.1.9
// A bare operator passed for `M: Op3<E>` takes the impl on its own type once a later argument binds `E`, so its associated type is that impl's; a closure keeps the fn-type impl's (decision 456; G7.3).
trait Op3<T>:
    type Fold
    fn fold2(self, a: T, b: T) -> Self.Fold
impl<T: Add> Op3<T> for type_of(+):
    type Fold = T
    fn fold2(self, a: T, b: T) -> T = a + b
impl<T: Ord> Op3<T> for type_of(max):
    type Fold = T
    fn fold2(self, a: T, b: T) -> T = max(a, b)
impl<T> Op3<T> for (T, T) -> T:
    type Fold = ?T
    fn fold2(self, a: T, b: T) -> ?T = self(a, b)
fn go<M: Op3<E>, E>(m: M, a: E, b: E) -> M.Fold = m.fold2(a, b)
let x: Int = go(+, 2, 3)
let m: Int = go(max, 2, 6)
let y = go((a: Int, b: Int) => a * b, 2, 3)
print(x, m, y ?? 0)
