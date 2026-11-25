// expect: ok, output "1 5"
// rule: same
// Same where the write is two calls deep.
var g = [1, 2, 3, 4]
fn grow():
    g.push(5)
fn f(a: List<Int>) -> Int:
    grow()
    a[0] * 1
print(f(g), g.len)
