// expect: ok, output "6"
// rule: R08.1.12
// engines: next
// Inside a trait's fn, a bare name finds the trait's own fn without `self`, as `Self.op` would.
trait Op<T>:
    fn op(a: T, b: T) -> T
    fn twice(a: T) -> T = op(a, a)
struct S: x: Int
impl Op<Int> for S:
    fn op(a: Int, b: Int) -> Int = a + b
fn g<M: Op<Int>>(m: M) -> Int = M.twice(3)
print(g(S(0)))
