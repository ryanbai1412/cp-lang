// expect: ok, output "6 -1"
// rule: R08.1.12
// engines: next
// The trait's own members come before the module's fns: inside the trait, `op` is the trait's.
fn op(a: Int, b: Int) -> Int = -1
trait Op<T>:
    fn op(a: T, b: T) -> T
    fn twice(a: T) -> T = op(a, a)
struct S: x: Int
impl Op<Int> for S:
    fn op(a: Int, b: Int) -> Int = a + b
fn g<M: Op<Int>>(m: M) -> Int = M.twice(3)
print(g(S(0)), op(1, 2))
