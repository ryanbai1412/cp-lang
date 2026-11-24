// expect: ok, output "1"
// rule: 08.3.1
// A law names the trait's own methods unqualified.
trait Op<T>:
    fn op(a: T, b: T) -> T
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
print(1)
