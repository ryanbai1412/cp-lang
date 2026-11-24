// expect: ok, output "10 7 34 8"
// rule: 05.6.3
// A field, tuple index or element, nested or not, under a `lends mut` call is a place: `=` and `+=` write into the arena's slot.
use std.marks.Arena
struct Node:
    var v: Int
    var kids: List<Int>
    var box: (w: Int, z: Int)
    var pair: (Int, Int)
var a = Arena<Node>()
let i = a.alloc(Node(v: 1, kids: [2, 3], box: (w: 4, z: 5), pair: (6, 7)))
a.at(i).v = 9
a.at(i).v += 1
a[i].kids[0] = 7
a[i].box.w += 30
a.at(i).pair.0 = 1
a[i].pair.0 += 7
print(a.at(i).v, a.at(i).kids[0], a.at(i).box.w, a.at(i).pair.0)
