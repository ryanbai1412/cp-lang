// expect: ok, output "9 5"
// rule: 08.1.9
// A static fn returning `Tree<M.Fold>` under `M: Op3<E>` gets the bare operator's own `type_of(+)` impl (`Fold = E`) and a closure's fn-type impl (`Fold = ?E`) once `a` gives `E` (decision 456, Q6; G7.3).
trait Op3<T>:
    type Fold
    fn lift(self, x: T) -> Self.Fold
impl<T> Op3<T> for type_of(+):
    type Fold = T
    fn lift(self, x: T) -> T = x
impl<T> Op3<T> for (T, T) -> T:
    type Fold = ?T
    fn lift(self, x: T) -> ?T = x
struct Tree<F>(xs: List<F>):
    pub fn new<M: Op3<E>, E>(op: M, a: List<E>) -> Tree<M.Fold> = Tree(a.map(x => op.lift(x)))
let t = Tree.new(+, [4, 5])
let u = Tree.new((p: Int, q: Int) => p + q, [4, 5])
let s: Int = t.xs[0] + t.xs[1]
print(s, u.xs[1]!)
