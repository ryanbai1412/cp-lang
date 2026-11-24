// expect: E0302
// rule: 08.1.9
// has: "help: write `Box<Int, Z>` in full: a type param left out takes its default"
use std.algebra.{Monoid, Op}
struct Z
fn x0<T>() -> T:
    let n: ?T = None
    n!
impl<T> Op<T> for Z:
    type Fold = T
    fn op(self, a: T, b: T) -> T = a
    fn folds(self) -> Monoid<T> = Monoid(x0(), (a, _) => a)
    fn lift(self, x: T) -> T = x
struct Box<T, M: Op<T> = Monoid<T>>(op: M):
    v: List<M.Fold> = []
    fn new<N: Op<T>>(op: N, x: T) -> Box<T, N>:
        var b = Box<T, N>(op)
        b.put(x)
        b
    fn put(mut self, x: T):
        v.push(op.lift(x))

fn first(b: Box<Int>) -> Int = b.v[0]

let b = Box(Z(), 5)
print(first(b)) // <- here
