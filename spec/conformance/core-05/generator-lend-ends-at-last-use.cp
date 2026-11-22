// expect: ok, output "6\n5"
// rule: 05.7.3
fn evens(xs: List<Int>) -> yields Int:
    for x in xs if x % 2 == 0: yield x
var a = [1, 2, 3, 4]
let g = evens(a)
print(g.sum())
a.push(6)
print(a.len)
