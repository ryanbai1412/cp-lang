// expect: E0602 at 5
// rule: 05.7.3 06.3.2 07.6.2
fn evens(xs: List<Int>) -> yields Int:
    for x in xs if x % 2 == 0: yield x
var a = [1, 2, 3, 4]
let g = evens(a)
a.push(6)
print(g.sum())
