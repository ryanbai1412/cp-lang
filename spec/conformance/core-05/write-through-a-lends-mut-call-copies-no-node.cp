// expect: ok
// rule: 05.6.3
// tool: build -o -
// has: "a._xs[ll(i.raw)].v = 9;"
// has: "a._xs[ll(i.raw)].kids[0] = 7;"
// lacks: "Node t"
// Writing a field through `at` is a store into the slot in C++, never a copy of the node.
use std.marks.Arena
struct Node:
    var v: Int
    var kids: List<Int>
var a = Arena<Node>()
let i = a.alloc(Node(v: 1, kids: [2, 3]))
a.at(i).v = 9
a[i].kids[0] = 7
a.at(i).v += 1
print(a.at(i).v)
