// expect: ok, output "5 3 20 7 -1 1"
// rule: 08.2.11
// An operator or overload set given to a type parameter nothing else fixes takes `Int`s, as a literal does: `ap(+)` is the `(Int, Int) -> Int` it returns.
fn ap<X>(op: X) -> X = op
fn use_op<M>(op: M) -> Int = 1
let k: (Int, Int) -> Int = ap(*)
print(ap(+)(2, 3), ap(max)(2, 3), k(4, 5), ap((a, b) => a * b + 1)(2, 3), ap((a: Int, b: Int) => a - b)(2, 3), use_op(max))
