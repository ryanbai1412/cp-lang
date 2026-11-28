// expect: ok, output "7 YES"
// rule: 08.3.1
// A trait's law calls the trait's fns unqualified, and `laws_hold` on an implementing struct runs them as that impl's fns.
trait Op<T>:
    fn op(a: T, b: T) -> T
    law assoc(a: T, b: T, c: T) where T: Eq = op(op(a, b), c) == op(a, op(b, c))
struct S: x: Int
impl Op<Int> for S:
    fn op(a: Int, b: Int) -> Int = a + b
print(S.op(3, 4), yes(laws_hold(S(0), [1, 2, 3])))
