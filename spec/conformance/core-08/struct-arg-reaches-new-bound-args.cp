// expect: ok, output "-10\n1 1"
// rule: 08.1.9
// A struct arg reaches `new`'s bound args: an explicit range `R` stays a range through `M: Op<E>`, and `T = M.Fold` lets `A: Tags<T, F>` give `F` (decision 456).
use std.algebra.{Op, Monoid}
trait Tags<T, F>:
    fn tag(self, x: T) -> F
struct Add()
impl<T> Tags<T, T> for Add:
    fn tag(self, x: T) -> T = x
struct Pairs()
impl<T> Tags<T, (T, T)> for Pairs:
    fn tag(self, x: T) -> (T, T) = (x, x)
struct NoTags()
impl<T> Tags<T, Int> for NoTags:
    fn tag(self, x: T) -> Int = 0
struct Tree<T, F = Int>(m: Monoid<T>):
    t: List<T>
    tags: List<F>
    sums: Bool
    pub fn new<M: Op<E>, E, A: Tags<T, F>>(op: M, a: ?List<E> = None, act: A = NoTags()) -> Tree<M.Fold, F>:
        let m = op.folds()
        let xs = if let v = a: [op.lift(x) for x in v] else: [m.id]
        Tree<M.Fold, F>(m, xs, [act.tag(x) for x in xs], false)
    @get pub fn all(self) -> T = t.fold(m)
type R = -10..=10
print(Tree<R>(max).all)
let p = Tree(+, [1, 2], act: Pairs())
print(p.tags[0])
