// expect: E0602 at 7
// rule: 05.7.5 12.4.11
// A closure borrows its captures per call: `f` reading `a` inside `g` overlaps the `mut a` lend of that call. Between calls `a` may change (core-12/closure-borrows-per-call).
fn g(mut xs: List<Int>, h: () -> Int) -> Int:
    xs.push(h())
    return xs.len
var a = [1]
let f = () => a.len
print(g(mut a, f))
