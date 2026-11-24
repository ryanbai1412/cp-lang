// expect: ok, output "1 9 2 3 920"
// rule: 05.6.3
// A `let` of a node read through a lend is a value: writing the slot afterwards leaves it as it was.
use std.marks.{Arena, Id}
struct N:
    var l: Int
    var big: List<Int>
fn swap_in(mut a: Arena<N>, i: Id<N>) -> Int:
    let n = a.get(i)
    a.at(i).l = 20
    n.l * 100 + a.at(i).l
var a = Arena<N>()
let i = a.alloc(N(1, [2, 3]))
let n = a.get(i)
a.at(i).l = 9
print(n.l, a.at(i).l, n.big, swap_in(mut a, i))
