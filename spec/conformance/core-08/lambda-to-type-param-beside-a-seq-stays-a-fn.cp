// expect: ok, output "2 2 8"
// rule: 08.2.11
// A lambda or operator given to an open `X` is a fn even when another parameter is a sequence: the sequence's element type never becomes `X`.
fn ap<X, S>(op: X, s: S) -> X = op
print(ap((a, b) => a - b, [1])(5, 3), ap(-, [1])(5, 3), ap((a, b) => a * b, 0..3)(2, 4))
