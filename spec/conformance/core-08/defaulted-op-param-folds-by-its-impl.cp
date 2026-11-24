// expect: ok, output "9 1 10 0 7 2 9"
// rule: 08.1.9
// A defaulted `op: M = NoOp2()` beside every way of passing the op: `M.Fold` resolves once M and E bind, from an arg, a fill or the struct arg (decision 456).
use std.algebra.{Op, Monoid}
struct NoOp2()
impl<T> Op<T> for NoOp2:
    type Fold = T
    fn op(self, a: T, b: T) -> T = a
    fn folds(self) -> Monoid<T> = Monoid(T.zero, (a, b) => a)
    fn lift(self, x: T) -> T = x
struct Tree<T>(m: Monoid<T>):
    t: List<T>
    pub fn new<M: Op<E>, E>(op: M = NoOp2(), a: ?List<E> = None, fill: ?E = None) -> Tree<M.Fold>:
        let m = op.folds()
        if let xs = a: return Tree<M.Fold>(m, [op.lift(x) for x in xs])
        if let f = fill: return Tree<M.Fold>(m, [op.lift(f), op.lift(f)])
        Tree<M.Fold>(m, [m.id, m.id])
    @get pub fn all(self) -> T = t.fold(m)
let a: Int = Tree(+, [4, 5]).all
let b = Tree((x: Int, y: Int) => min(x, y), [3, 1]).all!
let c: Int = Tree<Int>(+, fill: 5).all
let d: Int = Tree<Int>(+).all
let e = Tree<?Int>(Monoid<?Int>.of(min), [Some(7), None]).all!
let f: Int = Tree<Int>(max, [1, 2]).all
let g: Int = Tree(max, [1, 9]).all
print(a, b, c, d, e, f, g)
