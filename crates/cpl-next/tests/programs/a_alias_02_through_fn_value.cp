// expect: ok, output "1 5"
// rule: same
// Same through a function value held in a var.
var g = [1, 2, 3, 4]
fn f(a: List<Int>) -> Int:
    g.push(5)
    a[0] * 1
var h = f
print(h(g), g.len)
