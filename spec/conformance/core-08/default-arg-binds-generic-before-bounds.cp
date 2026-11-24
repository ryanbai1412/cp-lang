// expect: ok, output "3"
// rule: 08.1.9
// An omitted `m: M = Wrap()` binds `M` from its default before the bounds are checked, so a later bound may name `M.Fold` (decision 469).
trait Fo<T>:
    type Fold = ?T
    fn lift(self, x: T) -> Self.Fold
struct Wrap()
impl<T> Fo<T> for Wrap:
    fn lift(self, x: T) -> ?T = Some(x)
trait Takes<X>:
    fn take(self, x: X) -> Int
struct Unwrap()
impl Takes<?Int> for Unwrap:
    fn take(self, x: ?Int) -> Int = x ?? -1
fn go<M: Fo<T>, A: Takes<M.Fold>, T>(x: T, a: A, m: M = Wrap()) -> Int = a.take(m.lift(x))
print(go(3, Unwrap()))
