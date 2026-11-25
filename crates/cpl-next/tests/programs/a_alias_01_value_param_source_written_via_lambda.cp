// expect: ok, output "1 5"
// rule: R05 R4/R10; conformance core-05/param-unchanged-by-a-callee-writing-its-source
// R4/R10 (conformance core-05/param-unchanged-by-a-callee-writing-its-source): a value parameter keeps the value it was called with while the callee writes the global it came from.
var g = [1, 2, 3, 4]
fn f(a: List<Int>) -> Int:
    g.push(5)
    a[0] * 1
fn go() -> Int:
    let h = () => f(g)
    h()
print(go(), g.len)
