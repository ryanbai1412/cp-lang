// expect: ok, output "1 5"
// rule: same
// Same, the callee reassigns the global whole.
var g = [1, 2, 3, 4]
fn f(a: List<Int>) -> Int:
    g = [9, 9, 9, 9, 9]
    a[0] * 1
print(f(g), g.len)
