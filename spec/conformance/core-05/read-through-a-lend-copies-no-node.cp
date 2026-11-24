// expect: ok
// rule: 05.6.3
// tool: build -o -
// has: "const N& n = Arena_"
// has: "a._xs[ll(i.raw)].l"
// lacks: "N n ="
// lacks: "-> N {"
// A field read through `at` reads the slot, and a `let` of `get` that nothing changes while it lives is a reference to the slot: neither copies the node.
use std.marks.{Arena, Id}
struct N:
    var l: Int
    var big: List<Int>
fn sum(a: Arena<N>, i: Id<N>) -> Int:
    let n = a.get(i)
    n.l + n.big.len
var a = Arena<N>()
let i = a.alloc(N(1, [2, 3]))
print(a.at(i).l + a.get(i).l, sum(a, i))
