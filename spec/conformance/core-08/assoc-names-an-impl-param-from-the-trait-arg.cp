// expect: ok, output "4 5"
// rule: 08.1.9
// An impl param named only in the trait argument is the one the bound solved, so `Fold = T` names it.
trait Op3<T>:
    type Fold = ?T
    fn lift(self, x: T) -> Self.Fold = Some(x)
struct E3(k: Int)
impl<T> Op3<T> for E3:
    type Fold = T
    fn lift(self, x: T) -> T = x
fn mk<M: Op3<E>, E>(op: M, a: List<E>) -> List<M.Fold> = [op.lift(x) for x in a]
print(mk(E3(0), [4, 5]))
