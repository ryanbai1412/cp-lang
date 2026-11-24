// expect: ok, output "5"
// rule: 08.1.9
// `Box<Int>(..)` writes `T` alone: `M` follows `new`'s return, `Box<T, N>` with `N = Z`, not the struct's default `Monoid<T>`.
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

fn first(b: Box<Int, Z>) -> Int = b.v[0]

let b = Box<Int>(Z(), 5)
print(first(b))
