// expect: E0602
// rule: R07.6.2
// A generator value holding a list argument is lent from it until its last walk.
fn each_of(xs: List<Int>) -> yields Int:
    for x in xs: yield x
var xs = [1, 2]
let g = each_of(xs)
xs.push(3) // <- here
for x in g: print(x)
