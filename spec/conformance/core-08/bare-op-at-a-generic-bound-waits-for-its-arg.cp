// expect: ok, output "5 6 4"
// rule: 08.1.9
// A bound with a generic arg, `M: Op3<E>`, picks its impl once a later argument gives `E`: a bare operator takes its own type's impl (`type_of(+)`) and folds to `E`, a closure takes the fn-type impl and folds to `?E` (decision 456, Q6; G7.3).
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
    fn fold2(self, a: T, b: T) -> ?T = Some(self(a, b))
fn go<M: Op3<E>, E>(m: M, a: List<E>) -> M.Fold = m.fold2(a[0], a[1])
let x: Int = go(+, [2, 3])
let y = go((p: Int, q: Int) => p * q, [2, 3])
let z: Int = go(max, [4, 1])
print(x, y!, z)
