// expect: ok, output "1 2\n3\n4\n1 2 9"
// `f(by:)` and `(by:)` pass the variable the label names, as `by: by` does, beside a `mut` argument.
struct P:
    pub a: Int
    pub b: Int
fn grow(mut xs: List<Int>, x: Int):
    xs.push(x)
let a = 1
let b = 2
let p = P(a:, b:)
print(p.a, p.b)
let r = (a:, c: 3)
print(r.c)
print(P(b:, a: 3).b + r.a + p.a)
var xs = [1, 2]
let x = 9
grow(mut xs, x:)
print(xs)
