// expect: ok, output "7 9\n5 6"
// rule: R06.1.2, R08.2.11
// A lambda's unwritten parameter types come from where it is passed: a later argument can fix
// them, and a type parameter nothing else fixes gives them `Int`.
fn apply<T, U>(f: T -> U, x: T) -> U = f(x)
fn ap<X>(op: X) -> X = op
struct P:
    a: Int
    b: Int
print(apply(p => p.a + p.b, P(3, 4)), apply(n => n * 3, 3))
print(ap((a, b) => a + b)(2, 3), ap(n => n + 1)(5))
