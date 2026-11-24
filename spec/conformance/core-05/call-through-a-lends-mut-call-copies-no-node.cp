// expect: ok
// rule: 05.6.3
// tool: build -o -
// has: "a._xs[ll(i.raw)].kids.push_back(4);"
// has: "C_bump(a._xs[ll(i.raw)].c, 1);"
// lacks: "Node t"
// lacks: "-> Node {"
// A `mut self` call through `at` runs on the slot in C++, never on a copy of the node.
use std.marks.Arena
struct C:
    var n: Int
    fn bump(mut self, by: Int):
        n += by
struct Node:
    var kids: List<Int>
    var c: C
var a = Arena<Node>()
let i = a.alloc(Node([1], C(0)))
a.at(i).kids.push(4)
a[i].c.bump(1)
print(a.at(i).kids, a.at(i).c.n)
