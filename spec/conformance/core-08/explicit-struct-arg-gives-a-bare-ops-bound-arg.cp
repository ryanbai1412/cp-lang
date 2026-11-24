// expect: ok, output "9 0 0\n5"
// rule: 08.1.9
// `Tree<Int>(+)` through `fn new<M: Op<E>, E>(..) -> Tree<M.Fold>`: the struct's `T = Int` is `M.Fold`, and the `Monoid<E>` a bare op takes folds to `E`, so `E = Int` (decision 456).
use std.algebra.{Op, Monoid}
struct Tree<T>(m: Monoid<T>):
    t: List<T>
    pub fn new<M: Op<E>, E>(op: M, a: ?List<E> = None) -> Tree<M.Fold>:
        let m = op.folds()
        if let xs = a: return Tree<M.Fold>(m, [op.lift(x) for x in xs])
        Tree<M.Fold>(m, [m.id])
    @get pub fn all(self) -> T = t.fold(m)
struct B<D, T>(t: List<T>):
    fn new<M: Op<E>, E>(m: M, a: ?List<E> = None) -> B<D, M.Fold> = B<D, M.Fold>([])
let c: Int = Tree(+, [4, 5]).all
let d: Int = Tree<Int>(+).all
let b = B<Int, Int>.new(+)
print(c, d, b.t.len)
let e = Tree<Int>(max, [5, 2]).all
print(e)
