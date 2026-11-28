// expect: ok, output "9"
// rule: R08.1.12
// engines: next
// A supertrait's fn without `self` is the trait's own member too.
trait Op<T>:
    fn op(a: T, b: T) -> T
trait Thrice<T>: Op<T>:
    fn thrice(a: T) -> T = op(op(a, a), a)
struct S: x: Int
impl Op<Int> for S:
    fn op(a: Int, b: Int) -> Int = a + b
impl Thrice<Int> for S
fn g<M: Thrice<Int>>(m: M) -> Int = M.thrice(3)
print(g(S(0)))
