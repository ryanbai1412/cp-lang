// expect: ok, output "12"
// rule: R08.1.12
// engines: next
// A parameter is nearer than the trait's members: here `op` is the lambda passed in.
trait Op<T>:
    fn op(a: T, b: T) -> T
    fn via(a: T, op: (T, T) -> T) -> T = op(a, a)
struct S: x: Int
impl Op<Int> for S:
    fn op(a: Int, b: Int) -> Int = a + b
fn g<M: Op<Int>>(m: M) -> Int = M.via(3, (a, b) => a * b + a)
print(g(S(0)))
