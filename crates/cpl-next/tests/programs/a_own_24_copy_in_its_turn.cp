// expect: ok, output "1 3"
// rule: R11
// `f` writes `xs`, so `xs` is copied in, in its turn: before `g()`, the next argument, pushes.
var xs = [1]
fn g() -> Int:
    xs.push(2)
    0
fn f(a: List<Int>, n: Int):
    xs.push(3)
    print(a.len + n, xs.len)
f(xs, g())
